use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::Deserialize;

use crate::error::AppError;

const SUPPORTED_CONFIG_VERSION: u32 = 1;
const MAX_TOOLS: usize = 32;
const MAX_PREPARATION_STEPS: usize = 16;
const MAX_DELAY_MS: u64 = 600_000;
const MAX_WINDOW_WAIT_MS: u64 = 120_000;
const MAX_WINDOW_TITLE_CHARS: usize = 256;
const MAX_ARGUMENT_BYTES: usize = 16 * 1024;
const DEFAULT_WINDOW_WAIT_TIMEOUT_MS: u64 = 10_000;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub config_version: u32,
    #[serde(default)]
    pub launcher: LauncherConfig,
    pub game: ProgramConfig,
    #[serde(default)]
    pub tools: Vec<ToolConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct LauncherConfig {
    pub log_file: PathBuf,
    pub allow_external_paths: bool,
    pub continue_on_optional_tool_failure: bool,
}

impl Default for LauncherConfig {
    fn default() -> Self {
        Self {
            log_file: PathBuf::from("Tandem.log"),
            allow_external_paths: false,
            continue_on_optional_tool_failure: true,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ProgramConfig {
    pub name: String,
    pub path: PathBuf,
    #[serde(default)]
    pub arguments: Vec<String>,
    pub working_directory: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
pub struct ToolConfig {
    pub name: String,
    pub path: PathBuf,
    #[serde(default)]
    pub arguments: Vec<String>,
    pub working_directory: Option<PathBuf>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub launch: LaunchTiming,
    #[serde(default)]
    pub before_game_wait: BeforeGameWait,
    #[serde(default)]
    pub delay_ms: u64,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub close_when_game_exits: bool,
    #[serde(default)]
    pub prepare: Vec<PreparationStepConfig>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(tag = "action", rename_all = "kebab-case")]
pub enum PreparationStepConfig {
    WaitForWindow {
        title_equals: Option<String>,
        title_contains: Option<String>,
        #[serde(default = "default_window_wait_timeout_ms")]
        timeout_ms: u64,
    },
}

#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum LaunchTiming {
    BeforeGame,
    #[default]
    AfterGame,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum BeforeGameWait {
    #[default]
    None,
    UserConfirmation,
    ToolExit,
}

#[derive(Debug)]
pub struct ResolvedConfig {
    pub config_path: PathBuf,
    pub log_file: PathBuf,
    pub continue_on_optional_tool_failure: bool,
    pub game: ResolvedProgram,
    pub tools: Vec<ResolvedTool>,
}

#[derive(Debug)]
pub struct ResolvedProgram {
    pub name: String,
    pub path: PathBuf,
    pub arguments: Vec<String>,
    pub working_directory: PathBuf,
}

#[derive(Debug)]
pub struct ResolvedTool {
    pub program: ResolvedProgram,
    pub launch: LaunchTiming,
    pub before_game_wait: BeforeGameWait,
    pub delay_ms: u64,
    pub required: bool,
    pub close_when_game_exits: bool,
    pub prepare: Vec<PreparationStep>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreparationStep {
    WaitForWindow {
        matcher: WindowTitleMatcher,
        timeout_ms: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WindowTitleMatcher {
    Equals(String),
    Contains(String),
}

impl WindowTitleMatcher {
    #[cfg(any(windows, test))]
    pub fn matches(&self, title: &str) -> bool {
        match self {
            Self::Equals(expected) => title == expected,
            Self::Contains(fragment) => title.contains(fragment),
        }
    }

    pub fn description(&self) -> String {
        match self {
            Self::Equals(expected) => format!("title equals {expected:?}"),
            Self::Contains(fragment) => format!("title contains {fragment:?}"),
        }
    }
}

impl PreparationStep {
    pub fn description(&self) -> String {
        match self {
            Self::WaitForWindow {
                matcher,
                timeout_ms,
            } => format!(
                "wait-for-window ({}; timeout={}ms)",
                matcher.description(),
                timeout_ms
            ),
        }
    }
}

#[derive(Clone, Copy)]
enum ExistingPathKind {
    File,
    Directory,
}

pub fn load_and_resolve(path: &Path) -> Result<ResolvedConfig, AppError> {
    let config_path = fs::canonicalize(path)
        .map_err(|source| AppError::io(format!("could not locate {}", path.display()), source))?;

    let contents = fs::read_to_string(&config_path).map_err(|source| {
        AppError::io(
            format!("could not read configuration {}", config_path.display()),
            source,
        )
    })?;

    let config: Config = toml::from_str(&contents).map_err(|source| AppError::ConfigParse {
        path: config_path.clone(),
        source,
    })?;

    resolve_config(config_path, config)
}

fn resolve_config(config_path: PathBuf, config: Config) -> Result<ResolvedConfig, AppError> {
    let config_directory = config_path
        .parent()
        .ok_or_else(|| AppError::InvalidConfig(vec!["configuration has no parent folder".into()]))?
        .to_path_buf();

    let mut problems = Vec::new();

    if config.config_version != SUPPORTED_CONFIG_VERSION {
        problems.push(format!(
            "config_version must be {SUPPORTED_CONFIG_VERSION}, not {}",
            config.config_version
        ));
    }

    if config.tools.len() > MAX_TOOLS {
        problems.push(format!(
            "no more than {MAX_TOOLS} companion tools may be configured"
        ));
    }

    let game = resolve_program(
        "game",
        &config.game,
        &config_directory,
        config.launcher.allow_external_paths,
        &mut problems,
    );

    let mut tools = Vec::new();
    for (index, tool) in config.tools.iter().enumerate() {
        if !tool.enabled {
            continue;
        }

        if tool.delay_ms > MAX_DELAY_MS {
            problems.push(format!(
                "tool {} delay_ms exceeds the maximum of {MAX_DELAY_MS}",
                tool.name
            ));
        }

        if tool.before_game_wait != BeforeGameWait::None && tool.launch != LaunchTiming::BeforeGame
        {
            problems.push(format!(
                "tool {} before_game_wait requires launch = \"before-game\"",
                tool.name
            ));
        }

        if tool.prepare.len() > MAX_PREPARATION_STEPS {
            problems.push(format!(
                "tool {} may not define more than {MAX_PREPARATION_STEPS} preparation steps",
                tool.name
            ));
        }

        if !tool.prepare.is_empty() && tool.launch != LaunchTiming::BeforeGame {
            problems.push(format!(
                "tool {} preparation requires launch = \"before-game\"",
                tool.name
            ));
        }

        let prepare: Vec<PreparationStep> = tool
            .prepare
            .iter()
            .enumerate()
            .filter_map(|(step_index, step)| {
                resolve_preparation_step(&tool.name, step_index, step, &mut problems)
            })
            .collect();

        let label = format!("tool {} ({})", index + 1, tool.name);
        if let Some(program) = resolve_program(
            &label,
            &ProgramConfig {
                name: tool.name.clone(),
                path: tool.path.clone(),
                arguments: tool.arguments.clone(),
                working_directory: tool.working_directory.clone(),
            },
            &config_directory,
            config.launcher.allow_external_paths,
            &mut problems,
        ) {
            if !prepare.is_empty() && is_windows_script(&program.path) {
                problems.push(format!(
                    "tool {} preparation requires a directly launched EXE or COM file",
                    tool.name
                ));
            }

            tools.push(ResolvedTool {
                program,
                launch: tool.launch,
                before_game_wait: tool.before_game_wait,
                delay_ms: tool.delay_ms,
                required: tool.required,
                close_when_game_exits: tool.close_when_game_exits,
                prepare,
            });
        }
    }

    let log_file = resolve_output_path(
        "launcher.log_file",
        &config.launcher.log_file,
        &config_directory,
        config.launcher.allow_external_paths,
        &mut problems,
    );

    if let Some(log_file) = &log_file {
        if log_file == &config_path {
            problems.push("launcher.log_file may not overwrite the configuration file".into());
        }
        if game
            .as_ref()
            .is_some_and(|program| &program.path == log_file)
        {
            problems.push("launcher.log_file may not overwrite the game executable".into());
        }
        for tool in &tools {
            if &tool.program.path == log_file {
                problems.push(format!(
                    "launcher.log_file may not overwrite companion tool {}",
                    tool.program.name
                ));
            }
        }
    }

    if !problems.is_empty() {
        return Err(AppError::InvalidConfig(problems));
    }

    let Some(log_file) = log_file else {
        return Err(AppError::runtime(
            "validated configuration did not produce a log path",
        ));
    };
    let Some(game) = game else {
        return Err(AppError::runtime(
            "validated configuration did not produce a game program",
        ));
    };

    Ok(ResolvedConfig {
        config_path,
        log_file,
        continue_on_optional_tool_failure: config.launcher.continue_on_optional_tool_failure,
        game,
        tools,
    })
}

fn resolve_preparation_step(
    tool_name: &str,
    step_index: usize,
    step: &PreparationStepConfig,
    problems: &mut Vec<String>,
) -> Option<PreparationStep> {
    match step {
        PreparationStepConfig::WaitForWindow {
            title_equals,
            title_contains,
            timeout_ms,
        } => {
            let label = format!("tool {tool_name} preparation step {}", step_index + 1);

            if !(1..=MAX_WINDOW_WAIT_MS).contains(timeout_ms) {
                problems.push(format!(
                    "{label} timeout_ms must be between 1 and {MAX_WINDOW_WAIT_MS}"
                ));
            }

            let matcher = match (title_equals, title_contains) {
                (Some(_), Some(_)) => {
                    problems.push(format!(
                        "{label} must define exactly one of title_equals or title_contains"
                    ));
                    None
                }
                (None, None) => {
                    problems.push(format!(
                        "{label} must define exactly one of title_equals or title_contains"
                    ));
                    None
                }
                (Some(value), None) => validate_window_title_matcher(
                    &label,
                    "title_equals",
                    value,
                    WindowTitleMatcher::Equals,
                    problems,
                ),
                (None, Some(value)) => validate_window_title_matcher(
                    &label,
                    "title_contains",
                    value,
                    WindowTitleMatcher::Contains,
                    problems,
                ),
            };

            if !(1..=MAX_WINDOW_WAIT_MS).contains(timeout_ms) {
                return None;
            }

            matcher.map(|matcher| PreparationStep::WaitForWindow {
                matcher,
                timeout_ms: *timeout_ms,
            })
        }
    }
}

fn validate_window_title_matcher(
    label: &str,
    field: &str,
    value: &str,
    build: impl FnOnce(String) -> WindowTitleMatcher,
    problems: &mut Vec<String>,
) -> Option<WindowTitleMatcher> {
    if value.trim().is_empty() {
        problems.push(format!("{label} {field} may not be empty"));
        return None;
    }

    if value.chars().any(char::is_control) {
        problems.push(format!(
            "{label} {field} may not contain control characters"
        ));
        return None;
    }

    if value.chars().count() > MAX_WINDOW_TITLE_CHARS {
        problems.push(format!(
            "{label} {field} exceeds the {MAX_WINDOW_TITLE_CHARS}-character limit"
        ));
        return None;
    }

    Some(build(value.to_owned()))
}

fn resolve_program(
    label: &str,
    program: &ProgramConfig,
    config_directory: &Path,
    allow_external_paths: bool,
    problems: &mut Vec<String>,
) -> Option<ResolvedProgram> {
    if program.name.trim().is_empty() {
        problems.push(format!("{label} name may not be empty"));
    }

    if argument_bytes(&program.arguments) > MAX_ARGUMENT_BYTES {
        problems.push(format!(
            "{label} arguments exceed the {MAX_ARGUMENT_BYTES}-byte limit"
        ));
    }

    let path = resolve_existing_path(
        &format!("{label}.path"),
        &program.path,
        config_directory,
        allow_external_paths,
        ExistingPathKind::File,
        problems,
    )?;

    if path == current_executable() {
        problems.push(format!("{label} may not launch Tandem recursively"));
    }

    #[cfg(windows)]
    validate_windows_extension(label, &path, problems);

    validate_script_invocation(label, &path, &program.arguments, problems);

    let working_directory = match &program.working_directory {
        Some(directory) => resolve_existing_path(
            &format!("{label}.working_directory"),
            directory,
            config_directory,
            allow_external_paths,
            ExistingPathKind::Directory,
            problems,
        )?,
        None => match path.parent() {
            Some(parent) => parent.to_path_buf(),
            None => {
                problems.push(format!(
                    "{label} executable has no parent folder: {}",
                    path.display()
                ));
                return None;
            }
        },
    };

    Some(ResolvedProgram {
        name: program.name.clone(),
        path,
        arguments: program.arguments.clone(),
        working_directory,
    })
}

fn resolve_existing_path(
    label: &str,
    configured_path: &Path,
    config_directory: &Path,
    allow_external_paths: bool,
    kind: ExistingPathKind,
    problems: &mut Vec<String>,
) -> Option<PathBuf> {
    if configured_path.as_os_str().is_empty() {
        problems.push(format!("{label} may not be empty"));
        return None;
    }

    if !allow_external_paths && is_external_syntax(configured_path) {
        problems.push(format!(
            "{label} must remain inside the portable Tandem folder"
        ));
        return None;
    }

    let joined = if configured_path.is_absolute() {
        configured_path.to_path_buf()
    } else {
        config_directory.join(configured_path)
    };

    let canonical = match fs::canonicalize(&joined) {
        Ok(path) => path,
        Err(source) => {
            problems.push(format!(
                "{label} could not be resolved ({}): {source}",
                joined.display()
            ));
            return None;
        }
    };

    if !allow_external_paths
        && !path_is_inside_portable_root(&canonical, config_directory, problems)
    {
        problems.push(format!(
            "{label} resolves outside the portable Tandem folder"
        ));
        return None;
    }

    let correct_kind = match kind {
        ExistingPathKind::File => canonical.is_file(),
        ExistingPathKind::Directory => canonical.is_dir(),
    };
    if !correct_kind {
        let expected = match kind {
            ExistingPathKind::File => "a file",
            ExistingPathKind::Directory => "a folder",
        };
        problems.push(format!(
            "{label} is not {expected}: {}",
            canonical.display()
        ));
        return None;
    }

    Some(canonical)
}

fn resolve_output_path(
    label: &str,
    configured_path: &Path,
    config_directory: &Path,
    allow_external_paths: bool,
    problems: &mut Vec<String>,
) -> Option<PathBuf> {
    if configured_path.as_os_str().is_empty() {
        problems.push(format!("{label} may not be empty"));
        return None;
    }

    if !allow_external_paths && is_external_syntax(configured_path) {
        problems.push(format!(
            "{label} must remain inside the portable Tandem folder"
        ));
        return None;
    }

    let joined = if configured_path.is_absolute() {
        configured_path.to_path_buf()
    } else {
        config_directory.join(configured_path)
    };

    let Some(file_name) = joined.file_name() else {
        problems.push(format!("{label} must name a file"));
        return None;
    };
    let Some(parent) = joined.parent() else {
        problems.push(format!("{label} has no parent folder"));
        return None;
    };

    let canonical_parent = match fs::canonicalize(parent) {
        Ok(path) => path,
        Err(source) => {
            problems.push(format!(
                "{label} parent folder could not be resolved ({}): {source}",
                parent.display()
            ));
            return None;
        }
    };

    if !canonical_parent.is_dir() {
        problems.push(format!(
            "{label} parent path is not a folder: {}",
            canonical_parent.display()
        ));
        return None;
    }

    if !allow_external_paths
        && !path_is_inside_portable_root(&canonical_parent, config_directory, problems)
    {
        problems.push(format!(
            "{label} resolves outside the portable Tandem folder"
        ));
        return None;
    }

    let resolved = canonical_parent.join(file_name);
    if fs::symlink_metadata(&resolved).is_ok() {
        let canonical_target = match fs::canonicalize(&resolved) {
            Ok(path) => path,
            Err(source) => {
                problems.push(format!(
                    "{label} could not be resolved ({}): {source}",
                    resolved.display()
                ));
                return None;
            }
        };

        if canonical_target.is_dir() {
            problems.push(format!(
                "{label} points to a folder, not a file: {}",
                canonical_target.display()
            ));
            return None;
        }

        if !allow_external_paths
            && !path_is_inside_portable_root(&canonical_target, config_directory, problems)
        {
            problems.push(format!(
                "{label} resolves outside the portable Tandem folder"
            ));
            return None;
        }

        return Some(canonical_target);
    }

    Some(resolved)
}

fn path_is_inside_portable_root(
    path: &Path,
    config_directory: &Path,
    problems: &mut Vec<String>,
) -> bool {
    match fs::canonicalize(config_directory) {
        Ok(root) => path.starts_with(root),
        Err(source) => {
            problems.push(format!(
                "configuration folder could not be resolved: {source}"
            ));
            false
        }
    }
}

fn validate_script_invocation(
    label: &str,
    path: &Path,
    arguments: &[String],
    problems: &mut Vec<String>,
) {
    if !is_windows_script(path) {
        return;
    }

    if contains_cmd_metacharacters(&path.to_string_lossy()) {
        problems.push(format!(
            "{label} path contains characters that are unsafe to pass through cmd.exe"
        ));
    }

    for (index, argument) in arguments.iter().enumerate() {
        if contains_cmd_metacharacters(argument) {
            problems.push(format!(
                "{label} argument {} contains characters that are unsafe to pass through cmd.exe",
                index + 1
            ));
        }
    }
}

fn is_external_syntax(path: &Path) -> bool {
    path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::Prefix(_)))
}

fn argument_bytes(arguments: &[String]) -> usize {
    arguments.iter().map(|argument| argument.len()).sum()
}

fn current_executable() -> PathBuf {
    std::env::current_exe()
        .and_then(fs::canonicalize)
        .unwrap_or_default()
}

pub fn is_windows_script(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("bat") || extension.eq_ignore_ascii_case("cmd")
        })
}

fn contains_cmd_metacharacters(value: &str) -> bool {
    value.chars().any(|character| {
        matches!(
            character,
            '"' | '&' | '|' | '<' | '>' | '^' | '%' | '!' | '\r' | '\n' | '\0'
        )
    })
}

#[cfg(windows)]
fn validate_windows_extension(label: &str, path: &Path, problems: &mut Vec<String>) {
    let supported = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            ["exe", "com", "bat", "cmd"]
                .iter()
                .any(|supported| extension.eq_ignore_ascii_case(supported))
        });

    if !supported {
        problems.push(format!(
            "{label} must use an EXE, COM, BAT, or CMD file on Windows"
        ));
    }
}

fn default_true() -> bool {
    true
}

fn default_window_wait_timeout_ms() -> u64 {
    DEFAULT_WINDOW_WAIT_TIMEOUT_MS
}

#[cfg(test)]
mod tests {
    use super::{
        BeforeGameWait, Config, LaunchTiming, PreparationStepConfig, WindowTitleMatcher,
        contains_cmd_metacharacters, is_external_syntax, load_and_resolve,
    };
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_directory(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after the epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "tandem-config-{name}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("test directory should be created");
        path
    }

