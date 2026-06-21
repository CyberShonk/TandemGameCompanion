use std::fs::{File, OpenOptions};
use std::io::Write as IoWrite;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::config::{
    BeforeGameWait, LaunchTiming, ResolvedConfig, ResolvedProgram, ResolvedTool, is_windows_script,
};
use crate::error::AppError;
use crate::{platform, protocol};

const DELAY_POLL_INTERVAL: Duration = Duration::from_millis(50);

struct SessionLog {
    file: File,
}

struct RunningTool {
    name: String,
    child: Child,
    close_when_game_exits: bool,
}

impl SessionLog {
    fn open(path: &std::path::Path) -> Result<Self, AppError> {
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)
            .map_err(|source| {
                AppError::io(
                    format!("could not open log file {}", path.display()),
                    source,
                )
            })?;

        Ok(Self { file })
    }

    fn line(&mut self, message: impl AsRef<str>) -> Result<(), AppError> {
        let message = message.as_ref();
        println!("{message}");
        std::io::stdout()
            .flush()
            .map_err(|source| AppError::io("could not flush Tandem output", source))?;
        writeln!(self.file, "{message}")
            .map_err(|source| AppError::io("could not write the Tandem session log", source))?;
        self.file
            .flush()
            .map_err(|source| AppError::io("could not flush the Tandem session log", source))
    }

    fn best_effort_line(&mut self, message: impl AsRef<str>) {
        let message = message.as_ref();
        println!("{message}");
        let _ = std::io::stdout().flush();
        let _ = writeln!(self.file, "{message}");
        let _ = self.file.flush();
    }

    fn child_stdio(&self) -> Result<(Stdio, Stdio), AppError> {
        let stdout = self.file.try_clone().map_err(|source| {
            AppError::io("could not clone the log handle for child output", source)
        })?;
        let stderr = self.file.try_clone().map_err(|source| {
            AppError::io("could not clone the log handle for child errors", source)
        })?;
        Ok((Stdio::from(stdout), Stdio::from(stderr)))
    }
}

pub fn print_plan(config: &ResolvedConfig) {
    println!("Configuration: {}", config.config_path.display());
    println!("Log: {}", config.log_file.display());
    println!();
    println!("Game:");
    print_program(&config.game);
    println!();
    println!("Companion tools:");
    if config.tools.is_empty() {
        println!("  (none)");
    } else {
        for tool in &config.tools {
            println!(
                "  {} [{}; wait={}; delay={}ms; required={}; close_with_game={}]",
                tool.program.name,
                launch_label(tool.launch),
                wait_label(tool.before_game_wait),
                tool.delay_ms,
                tool.required,
                tool.close_when_game_exits
            );
            println!("    path: {}", tool.program.path.display());
            println!(
                "    working directory: {}",
                tool.program.working_directory.display()
            );
            if !tool.program.arguments.is_empty() {
                println!("    arguments: {:?}", tool.program.arguments);
            }
        }
    }
}

pub fn run_worker(config: &ResolvedConfig) -> Result<(), AppError> {
    platform::protect_guardian_status_channel()?;
    let mut log = SessionLog::open(&config.log_file)?;
    let mut running_tools = Vec::new();

    log.line("Tandem Game Companion")?;
    log.line(format!("Configuration: {}", config.config_path.display()))?;

    let session_result = run_session(config, &mut running_tools, &mut log);
    let force_close = session_result.is_err();
    let cleanup_result = clean_up_tools(&mut running_tools, &mut log, force_close);

    match session_result {
        Err(error) => {
            if let Err(cleanup_error) = cleanup_result {
                log.best_effort_line(format!(
                    "Tool cleanup also failed while handling the session error: {cleanup_error}"
                ));
            }
            log.best_effort_line("Tandem session failed.");
            Err(error)
        }
        Ok(()) => {
            cleanup_result?;
            log.line("Tandem session finished.")?;
            Ok(())
        }
    }
}

