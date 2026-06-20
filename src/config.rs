use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::Deserialize;

use crate::error::AppError;

const SUPPORTED_CONFIG_VERSION: u32 = 1;
const MAX_TOOLS: usize = 32;
const MAX_DELAY_MS: u64 = 600_000;
const MAX_ARGUMENT_BYTES: usize = 16 * 1024;

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
    pub delay_ms: u64,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub close_when_game_exits: bool,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum LaunchTiming {
    BeforeGame,
    #[default]
    AfterGame,
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
    pub delay_ms: u64,
    pub required: bool,
    pub close_when_game_exits: bool,
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
            if is_windows_script(&program.path) && !program.arguments.is_empty() {
                problems.push(format!(
                    "{label} uses BAT/CMD arguments, which are intentionally unsupported in the first MVP"
                ));
            }

            if is_windows_script(&program.path) && contains_cmd_metacharacters(&program.path) {
                problems.push(format!(
                    "{label} path contains characters that are unsafe to pass through cmd.exe"
                ));
            }

            tools.push(ResolvedTool {
                program,
                launch: tool.launch,
                delay_ms: tool.delay_ms,
                required: tool.required,
                close_when_game_exits: tool.close_when_game_exits,
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

    if !problems.is_empty() {
        return Err(AppError::InvalidConfig(problems));
    }

    Ok(ResolvedConfig {
        config_path,
        log_file: log_file.expect("validated log path must exist"),
        continue_on_optional_tool_failure: config.launcher.continue_on_optional_tool_failure,
        game: game.expect("validated game must exist"),
        tools,
    })
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
        problems,
    )?;

    if path == current_executable() {
        problems.push(format!("{label} may not launch Tandem recursively"));
    }

    #[cfg(windows)]
    validate_windows_extension(label, &path, problems);

    let working_directory = match &program.working_directory {
        Some(directory) => resolve_existing_path(
            &format!("{label}.working_directory"),
            directory,
            config_directory,
            allow_external_paths,
            problems,
        )?,
        None => path
            .parent()
            .expect("resolved executable path must have a parent")
            .to_path_buf(),
    };

    if !working_directory.is_dir() {
        problems.push(format!(
            "{label} working directory is not a folder: {}",
            working_directory.display()
        ));
    }

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

    if !allow_external_paths {
        let canonical_root = match fs::canonicalize(config_directory) {
            Ok(root) => root,
            Err(source) => {
                problems.push(format!(
                    "configuration folder could not be resolved: {source}"
                ));
                return None;
            }
        };

        if !canonical.starts_with(&canonical_root) {
            problems.push(format!(
                "{label} resolves outside the portable Tandem folder"
            ));
            return None;
        }
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

    Some(if configured_path.is_absolute() {
        configured_path.to_path_buf()
    } else {
        config_directory.join(configured_path)
    })
}

fn is_external_syntax(path: &Path) -> bool {
    path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::Prefix(_)))
}

fn argument_bytes(arguments: &[String]) -> usize {
    arguments.iter().map(String::len).sum()
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

fn contains_cmd_metacharacters(path: &Path) -> bool {
    path.to_string_lossy().chars().any(|character| {
        matches!(
            character,
            '"' | '&' | '|' | '<' | '>' | '^' | '%' | '!' | '\r' | '\n'
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

#[cfg(test)]
mod tests {
    use super::{Config, LaunchTiming, is_external_syntax};
    use std::path::Path;

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
    fn tools_default_to_after_game() {
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
        assert!(config.tools[0].enabled);
    }

    #[test]
    fn parent_paths_are_external_syntax() {
        assert!(is_external_syntax(Path::new("../Tool.exe")));
        assert!(!is_external_syntax(Path::new("Tools/Tool.exe")));
    }
}