    #[test]
    fn parses_minimal_configuration() {
        let config: Config = toml::from_str(
            r#"
config_version = 1

[game]
name = "Demo Game"
path = "Game.exe"
"#,
        )
        .expect("minimal configuration should parse");

        assert_eq!(config.config_version, 1);
        assert!(config.tools.is_empty());
        assert!(!config.launcher.allow_external_paths);
    }

    #[test]
    fn tools_default_to_after_game_without_a_wait() {
        let config: Config = toml::from_str(
            r#"
config_version = 1

[game]
name = "Demo Game"
path = "Game.exe"

[[tools]]
name = "Trainer"
path = "Trainer.exe"
"#,
        )
        .expect("tool configuration should parse");

        assert_eq!(config.tools[0].launch, LaunchTiming::AfterGame);
        assert_eq!(config.tools[0].before_game_wait, BeforeGameWait::None);
        assert!(config.tools[0].enabled);
        assert!(config.tools[0].prepare.is_empty());
    }

    #[test]
    fn parses_before_game_wait_modes() {
        let config: Config = toml::from_str(
            r#"
config_version = 1

[game]
name = "Demo Game"
path = "Game.exe"

[[tools]]
name = "Trainer"
path = "Trainer.exe"
launch = "before-game"
before_game_wait = "user-confirmation"

[[tools]]
name = "Setup"
path = "Setup.exe"
launch = "before-game"
before_game_wait = "tool-exit"
"#,
        )
        .expect("wait modes should parse");

        assert_eq!(
            config.tools[0].before_game_wait,
            BeforeGameWait::UserConfirmation
        );
        assert_eq!(config.tools[1].before_game_wait, BeforeGameWait::ToolExit);
    }