fn run_session(
    config: &ResolvedConfig,
    running_tools: &mut Vec<RunningTool>,
    log: &mut SessionLog,
) -> Result<(), AppError> {
    for tool in config
        .tools
        .iter()
        .filter(|tool| tool.launch == LaunchTiming::BeforeGame)
    {
        start_before_game_tool(
            tool,
            config.continue_on_optional_tool_failure,
            running_tools,
            log,
        )?;
    }

    log.line(format!("Starting game: {}", config.game.name))?;
    let mut game = spawn_program(&config.game, log)?;
    if let Err(error) = log.line(format!("Game process started with PID {}", game.id())) {
        terminate_untracked_child(&config.game.name, &mut game, log);
        return Err(error);
    }

    if let Err(source) = protocol::report_game_pid(game.id()) {
        log.best_effort_line(
            "Could not report the game process; terminating the unsupervised game.",
        );
        let _ = game.kill();
        let _ = game.wait();
        return Err(AppError::io(
            "could not report the game process to the Tandem guardian",
            source,
        ));
    }

    if std::env::var_os("TANDEM_TEST_WORKER_EXIT_AFTER_GAME_START").is_some() {
        return Err(AppError::runtime(
            "simulated Tandem worker exit after game startup",
        ));
    }

    for tool in config
        .tools
        .iter()
        .filter(|tool| tool.launch == LaunchTiming::AfterGame)
    {
        if let Some(status) = wait_for_delay_or_game_exit(&mut game, tool, log)? {
            log.line(format!(
                "Game exited before {} could be launched; remaining delayed tools were skipped.",
                tool.program.name
            ))?;
            return game_result(status);
        }

        if let Some(status) = game
            .try_wait()
            .map_err(|source| AppError::io("could not inspect the game process", source))?
        {
            log.line(format!(
                "Game exited before {} could be launched; remaining tools were skipped.",
                tool.program.name
            ))?;
            return game_result(status);
        }

        if let Some(child) = spawn_tool(tool, config.continue_on_optional_tool_failure, log)? {
            running_tools.push(RunningTool {
                name: tool.program.name.clone(),
                child,
                close_when_game_exits: tool.close_when_game_exits,
            });
        }
    }

    log.line("Tandem is supervising the game process.")?;
    let game_status = game
        .wait()
        .map_err(|source| AppError::io("could not wait for the game process", source))?;
    log.line(format!("Game exited with status: {game_status}"))?;
    game_result(game_status)
}

fn start_before_game_tool(
    tool: &ResolvedTool,
    continue_on_optional_failure: bool,
    running_tools: &mut Vec<RunningTool>,
    log: &mut SessionLog,
) -> Result<(), AppError> {
    if tool.delay_ms > 0 {
        log.line(format!(
            "Waiting {} ms before starting {}.",
            tool.delay_ms, tool.program.name
        ))?;
        thread::sleep(Duration::from_millis(tool.delay_ms));
    }

    let Some(mut child) = spawn_tool(tool, continue_on_optional_failure, log)? else {
        return Ok(());
    };

    match tool.before_game_wait {
        BeforeGameWait::None => {
            running_tools.push(RunningTool {
                name: tool.program.name.clone(),
                child,
                close_when_game_exits: tool.close_when_game_exits,
            });
            Ok(())
        }
        BeforeGameWait::ToolExit => {
            log.line(format!(
                "Waiting for {} to finish before launching the game.",
                tool.program.name
            ))?;
            let status = match child.wait() {
                Ok(status) => status,
                Err(source) => {
                    terminate_untracked_child(&tool.program.name, &mut child, log);
                    return Err(AppError::io(
                        format!("could not wait for companion tool {}", tool.program.name),
                        source,
                    ));
                }
            };
            log.line(format!(
                "{} exited before game launch with status: {status}",
                tool.program.name
            ))?;
            handle_tool_exit(tool, status, continue_on_optional_failure, log)
        }
        BeforeGameWait::UserConfirmation => {
            running_tools.push(RunningTool {
                name: tool.program.name.clone(),
                child,
                close_when_game_exits: tool.close_when_game_exits,
            });
            log.line(format!(
                "Waiting for user confirmation after starting {}.",
                tool.program.name
            ))?;

            if !platform::confirm_before_game(&tool.program.name)? {
                return Err(AppError::runtime(format!(
                    "user cancelled game launch while configuring {}",
                    tool.program.name
                )));
            }

            log.line(format!(
                "User confirmed that {} is configured.",
                tool.program.name
            ))?;

            let index = running_tools.len() - 1;
            let status = running_tools[index].child.try_wait().map_err(|source| {
                AppError::io(
                    format!("could not inspect companion tool {}", tool.program.name),
                    source,
                )
            })?;
            if let Some(status) = status {
                running_tools.remove(index);
                log.line(format!(
                    "{} exited before confirmation completed with status: {status}",
                    tool.program.name
                ))?;
                handle_tool_exit(tool, status, continue_on_optional_failure, log)?;
            }

            Ok(())
        }
    }
}

