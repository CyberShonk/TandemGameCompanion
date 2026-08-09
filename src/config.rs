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
const MAX_CONTROL_CLASS_CHARS: usize = 256;
const MAX_COMBO_BOX_INDEX: i64 = 1_000_000;
pub(crate) const MAX_EDIT_TEXT_UTF16_UNITS: usize = 4_096;
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
    WaitForControl {
        window_title_equals: Option<String>,
        window_title_contains: Option<String>,
        control_id: Option<u32>,
        control_class_equals: Option<String>,
        #[serde(default = "default_window_wait_timeout_ms")]
        timeout_ms: u64,
    },
    SelectComboBoxIndex {
        window_title_equals: Option<String>,
        window_title_contains: Option<String>,
        control_id: Option<u32>,
        control_class_equals: Option<String>,
        selected_index: Option<i64>,
        #[serde(default = "default_window_wait_timeout_ms")]
        timeout_ms: u64,
    },
    InvokeButton {
        window_title_equals: Option<String>,
        window_title_contains: Option<String>,
        control_id: Option<u32>,
        control_class_equals: Option<String>,
        #[serde(default = "default_window_wait_timeout_ms")]
        timeout_ms: u64,
    },
    SetCheckboxState {
        window_title_equals: Option<String>,
        window_title_contains: Option<String>,
        control_id: Option<u32>,
        control_class_equals: Option<String>,
        checked: Option<bool>,
        #[serde(default = "default_window_wait_timeout_ms")]
        timeout_ms: u64,
    },
    SetEditText {
        window_title_equals: Option<String>,
        window_title_contains: Option<String>,
        control_id: Option<u32>,
        control_class_equals: Option<String>,
        text: Option<String>,
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
    WaitForControl {
        window_matcher: WindowTitleMatcher,
        control_selector: ControlSelector,
        timeout_ms: u64,
    },
    SelectComboBoxIndex {
        window_matcher: WindowTitleMatcher,
        control_selector: ControlSelector,
        selected_index: u32,
        timeout_ms: u64,
    },
    InvokeButton {
        window_matcher: WindowTitleMatcher,
        control_selector: ControlSelector,
        timeout_ms: u64,
    },
    SetCheckboxState {
        window_matcher: WindowTitleMatcher,
        control_selector: ControlSelector,
        checked: bool,
        timeout_ms: u64,
    },
    SetEditText {
        window_matcher: WindowTitleMatcher,
        control_selector: ControlSelector,
        text: String,
        timeout_ms: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WindowTitleMatcher {
    Equals(String),
    Contains(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlSelector {
    pub id: Option<u32>,
    pub class_equals: Option<String>,
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

impl ControlSelector {
    #[cfg(any(windows, test))]
    pub fn matches(&self, id: i32, class_name: &str) -> bool {
        self.id
            .is_none_or(|expected| i32::try_from(expected) == Ok(id))
            && self
                .class_equals
                .as_ref()
                .is_none_or(|expected| class_name == expected)
    }

    pub fn description(&self) -> String {
        match (&self.id, &self.class_equals) {
            (Some(id), Some(class_name)) => {
                format!("control ID {id} and class equals {class_name:?}")
            }
            (Some(id), None) => format!("control ID {id}"),
            (None, Some(class_name)) => format!("control class equals {class_name:?}"),
            (None, None) => "no control selector".to_owned(),
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
            Self::WaitForControl {
                window_matcher,
                control_selector,
                timeout_ms,
            } => format!(
                "wait-for-control (window {}; {}; visible and enabled; timeout={}ms)",
                window_matcher.description(),
                control_selector.description(),
                timeout_ms
            ),
            Self::SelectComboBoxIndex {
                window_matcher,
                control_selector,
                selected_index,
                timeout_ms,
            } => format!(
                "select-combo-box-index (window {}; {}; runtime class equals \"ComboBox\"; selected index {}; visible and enabled; timeout={}ms)",
                window_matcher.description(),
                control_selector.description(),
                selected_index,
                timeout_ms
            ),
            Self::InvokeButton {
                window_matcher,
                control_selector,
                timeout_ms,
            } => format!(
                "invoke-button (window {}; {}; runtime class equals \"Button\"; standard push-button style; visible and enabled; timeout={}ms)",
                window_matcher.description(),
                control_selector.description(),
                timeout_ms
            ),
            Self::SetCheckboxState {
                window_matcher,
                control_selector,
                checked,
                timeout_ms,
            } => format!(
                "set-checkbox-state (window {}; {}; runtime class equals \"Button\"; BS_AUTOCHECKBOX; checked={}; visible and enabled; timeout={}ms)",
                window_matcher.description(),
                control_selector.description(),
                checked,
                timeout_ms
            ),
            Self::SetEditText {
                window_matcher,
                control_selector,
                text,
                timeout_ms,
            } => format!(
                "set-edit-text (window {}; {}; runtime class equals \"Edit\"; single-line editable control; text UTF-16 units={}; visible and enabled; timeout={}ms)",
                window_matcher.description(),
                control_selector.description(),
                text.encode_utf16().count(),
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
    let label = format!("tool {tool_name} preparation step {}", step_index + 1);

    match step {
        PreparationStepConfig::WaitForWindow {
            title_equals,
            title_contains,
            timeout_ms,
        } => {
            let timeout_valid = validate_preparation_timeout(&label, *timeout_ms, problems);
            let matcher = resolve_window_title_matcher(
                &label,
                "title_equals",
                title_equals,
                "title_contains",
                title_contains,
                problems,
            );

            if !timeout_valid {
                return None;
            }

            matcher.map(|matcher| PreparationStep::WaitForWindow {
                matcher,
                timeout_ms: *timeout_ms,
            })
        }
        PreparationStepConfig::WaitForControl {
            window_title_equals,
            window_title_contains,
            control_id,
            control_class_equals,
            timeout_ms,
        } => {
            let timeout_valid = validate_preparation_timeout(&label, *timeout_ms, problems);
            let window_matcher = resolve_window_title_matcher(
                &label,
                "window_title_equals",
                window_title_equals,
                "window_title_contains",
                window_title_contains,
                problems,
            );

            if control_id.is_none() && control_class_equals.is_none() {
                problems.push(format!(
                    "{label} must define control_id, control_class_equals, or both"
                ));
            }

            let control_id_valid = match control_id {
                Some(id) if !(1..=i32::MAX as u32).contains(id) => {
                    problems.push(format!(
                        "{label} control_id must be between 1 and {}",
                        i32::MAX
                    ));
                    false
                }
                _ => true,
            };

            let class_equals = control_class_equals
                .as_ref()
                .and_then(|value| validate_control_class(&label, value, problems));
            let control_class_valid = control_class_equals.is_none() || class_equals.is_some();

            if !timeout_valid
                || !control_id_valid
                || !control_class_valid
                || (control_id.is_none() && control_class_equals.is_none())
            {
                return None;
            }

            window_matcher.map(|window_matcher| PreparationStep::WaitForControl {
                window_matcher,
                control_selector: ControlSelector {
                    id: *control_id,
                    class_equals,
                },
                timeout_ms: *timeout_ms,
            })
        }
        PreparationStepConfig::SelectComboBoxIndex {
            window_title_equals,
            window_title_contains,
            control_id,
            control_class_equals,
            selected_index,
            timeout_ms,
        } => {
            let timeout_valid = validate_preparation_timeout(&label, *timeout_ms, problems);
            let window_matcher = resolve_window_title_matcher(
                &label,
                "window_title_equals",
                window_title_equals,
                "window_title_contains",
                window_title_contains,
                problems,
            );

            if control_id.is_none() {
                problems.push(format!(
                    "{label} must define control_id for deterministic ComboBox parent notification"
                ));
            }

            let control_id_valid = match control_id {
                Some(id) if !(1..=u16::MAX as u32).contains(id) => {
                    problems.push(format!(
                        "{label} control_id must be between 1 and {} for select-combo-box-index",
                        u16::MAX
                    ));
                    false
                }
                Some(_) => true,
                None => false,
            };

            let class_equals = control_class_equals
                .as_ref()
                .and_then(|value| validate_control_class(&label, value, problems));
            let control_class_valid = control_class_equals.is_none() || class_equals.is_some();
            let supported_class = match class_equals.as_deref() {
                Some("ComboBox") | None => true,
                Some(_) => {
                    problems.push(format!(
                        "{label} control_class_equals must be exactly \"ComboBox\" for select-combo-box-index"
                    ));
                    false
                }
            };

            let selected_index = match selected_index {
                Some(index) if (0..=MAX_COMBO_BOX_INDEX).contains(index) => Some(*index as u32),
                Some(_) => {
                    problems.push(format!(
                        "{label} selected_index must be between 0 and {MAX_COMBO_BOX_INDEX}"
                    ));
                    None
                }
                None => {
                    problems.push(format!("{label} must define selected_index"));
                    None
                }
            };

            if !timeout_valid
                || !control_id_valid
                || !control_class_valid
                || !supported_class
                || selected_index.is_none()
                || control_id.is_none()
            {
                return None;
            }

            match (window_matcher, selected_index) {
                (Some(window_matcher), Some(selected_index)) => {
                    Some(PreparationStep::SelectComboBoxIndex {
                        window_matcher,
                        control_selector: ControlSelector {
                            id: *control_id,
                            class_equals,
                        },
                        selected_index,
                        timeout_ms: *timeout_ms,
                    })
                }
                _ => None,
            }
        }
        PreparationStepConfig::InvokeButton {
            window_title_equals,
            window_title_contains,
            control_id,
            control_class_equals,
            timeout_ms,
        } => {
            let timeout_valid = validate_preparation_timeout(&label, *timeout_ms, problems);
            let window_matcher = resolve_window_title_matcher(
                &label,
                "window_title_equals",
                window_title_equals,
                "window_title_contains",
                window_title_contains,
                problems,
            );
            if control_id.is_none() {
                problems.push(format!(
                    "{label} must define control_id for deterministic button invocation"
                ));
            }
            let control_id_valid = match control_id {
                Some(id) if !(1..=i32::MAX as u32).contains(id) => {
                    problems.push(format!(
                        "{label} control_id must be between 1 and {} for invoke-button",
                        i32::MAX
                    ));
                    false
                }
                Some(_) => true,
                None => false,
            };
            let class_equals = control_class_equals
                .as_ref()
                .and_then(|value| validate_control_class(&label, value, problems));
            let control_class_valid = control_class_equals.is_none() || class_equals.is_some();
            let supported_class = match class_equals.as_deref() {
                Some("Button") | None => true,
                Some(_) => {
                    problems.push(format!(
                        "{label} control_class_equals must be exactly \"Button\" for invoke-button"
                    ));
                    false
                }
            };
            if !timeout_valid
                || !control_id_valid
                || !control_class_valid
                || !supported_class
                || control_id.is_none()
            {
                return None;
            }
            window_matcher.map(|window_matcher| PreparationStep::InvokeButton {
                window_matcher,
                control_selector: ControlSelector {
                    id: *control_id,
                    class_equals,
                },
                timeout_ms: *timeout_ms,
            })
        }
        PreparationStepConfig::SetCheckboxState {
            window_title_equals,
            window_title_contains,
            control_id,
            control_class_equals,
            checked,
            timeout_ms,
        } => {
            let timeout_valid = validate_preparation_timeout(&label, *timeout_ms, problems);
            let window_matcher = resolve_window_title_matcher(
                &label,
                "window_title_equals",
                window_title_equals,
                "window_title_contains",
                window_title_contains,
                problems,
            );
            if control_id.is_none() {
                problems.push(format!(
                    "{label} must define control_id for deterministic checkbox state preparation"
                ));
            }
            let control_id_valid = match control_id {
                Some(id) if !(1..=i32::MAX as u32).contains(id) => {
                    problems.push(format!(
                        "{label} control_id must be between 1 and {} for set-checkbox-state",
                        i32::MAX
                    ));
                    false
                }
                Some(_) => true,
                None => false,
            };
            let class_equals = control_class_equals
                .as_ref()
                .and_then(|value| validate_control_class(&label, value, problems));
            let control_class_valid = control_class_equals.is_none() || class_equals.is_some();
            let supported_class = match class_equals.as_deref() {
                Some("Button") | None => true,
                Some(_) => {
                    problems.push(format!(
                        "{label} control_class_equals must be exactly \"Button\" for set-checkbox-state"
                    ));
                    false
                }
            };
            let checked = match checked {
                Some(value) => Some(*value),
                None => {
                    problems.push(format!("{label} must define checked"));
                    None
                }
            };
            if !timeout_valid
                || !control_id_valid
                || !control_class_valid
                || !supported_class
                || control_id.is_none()
                || checked.is_none()
            {
                return None;
            }
            match (window_matcher, checked) {
                (Some(window_matcher), Some(checked)) => Some(PreparationStep::SetCheckboxState {
                    window_matcher,
                    control_selector: ControlSelector {
                        id: *control_id,
                        class_equals,
                    },
                    checked,
                    timeout_ms: *timeout_ms,
                }),
                _ => None,
            }
        }
        PreparationStepConfig::SetEditText {
            window_title_equals,
            window_title_contains,
            control_id,
            control_class_equals,
            text,
            timeout_ms,
        } => {
            let timeout_valid = validate_preparation_timeout(&label, *timeout_ms, problems);
            let window_matcher = resolve_window_title_matcher(
                &label,
                "window_title_equals",
                window_title_equals,
                "window_title_contains",
                window_title_contains,
                problems,
            );
            if control_id.is_none() {
                problems.push(format!(
                    "{label} must define control_id for deterministic edit text preparation"
                ));
            }
            let control_id_valid = match control_id {
                Some(id) if !(1..=i32::MAX as u32).contains(id) => {
                    problems.push(format!(
                        "{label} control_id must be between 1 and {} for set-edit-text",
                        i32::MAX
                    ));
                    false
                }
                Some(_) => true,
                None => false,
            };
            let class_equals = control_class_equals
                .as_ref()
                .and_then(|value| validate_control_class(&label, value, problems));
            let control_class_valid = control_class_equals.is_none() || class_equals.is_some();
            let supported_class = match class_equals.as_deref() {
                Some("Edit") | None => true,
                Some(_) => {
                    problems.push(format!(
                        "{label} control_class_equals must be exactly \"Edit\" for set-edit-text"
                    ));
                    false
                }
            };
            let text = match text {
                Some(value) => {
                    let utf16_units = value.encode_utf16().count();
                    if value.contains('\0') {
                        problems.push(format!(
                            "{label} text may not contain NUL for set-edit-text"
                        ));
                        None
                    } else if value.contains('\r') || value.contains('\n') {
                        problems.push(format!(
                            "{label} text may not contain CR or LF for single-line set-edit-text"
                        ));
                        None
                    } else if utf16_units > MAX_EDIT_TEXT_UTF16_UNITS {
                        problems.push(format!(
                            "{label} text exceeds the {MAX_EDIT_TEXT_UTF16_UNITS}-UTF-16-unit limit for set-edit-text"
                        ));
                        None
                    } else {
                        Some(value.clone())
                    }
                }
                None => {
                    problems.push(format!("{label} must define text for set-edit-text"));
                    None
                }
            };
            if !timeout_valid
                || !control_id_valid
                || !control_class_valid
                || !supported_class
                || control_id.is_none()
                || text.is_none()
            {
                return None;
            }
            match (window_matcher, text) {
                (Some(window_matcher), Some(text)) => Some(PreparationStep::SetEditText {
                    window_matcher,
                    control_selector: ControlSelector {
                        id: *control_id,
                        class_equals,
                    },
                    text,
                    timeout_ms: *timeout_ms,
                }),
                _ => None,
            }
        }
    }
}

fn validate_preparation_timeout(label: &str, timeout_ms: u64, problems: &mut Vec<String>) -> bool {
    if (1..=MAX_WINDOW_WAIT_MS).contains(&timeout_ms) {
        true
    } else {
        problems.push(format!(
            "{label} timeout_ms must be between 1 and {MAX_WINDOW_WAIT_MS}"
        ));
        false
    }
}

fn resolve_window_title_matcher(
    label: &str,
    equals_field: &str,
    title_equals: &Option<String>,
    contains_field: &str,
    title_contains: &Option<String>,
    problems: &mut Vec<String>,
) -> Option<WindowTitleMatcher> {
    match (title_equals, title_contains) {
        (Some(_), Some(_)) | (None, None) => {
            problems.push(format!(
                "{label} must define exactly one of {equals_field} or {contains_field}"
            ));
            None
        }
        (Some(value), None) => validate_window_title_matcher(
            label,
            equals_field,
            value,
            WindowTitleMatcher::Equals,
            problems,
        ),
        (None, Some(value)) => validate_window_title_matcher(
            label,
            contains_field,
            value,
            WindowTitleMatcher::Contains,
            problems,
        ),
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

fn validate_control_class(label: &str, value: &str, problems: &mut Vec<String>) -> Option<String> {
    if value.trim().is_empty() {
        problems.push(format!("{label} control_class_equals may not be empty"));
        return None;
    }

    if value.chars().any(char::is_control) {
        problems.push(format!(
            "{label} control_class_equals may not contain control characters"
        ));
        return None;
    }

    if value.chars().count() > MAX_CONTROL_CLASS_CHARS {
        problems.push(format!(
            "{label} control_class_equals exceeds the {MAX_CONTROL_CLASS_CHARS}-character limit"
        ));
        return None;
    }

    Some(value.to_owned())
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
        BeforeGameWait, Config, ControlSelector, LaunchTiming, MAX_EDIT_TEXT_UTF16_UNITS,
        PreparationStep, PreparationStepConfig, WindowTitleMatcher, contains_cmd_metacharacters,
        is_external_syntax, load_and_resolve, resolve_preparation_step,
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
    fn parses_wait_for_control_preparation() {
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
action = "wait-for-control"
window_title_contains = "Trainer"
control_class_equals = "ComboBox"
control_id = 1001
"#,
        )
        .expect("control preparation should parse");

        assert_eq!(config.tools[0].prepare.len(), 1);
        assert_eq!(
            config.tools[0].prepare[0],
            PreparationStepConfig::WaitForControl {
                window_title_equals: None,
                window_title_contains: Some("Trainer".into()),
                control_id: Some(1001),
                control_class_equals: Some("ComboBox".into()),
                timeout_ms: 10_000,
            }
        );
    }

    #[test]
    fn parses_select_combo_box_index_preparation() {
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
action = "select-combo-box-index"
window_title_contains = "Trainer"
control_class_equals = "ComboBox"
control_id = 1001
selected_index = 2
"#,
        )
        .expect("ComboBox selection preparation should parse");

        assert_eq!(config.tools[0].prepare.len(), 1);
        assert_eq!(
            config.tools[0].prepare[0],
            PreparationStepConfig::SelectComboBoxIndex {
                window_title_equals: None,
                window_title_contains: Some("Trainer".into()),
                control_id: Some(1001),
                control_class_equals: Some("ComboBox".into()),
                selected_index: Some(2),
                timeout_ms: 10_000,
            }
        );
    }

    #[test]
    fn parses_invoke_button_preparation() {
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
action = "invoke-button"
window_title_contains = "Trainer"
control_class_equals = "Button"
control_id = 1002
"#,
        )
        .expect("button invocation preparation should parse");
        assert_eq!(
            config.tools[0].prepare[0],
            PreparationStepConfig::InvokeButton {
                window_title_equals: None,
                window_title_contains: Some("Trainer".into()),
                control_id: Some(1002),
                control_class_equals: Some("Button".into()),
                timeout_ms: 10_000,
            }
        );
    }

    #[test]
    fn parses_set_checkbox_state_preparation() {
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
action = "set-checkbox-state"
window_title_contains = "Trainer"
control_class_equals = "Button"
control_id = 1003
checked = true
"#,
        )
        .expect("checkbox state preparation should parse");
        assert_eq!(
            config.tools[0].prepare[0],
            PreparationStepConfig::SetCheckboxState {
                window_title_equals: None,
                window_title_contains: Some("Trainer".into()),
                control_id: Some(1003),
                control_class_equals: Some("Button".into()),
                checked: Some(true),
                timeout_ms: 10_000,
            }
        );
    }
    #[test]
    fn parses_set_edit_text_preparation() {
        let config: Config = toml::from_str(
            r#"config_version = 1
[game]
name = "Game"
path = "Game"
[[tools]]
name = "Tool"
path = "Tool"
launch = "before-game"
[[tools.prepare]]
action = "set-edit-text"
window_title_contains = "Trainer"
control_id = 4001
control_class_equals = "Edit"
text = "60"
"#,
        )
        .expect("set-edit-text preparation should parse");

        assert_eq!(
            config.tools[0].prepare[0],
            PreparationStepConfig::SetEditText {
                window_title_equals: None,
                window_title_contains: Some("Trainer".into()),
                control_id: Some(4001),
                control_class_equals: Some("Edit".into()),
                text: Some("60".into()),
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
    fn control_selectors_use_and_semantics_when_both_are_present() {
        let both = ControlSelector {
            id: Some(1001),
            class_equals: Some("ComboBox".into()),
        };
        assert!(both.matches(1001, "ComboBox"));
        assert!(!both.matches(1002, "ComboBox"));
        assert!(!both.matches(1001, "Button"));

        let id_only = ControlSelector {
            id: Some(1001),
            class_equals: None,
        };
        assert!(id_only.matches(1001, "Button"));
        assert!(!id_only.matches(1002, "Button"));

        let class_only = ControlSelector {
            id: None,
            class_equals: Some("ComboBox".into()),
        };
        assert!(class_only.matches(1002, "ComboBox"));
        assert!(!class_only.matches(1002, "combobox"));
    }

    #[test]
    fn control_preparation_description_is_deterministic() {
        let step = PreparationStep::WaitForControl {
            window_matcher: WindowTitleMatcher::Contains("Trainer".into()),
            control_selector: ControlSelector {
                id: Some(1001),
                class_equals: Some("ComboBox".into()),
            },
            timeout_ms: 10_000,
        };

        assert_eq!(
            step.description(),
            "wait-for-control (window title contains \"Trainer\"; control ID 1001 and class equals \"ComboBox\"; visible and enabled; timeout=10000ms)"
        );
    }

    #[test]
    fn combo_box_selection_description_is_deterministic() {
        let step = PreparationStep::SelectComboBoxIndex {
            window_matcher: WindowTitleMatcher::Contains("Trainer".into()),
            control_selector: ControlSelector {
                id: Some(1001),
                class_equals: Some("ComboBox".into()),
            },
            selected_index: 2,
            timeout_ms: 10_000,
        };

        assert_eq!(
            step.description(),
            "select-combo-box-index (window title contains \"Trainer\"; control ID 1001 and class equals \"ComboBox\"; runtime class equals \"ComboBox\"; selected index 2; visible and enabled; timeout=10000ms)"
        );
    }

    #[test]
    fn button_invocation_description_is_deterministic() {
        let step = PreparationStep::InvokeButton {
            window_matcher: WindowTitleMatcher::Contains("Trainer".into()),
            control_selector: ControlSelector {
                id: Some(1002),
                class_equals: Some("Button".into()),
            },
            timeout_ms: 10_000,
        };
        assert_eq!(
            step.description(),
            "invoke-button (window title contains \"Trainer\"; control ID 1002 and class equals \"Button\"; runtime class equals \"Button\"; standard push-button style; visible and enabled; timeout=10000ms)"
        );
    }

    #[test]
    fn checkbox_state_description_is_deterministic() {
        let step = PreparationStep::SetCheckboxState {
            window_matcher: WindowTitleMatcher::Contains("Trainer".into()),
            control_selector: ControlSelector {
                id: Some(1003),
                class_equals: Some("Button".into()),
            },
            checked: true,
            timeout_ms: 10_000,
        };
        assert_eq!(
            step.description(),
            "set-checkbox-state (window title contains \"Trainer\"; control ID 1003 and class equals \"Button\"; runtime class equals \"Button\"; BS_AUTOCHECKBOX; checked=true; visible and enabled; timeout=10000ms)"
        );
    }
    #[test]
    fn edit_text_description_is_deterministic_and_redacts_content() {
        let step = PreparationStep::SetEditText {
            window_matcher: WindowTitleMatcher::Contains("Trainer".into()),
            control_selector: ControlSelector {
                id: Some(4001),
                class_equals: Some("Edit".into()),
            },
            text: "secret-value".into(),
            timeout_ms: 10_000,
        };
        let description = step.description();
        assert_eq!(
            description,
            "set-edit-text (window title contains \"Trainer\"; control ID 4001 and class equals \"Edit\"; runtime class equals \"Edit\"; single-line editable control; text UTF-16 units=12; visible and enabled; timeout=10000ms)"
        );
        assert!(!description.contains("secret-value"));
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
    fn rejects_control_preparation_on_after_game_tools() {
        let root = test_directory("after-game-control-preparation");
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
action = "wait-for-control"
window_title_contains = "Tool"
control_id = 1001
"#,
        )
        .expect("configuration should be written");

        let error =
            load_and_resolve(&config).expect_err("after-game control preparation must be rejected");
        assert!(
            error
                .to_string()
                .contains("preparation requires launch = \"before-game\"")
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_control_preparation_for_script_wrappers() {
        let root = test_directory("script-control-preparation");
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
action = "wait-for-control"
window_title_contains = "Tool"
control_class_equals = "ComboBox"
"#,
        )
        .expect("configuration should be written");

        let error =
            load_and_resolve(&config).expect_err("script control preparation must be rejected");
        assert!(
            error
                .to_string()
                .contains("preparation requires a directly launched EXE or COM file")
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_ambiguous_control_preparation() {
        let root = test_directory("ambiguous-control-preparation");
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
action = "wait-for-control"
window_title_equals = "Tool"
window_title_contains = "Tool"
"#,
        )
        .expect("configuration should be written");

        let error =
            load_and_resolve(&config).expect_err("ambiguous control preparation must be rejected");
        let message = error.to_string();
        assert!(
            message.contains(
                "must define exactly one of window_title_equals or window_title_contains"
            )
        );
        assert!(message.contains("must define control_id, control_class_equals, or both"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_invalid_or_unbounded_control_waits() {
        let root = test_directory("invalid-control-waits");
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
action = "wait-for-control"
window_title_contains = "Tool"
control_id = 1001
timeout_ms = 0
[[tools.prepare]]
action = "wait-for-control"
window_title_contains = "Tool"
control_class_equals = "ComboBox"
timeout_ms = 120001
[[tools.prepare]]
action = "wait-for-control"
window_title_contains = "Tool"
control_id = 2147483648
timeout_ms = 1000
"#,
        )
        .expect("configuration should be written");

        let error = load_and_resolve(&config).expect_err("invalid control waits must be rejected");
        let message = error.to_string();
        assert_eq!(
            message
                .matches("timeout_ms must be between 1 and 120000")
                .count(),
            2
        );
        assert!(message.contains("control_id must be between 1 and 2147483647"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_invalid_combo_box_selection_recipes() {
        let root = test_directory("invalid-combo-selection");
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
action = "select-combo-box-index"
window_title_equals = "Tool"
window_title_contains = "Tool"
control_class_equals = "ComboBox"
selected_index = 2
[[tools.prepare]]
action = "select-combo-box-index"
window_title_contains = "Tool"
control_id = 1001
control_class_equals = "Button"
selected_index = -1
[[tools.prepare]]
action = "select-combo-box-index"
window_title_contains = "Tool"
control_id = 65536
selected_index = 1000001
timeout_ms = 0
[[tools.prepare]]
action = "select-combo-box-index"
window_title_contains = "Tool"
control_id = 1001
"#,
        )
        .expect("configuration should be written");

        let error =
            load_and_resolve(&config).expect_err("invalid ComboBox recipes must be rejected");
        let message = error.to_string();
        assert!(
            message.contains(
                "must define exactly one of window_title_equals or window_title_contains"
            )
        );
        assert!(
            message
                .contains("must define control_id for deterministic ComboBox parent notification")
        );
        assert!(message.contains(
            "control_class_equals must be exactly \"ComboBox\" for select-combo-box-index"
        ));
        assert_eq!(
            message
                .matches("selected_index must be between 0 and 1000000")
                .count(),
            2
        );
        assert!(
            message.contains("control_id must be between 1 and 65535 for select-combo-box-index")
        );
        assert!(message.contains("timeout_ms must be between 1 and 120000"));
        assert!(message.contains("must define selected_index"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_combo_box_selection_on_after_game_tools_and_script_wrappers() {
        let root = test_directory("combo-selection-boundaries");
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
launch = "after-game"
[[tools.prepare]]
action = "select-combo-box-index"
window_title_contains = "Tool"
control_id = 1001
control_class_equals = "ComboBox"
selected_index = 2
"#,
        )
        .expect("configuration should be written");

        let error = load_and_resolve(&config)
            .expect_err("after-game script ComboBox preparation must be rejected");
        let message = error.to_string();
        assert!(message.contains("preparation requires launch = \"before-game\""));
        assert!(message.contains("preparation requires a directly launched EXE or COM file"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_nul_in_edit_text_configuration() {
        let mut problems = Vec::new();
        let step = PreparationStepConfig::SetEditText {
            window_title_equals: Some("Tool".into()),
            window_title_contains: None,
            control_id: Some(4001),
            control_class_equals: Some("Edit".into()),
            text: Some("before\0after".into()),
            timeout_ms: 10_000,
        };
        let resolved = resolve_preparation_step("Tool", 0, &step, &mut problems);
        assert!(resolved.is_none());
        assert!(
            problems
                .iter()
                .any(|problem| problem.contains("text may not contain NUL for set-edit-text"))
        );
    }

    #[test]
    fn rejects_invalid_edit_text_recipes() {
        let root = test_directory("invalid-edit-text");
        fs::write(root.join("Game"), "game").expect("game should be written");
        fs::write(root.join("Tool"), "tool").expect("tool should be written");
        let config = root.join("Tandem.toml");
        let too_long = "x".repeat(MAX_EDIT_TEXT_UTF16_UNITS + 1);
        fs::write(
            &config,
            format!(
                r#"config_version = 1
[game]
name = "Game"
path = "Game"
[[tools]]
name = "Tool"
path = "Tool"
launch = "before-game"
[[tools.prepare]]
action = "set-edit-text"
window_title_equals = "Tool"
window_title_contains = "Tool"
control_class_equals = "Edit"
[[tools.prepare]]
action = "set-edit-text"
window_title_contains = "Tool"
control_id = 2147483648
control_class_equals = "Button"
text = "line\nfeed"
timeout_ms = 0
[[tools.prepare]]
action = "set-edit-text"
window_title_contains = "Tool"
control_id = 4001
text = {too_long:?}
"#
            ),
        )
        .expect("configuration should be written");
        let error =
            load_and_resolve(&config).expect_err("invalid edit text recipes must be rejected");
        let message = error.to_string();
        assert!(
            message.contains(
                "must define exactly one of window_title_equals or window_title_contains"
            )
        );
        assert!(message.contains("must define control_id for deterministic edit text preparation"));
        assert!(message.contains("control_id must be between 1 and 2147483647 for set-edit-text"));
        assert!(
            message.contains("control_class_equals must be exactly \"Edit\" for set-edit-text")
        );
        assert!(message.contains("must define text for set-edit-text"));
        assert!(message.contains("text may not contain CR or LF for single-line set-edit-text"));
        assert!(message.contains("text exceeds the 4096-UTF-16-unit limit for set-edit-text"));
        assert!(message.contains("timeout_ms must be between 1 and 120000"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_invalid_checkbox_state_recipes() {
        let root = test_directory("invalid-checkbox-state");
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
action = "set-checkbox-state"
window_title_equals = "Tool"
window_title_contains = "Tool"
control_class_equals = "Button"
[[tools.prepare]]
action = "set-checkbox-state"
window_title_contains = "Tool"
control_id = 2147483648
control_class_equals = "ComboBox"
checked = true
timeout_ms = 0
"#,
        )
        .expect("configuration should be written");
        let error =
            load_and_resolve(&config).expect_err("invalid checkbox state recipes must be rejected");
        let message = error.to_string();
        assert!(
            message.contains(
                "must define exactly one of window_title_equals or window_title_contains"
            )
        );
        assert!(
            message.contains("must define control_id for deterministic checkbox state preparation")
        );
        assert!(
            message.contains("control_id must be between 1 and 2147483647 for set-checkbox-state")
        );
        assert!(
            message
                .contains("control_class_equals must be exactly \"Button\" for set-checkbox-state")
        );
        assert!(message.contains("must define checked"));
        assert!(message.contains("timeout_ms must be between 1 and 120000"));
        let _ = fs::remove_dir_all(root);
    }
    #[test]
    fn rejects_invalid_button_invocation_recipes() {
        let root = test_directory("invalid-button-invocation");
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
action = "invoke-button"
window_title_equals = "Tool"
window_title_contains = "Tool"
control_class_equals = "Button"
[[tools.prepare]]
action = "invoke-button"
window_title_contains = "Tool"
control_id = 2147483648
control_class_equals = "ComboBox"
timeout_ms = 0
"#,
        )
        .expect("configuration should be written");
        let error = load_and_resolve(&config)
            .expect_err("invalid button invocation recipes must be rejected");
        let message = error.to_string();
        assert!(
            message.contains(
                "must define exactly one of window_title_equals or window_title_contains"
            )
        );
        assert!(message.contains("must define control_id for deterministic button invocation"));
        assert!(message.contains("control_id must be between 1 and 2147483647 for invoke-button"));
        assert!(
            message.contains("control_class_equals must be exactly \"Button\" for invoke-button")
        );
        assert!(message.contains("timeout_ms must be between 1 and 120000"));
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