    #[test]
    fn parses_wait_for_window_preparation() {
        let config: Config = toml::from_str(
            r#"
config_version = 1

[game]
name = "Demo Game"
path = "Game.exe"

[[tools]]
name = "Trainer"
path = "Trainer.exe"
launch = "before-game"

[[tools.prepare]]
action = "wait-for-window"
title_contains = "Trainer"
"#,
        )
        .expect("window preparation should parse");

        assert_eq!(config.tools[0].prepare.len(), 1);
        assert_eq!(
            config.tools[0].prepare[0],
            PreparationStepConfig::WaitForWindow {
                title_equals: None,
                title_contains: Some("Trainer".into()),
                timeout_ms: 10_000,
            }
        );
    }

    #[test]
    fn window_title_matchers_match_expected_titles() {
        assert!(WindowTitleMatcher::Equals("Trainer".into()).matches("Trainer"));
        assert!(!WindowTitleMatcher::Equals("Trainer".into()).matches("Trainer 1.0"));
        assert!(WindowTitleMatcher::Contains("Trainer".into()).matches("Universal Trainer 1.0"));
        assert!(!WindowTitleMatcher::Contains("trainer".into()).matches("Trainer"));
    }

    #[test]
    fn parent_paths_are_external_syntax() {
        assert!(is_external_syntax(Path::new("../Tool.exe")));
        assert!(!is_external_syntax(Path::new("Tools/Tool.exe")));
    }