fn spawn_tool(
    tool: &ResolvedTool,
    continue_on_optional_failure: bool,
    log: &mut SessionLog,
) -> Result<Option<Child>, AppError> {
    log.line(format!("Starting companion tool: {}", tool.program.name))?;
    match spawn_program(&tool.program, log) {
        Ok(mut child) => {
            if let Err(error) = log.line(format!(
                "{} started with PID {}.",
                tool.program.name,
                child.id()
            )) {
                terminate_untracked_child(&tool.program.name, &mut child, log);
                return Err(error);
            }
            Ok(Some(child))
        }
        Err(error) if tool.required || !continue_on_optional_failure => Err(error),
        Err(error) => {
            log.line(format!(
                "Optional tool {} could not be started: {error}",
                tool.program.name
            ))?;
            Ok(None)
        }
    }
}

fn handle_tool_exit(
    tool: &ResolvedTool,
    status: ExitStatus,
    continue_on_optional_failure: bool,
    log: &mut SessionLog,
) -> Result<(), AppError> {
    if status.success() {
        return Ok(());
    }

    if tool.required || !continue_on_optional_failure {
        return Err(AppError::process_exit(
            format!("companion tool {} failed", tool.program.name),
            status.code(),
        ));
    }

    log.line(format!(
        "Optional tool {} exited unsuccessfully; continuing because it is not required.",
        tool.program.name
    ))
}

fn wait_for_delay_or_game_exit(
    game: &mut Child,
    tool: &ResolvedTool,
    log: &mut SessionLog,
) -> Result<Option<ExitStatus>, AppError> {
    if tool.delay_ms == 0 {
        return Ok(None);
    }

    log.line(format!(
        "Waiting {} ms before starting {}.",
        tool.delay_ms, tool.program.name
    ))?;
    let deadline = Instant::now() + Duration::from_millis(tool.delay_ms);

    loop {
        if let Some(status) = game
            .try_wait()
            .map_err(|source| AppError::io("could not inspect the game process", source))?
        {
            return Ok(Some(status));
        }

        let now = Instant::now();
        if now >= deadline {
            return Ok(None);
        }

        thread::sleep((deadline - now).min(DELAY_POLL_INTERVAL));
    }
}

fn spawn_program(program: &ResolvedProgram, log: &SessionLog) -> Result<Child, AppError> {
    let mut command = build_command(program)?;
    let (stdout, stderr) = log.child_stdio()?;
    command
        .current_dir(&program.working_directory)
        .stdin(Stdio::inherit())
        .stdout(stdout)
        .stderr(stderr);

    command.spawn().map_err(|source| {
        AppError::io(
            format!(
                "could not start {} ({})",
                program.name,
                program.path.display()
            ),
            source,
        )
    })
}

fn build_command(program: &ResolvedProgram) -> Result<Command, AppError> {
    if is_windows_script(&program.path) {
        return build_windows_script_command(program);
    }

    let mut command = Command::new(&program.path);
    command.args(&program.arguments);
    Ok(command)
}

#[cfg(windows)]
fn build_windows_script_command(program: &ResolvedProgram) -> Result<Command, AppError> {
    use std::os::windows::process::CommandExt;

    let mut command = Command::new("cmd.exe");
    command.args(["/D", "/S", "/C"]);
    command.raw_arg(windows_script_command_line(
        &program.path,
        &program.arguments,
    ));
    Ok(command)
}

#[cfg(windows)]
fn windows_script_command_line(path: &std::path::Path, arguments: &[String]) -> String {
    let mut command_line = format!("call \"{}\"", cmd_compatible_path(path));
    for argument in arguments {
        command_line.push(' ');
        command_line.push('"');
        command_line.push_str(argument);
        command_line.push('"');
    }
    command_line
}

#[cfg(windows)]
fn cmd_compatible_path(path: &std::path::Path) -> String {
    let path = path.to_string_lossy();
    if let Some(path) = path.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{path}")
    } else if let Some(path) = path.strip_prefix(r"\\?\") {
        path.to_owned()
    } else {
        path.into_owned()
    }
}

#[cfg(not(windows))]
fn build_windows_script_command(program: &ResolvedProgram) -> Result<Command, AppError> {
    Err(AppError::usage(format!(
        "{} is a Windows BAT/CMD script and must be launched by a Windows Tandem build",
        program.path.display()
    )))
}

