use std::fs::{File, OpenOptions};
use std::io::Write;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

use crate::config::{
    LaunchTiming, ResolvedConfig, ResolvedProgram, ResolvedTool, is_windows_script,
};
use crate::error::AppError;
use crate::protocol;

struct SessionLog {
    file: Option<File>,
}

struct RunningTool {
    name: String,
    child: Child,
    close_when_game_exits: bool,
}

impl SessionLog {
    fn open(path: &std::path::Path) -> Self {
        let file = match OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)
        {
            Ok(file) => Some(file),
            Err(source) => {
                eprintln!(
                    "Warning: could not open log file {}: {source}",
                    path.display()
                );
                None
            }
        };

        Self { file }
    }

    fn line(&mut self, message: impl AsRef<str>) {
        let message = message.as_ref();
        println!("{message}");
        let _ = std::io::stdout().flush();

        if let Some(file) = &mut self.file {
            let _ = writeln!(file, "{message}");
            let _ = file.flush();
        }
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
                "  {} [{}; delay={}ms; required={}; close_with_game={}]",
                tool.program.name,
                launch_label(tool.launch),
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
    let mut log = SessionLog::open(&config.log_file);
    let mut running_tools = Vec::new();

    log.line("Tandem Game Companion");
    log.line(format!("Configuration: {}", config.config_path.display()));

    for tool in config
        .tools
        .iter()
        .filter(|tool| tool.launch == LaunchTiming::BeforeGame)
    {
        start_tool(
            tool,
            config.continue_on_optional_tool_failure,
            &mut running_tools,
            &mut log,
        )?;
    }

    log.line(format!("Starting game: {}", config.game.name));
    let mut game = spawn_program(&config.game)?;
    log.line(format!("Game process started with PID {}", game.id()));

    protocol::report_game_pid(game.id()).map_err(|source| {
        AppError::io(
            "could not report the game process to the Tandem guardian",
            source,
        )
    })?;

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
        start_tool(
            tool,
            config.continue_on_optional_tool_failure,
            &mut running_tools,
            &mut log,
        )?;
    }

    log.line("Tandem is supervising the game process.");
    let game_status = game
        .wait()
        .map_err(|source| AppError::io("could not wait for the game process", source))?;

    log.line(format!("Game exited with status: {game_status}"));
    clean_up_tools(&mut running_tools, &mut log);
    log.line("Tandem session finished.");

    Ok(())
}

fn start_tool(
    tool: &ResolvedTool,
    continue_on_optional_failure: bool,
    running_tools: &mut Vec<RunningTool>,
    log: &mut SessionLog,
) -> Result<(), AppError> {
    if tool.delay_ms > 0 {
        log.line(format!(
            "Waiting {} ms before starting {}.",
            tool.delay_ms, tool.program.name
        ));
        thread::sleep(Duration::from_millis(tool.delay_ms));
    }

    log.line(format!("Starting companion tool: {}", tool.program.name));

    match spawn_program(&tool.program) {
        Ok(child) => {
            log.line(format!(
                "{} started with PID {}.",
                tool.program.name,
                child.id()
            ));
            running_tools.push(RunningTool {
                name: tool.program.name.clone(),
                child,
                close_when_game_exits: tool.close_when_game_exits,
            });
            Ok(())
        }
        Err(error) if tool.required || !continue_on_optional_failure => Err(error),
        Err(error) => {
            log.line(format!(
                "Optional tool {} could not be started: {error}",
                tool.program.name
            ));
            Ok(())
        }
    }
}

fn spawn_program(program: &ResolvedProgram) -> Result<Child, AppError> {
    let mut command = build_command(program)?;

    command
        .current_dir(&program.working_directory)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

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
    command.raw_arg(windows_script_command_line(&program.path));
    Ok(command)
}

#[cfg(windows)]
fn windows_script_command_line(path: &std::path::Path) -> String {
    format!("call \"{}\"", cmd_compatible_path(path))
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

fn clean_up_tools(tools: &mut [RunningTool], log: &mut SessionLog) {
    for tool in tools {
        match tool.child.try_wait() {
            Ok(Some(status)) => {
                log.line(format!(
                    "{} already exited with status: {status}",
                    tool.name
                ));
            }
            Ok(None) if tool.close_when_game_exits => {
                log.line(format!("Closing companion tool: {}", tool.name));
                if let Err(source) = tool.child.kill() {
                    log.line(format!("Could not close {}: {source}", tool.name));
                    continue;
                }

                match tool.child.wait() {
                    Ok(status) => {
                        log.line(format!("{} closed with status: {status}", tool.name));
                    }
                    Err(source) => {
                        log.line(format!(
                            "Could not wait for {} to close: {source}",
                            tool.name
                        ));
                    }
                }
            }
            Ok(None) => {
                log.line(format!(
                    "{} remains running because close_when_game_exits is false.",
                    tool.name
                ));
            }
            Err(source) => {
                log.line(format!(
                    "Could not inspect companion tool {}: {source}",
                    tool.name
                ));
            }
        }
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
    fn builds_raw_cmd_call_with_quoted_script_path() {
        assert_eq!(
            windows_script_command_line(Path::new(r"\\?\Z:\Games\Tandem Test\BeforeTool.cmd")),
            r#"call "Z:\Games\Tandem Test\BeforeTool.cmd""#
        );
    }
}