    #[test]
    fn command_metacharacter_validation_allows_plain_arguments() {
        assert!(!contains_cmd_metacharacters("--profile"));
        assert!(!contains_cmd_metacharacters("Low latency mode"));
        assert!(contains_cmd_metacharacters("safe & whoami"));
        assert!(contains_cmd_metacharacters("%PATH%"));
    }

    #[test]
    fn rejects_a_directory_as_a_program() {
        let root = test_directory("directory-program");
        fs::create_dir(root.join("Game.exe")).expect("fake program directory should be created");
        let config = root.join("Tandem.toml");
        fs::write(
            &config,
            "config_version = 1\n[game]\nname = \"Game\"\npath = \"Game.exe\"\n",
        )
        .expect("configuration should be written");

        let error = load_and_resolve(&config).expect_err("directory must not validate as a file");
        assert!(error.to_string().contains("is not a file"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_wait_modes_on_after_game_tools() {
        let root = test_directory("after-game-wait");
        fs::write(root.join("Game"), "game").expect("game should be written");
        fs::write(root.join("Tool"), "tool").expect("tool should be written");
        let config = root.join("Tandem.toml");
        fs::write(
            &config,
            r#"config_version = 1
[game]
name = "Game"
path = "Game"
[[tools]]
name = "Tool"
path = "Tool"
launch = "after-game"
before_game_wait = "tool-exit"
"#,
        )
        .expect("configuration should be written");

        let error = load_and_resolve(&config).expect_err("invalid wait mode should be rejected");
        assert!(
            error
                .to_string()
                .contains("requires launch = \"before-game\"")
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_preparation_on_after_game_tools() {
        let root = test_directory("after-game-preparation");
        fs::write(root.join("Game"), "game").expect("game should be written");
        fs::write(root.join("Tool"), "tool").expect("tool should be written");
        let config = root.join("Tandem.toml");
        fs::write(
            &config,
            r#"config_version = 1
[game]
name = "Game"
path = "Game"
[[tools]]
name = "Tool"
path = "Tool"
launch = "after-game"
[[tools.prepare]]
action = "wait-for-window"
title_contains = "Tool"
"#,
        )
        .expect("configuration should be written");

        let error = load_and_resolve(&config).expect_err("after-game preparation must be rejected");
        assert!(
            error
                .to_string()
                .contains("preparation requires launch = \"before-game\"")
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_window_preparation_for_script_wrappers() {
        let root = test_directory("script-window-preparation");
        fs::write(root.join("Game"), "game").expect("game should be written");
        fs::write(root.join("Tool.cmd"), "tool").expect("tool should be written");
        let config = root.join("Tandem.toml");
        fs::write(
            &config,
            r#"config_version = 1
[game]
name = "Game"
path = "Game"
[[tools]]
name = "Tool"
path = "Tool.cmd"
launch = "before-game"
[[tools.prepare]]
action = "wait-for-window"
title_contains = "Tool"
"#,
        )
        .expect("configuration should be written");

        let error = load_and_resolve(&config).expect_err("script preparation must be rejected");
        assert!(
            error
                .to_string()
                .contains("preparation requires a directly launched EXE or COM file")
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_ambiguous_and_unbounded_window_preparation() {
        let root = test_directory("invalid-window-preparation");
        fs::write(root.join("Game"), "game").expect("game should be written");
        fs::write(root.join("Tool"), "tool").expect("tool should be written");
        let config = root.join("Tandem.toml");
        fs::write(
            &config,
            r#"config_version = 1
[game]
name = "Game"
path = "Game"
[[tools]]
name = "Tool"
path = "Tool"
launch = "before-game"
[[tools.prepare]]
action = "wait-for-window"
title_equals = "Tool"
title_contains = "Tool"
timeout_ms = 120001
"#,
        )
        .expect("configuration should be written");

        let error = load_and_resolve(&config).expect_err("invalid preparation must be rejected");
        let message = error.to_string();
        assert!(message.contains("exactly one of title_equals or title_contains"));
        assert!(message.contains("timeout_ms must be between 1 and 120000"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_a_log_path_that_would_overwrite_the_game() {
        let root = test_directory("log-overwrites-game");
        fs::write(root.join("Game"), "game").expect("game should be written");
        let config = root.join("Tandem.toml");
        fs::write(
            &config,
            "config_version = 1\n[launcher]\nlog_file = \"Game\"\n[game]\nname = \"Game\"\npath = \"Game\"\n",
        )
        .expect("configuration should be written");

        let error = load_and_resolve(&config).expect_err("log must not overwrite the game");
        assert!(
            error
                .to_string()
                .contains("may not overwrite the game executable")
        );
        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn rejects_a_dangling_log_symlink() {
        use std::os::unix::fs::symlink;

        let root = test_directory("dangling-log-symlink");
        fs::write(root.join("Game"), "game").expect("game should be written");
        symlink(root.join("missing-target"), root.join("Tandem.log"))
            .expect("dangling log symlink should be created");
        let config = root.join("Tandem.toml");
        fs::write(
            &config,
            "config_version = 1\n[game]\nname = \"Game\"\npath = \"Game\"\n",
        )
        .expect("configuration should be written");

        let error = load_and_resolve(&config).expect_err("dangling log link must be rejected");
        assert!(error.to_string().contains("could not be resolved"));
        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn rejects_a_log_file_symlink_that_escapes_the_portable_folder() {
        use std::os::unix::fs::symlink;

        let root = test_directory("log-symlink");
        let outside = test_directory("log-outside");
        fs::write(root.join("Game"), "game").expect("game should be written");
        symlink(&outside, root.join("logs")).expect("log symlink should be created");
        let config = root.join("Tandem.toml");
        fs::write(
            &config,
            "config_version = 1\n[launcher]\nlog_file = \"logs/Tandem.log\"\n[game]\nname = \"Game\"\npath = \"Game\"\n",
        )
        .expect("configuration should be written");

        let error = load_and_resolve(&config).expect_err("escaping log path should be rejected");
        assert!(
            error
                .to_string()
                .contains("resolves outside the portable Tandem folder")
        );
        let _ = fs::remove_dir_all(root);
        let _ = fs::remove_dir_all(outside);
    }
}