fn terminate_untracked_child(name: &str, child: &mut Child, log: &mut SessionLog) {
    log.best_effort_line(format!(
        "Terminating {name} because Tandem could not finish registering the process."
    ));
    let _ = child.kill();
    let _ = child.wait();
}

fn clean_up_tools(
    tools: &mut [RunningTool],
    log: &mut SessionLog,
    force_close: bool,
) -> Result<(), AppError> {
    let mut failures = Vec::new();

    for tool in tools {
        match tool.child.try_wait() {
            Ok(Some(status)) => {
                log.best_effort_line(format!(
                    "{} already exited with status: {status}",
                    tool.name
                ));
            }
            Ok(None) if force_close || tool.close_when_game_exits => {
                log.best_effort_line(format!("Closing companion tool: {}", tool.name));
                if let Err(source) = tool.child.kill() {
                    match tool.child.try_wait() {
                        Ok(Some(status)) => {
                            log.best_effort_line(format!(
                                "{} exited during cleanup with status: {status}",
                                tool.name
                            ));
                            continue;
                        }
                        Ok(None) | Err(_) => {
                            let problem = format!("could not close {}: {source}", tool.name);
                            log.best_effort_line(&problem);
                            failures.push(problem);
                            continue;
                        }
                    }
                }
                match tool.child.wait() {
                    Ok(status) => {
                        log.best_effort_line(format!("{} closed with status: {status}", tool.name));
                    }
                    Err(source) => {
                        let problem =
                            format!("could not wait for {} to close: {source}", tool.name);
                        log.best_effort_line(&problem);
                        failures.push(problem);
                    }
                }
            }
            Ok(None) => {
                log.best_effort_line(format!(
                    "{} remains running because close_when_game_exits is false.",
                    tool.name
                ));
            }
            Err(source) => {
                let problem = format!("could not inspect companion tool {}: {source}", tool.name);
                log.best_effort_line(&problem);
                failures.push(problem);
            }
        }
    }

    if failures.is_empty() {
        Ok(())
    } else {
        Err(AppError::runtime(format!(
            "tool cleanup failed: {}",
            failures.join("; ")
        )))
    }
}

fn game_result(status: ExitStatus) -> Result<(), AppError> {
    if status.success() {
        Ok(())
    } else {
        Err(AppError::process_exit("game process failed", status.code()))
    }
}

fn print_program(program: &ResolvedProgram) {
    println!("  name: {}", program.name);
    println!("  path: {}", program.path.display());
    println!(
        "  working directory: {}",
        program.working_directory.display()
    );
    if !program.arguments.is_empty() {
        println!("  arguments: {:?}", program.arguments);
    }
}

fn launch_label(timing: LaunchTiming) -> &'static str {
    match timing {
        LaunchTiming::BeforeGame => "before-game",
        LaunchTiming::AfterGame => "after-game",
    }
}

fn wait_label(wait: BeforeGameWait) -> &'static str {
    match wait {
        BeforeGameWait::None => "none",
        BeforeGameWait::UserConfirmation => "user-confirmation",
        BeforeGameWait::ToolExit => "tool-exit",
    }
}

#[cfg(all(test, windows))]
mod windows_tests {
    use super::{cmd_compatible_path, windows_script_command_line};
    use std::path::Path;

    #[test]
    fn removes_verbatim_drive_prefix_for_cmd() {
        assert_eq!(
            cmd_compatible_path(Path::new(r"\\?\Z:\Games\Tandem Test\BeforeTool.cmd")),
            r"Z:\Games\Tandem Test\BeforeTool.cmd"
        );
    }

    #[test]
    fn converts_verbatim_unc_prefix_for_cmd() {
        assert_eq!(
            cmd_compatible_path(Path::new(
                r"\\?\UNC\server\share\Tandem Test\BeforeTool.cmd"
            )),
            r"\\server\share\Tandem Test\BeforeTool.cmd"
        );
    }

    #[test]
    fn builds_raw_cmd_call_with_quoted_script_path_and_arguments() {
        assert_eq!(
            windows_script_command_line(
                Path::new(r"\\?\Z:\Games\Tandem Test\BeforeTool.cmd"),
                &["--profile".into(), "Low latency".into()]
            ),
            r#"call "Z:\Games\Tandem Test\BeforeTool.cmd" "--profile" "Low latency""#
        );
    }
}
