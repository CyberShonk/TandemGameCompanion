use std::process::Child;
use std::thread;
use std::time::{Duration, Instant};

use crate::config::{ControlSelector, PreparationStep, WindowTitleMatcher};
use crate::error::AppError;
use crate::platform;

const WINDOW_POLL_INTERVAL: Duration = Duration::from_millis(50);

pub enum PreparationOutcome {
    WindowReady {
        title: String,
    },
    ControlReady {
        window_title: String,
        control_id: i32,
        class_name: String,
    },
}

impl PreparationOutcome {
    pub fn description(&self) -> String {
        match self {
            Self::WindowReady { title } => format!("matched window {title:?}"),
            Self::ControlReady {
                window_title,
                control_id,
                class_name,
            } => format!(
                "matched visible enabled control ID {control_id} with class {class_name:?} in window {window_title:?}"
            ),
        }
    }
}

pub fn execute(
    step: &PreparationStep,
    tool_name: &str,
    child: &mut Child,
) -> Result<PreparationOutcome, AppError> {
    match step {
        PreparationStep::WaitForWindow {
            matcher,
            timeout_ms,
        } => wait_for_window(tool_name, child, matcher, *timeout_ms),
        PreparationStep::WaitForControl {
            window_matcher,
            control_selector,
            timeout_ms,
        } => wait_for_control(
            tool_name,
            child,
            window_matcher,
            control_selector,
            *timeout_ms,
        ),
    }
}

fn wait_for_window(
    tool_name: &str,
    child: &mut Child,
    matcher: &WindowTitleMatcher,
    timeout_ms: u64,
) -> Result<PreparationOutcome, AppError> {
    let pid = child.id();
    wait_until_ready(
        tool_name,
        child,
        timeout_ms,
        "a matching window appeared",
        &format!(
            "waiting for companion tool {tool_name} window whose {}",
            matcher.description()
        ),
        || {
            platform::find_top_level_window(pid, matcher)
                .map(|matched| matched.map(|title| PreparationOutcome::WindowReady { title }))
        },
    )
}

fn wait_for_control(
    tool_name: &str,
    child: &mut Child,
    window_matcher: &WindowTitleMatcher,
    control_selector: &ControlSelector,
    timeout_ms: u64,
) -> Result<PreparationOutcome, AppError> {
    let pid = child.id();
    wait_until_ready(
        tool_name,
        child,
        timeout_ms,
        "a matching visible enabled control appeared",
        &format!(
            "waiting for companion tool {tool_name} visible enabled control in window whose {} and whose {}",
            window_matcher.description(),
            control_selector.description()
        ),
        || {
            platform::find_descendant_control(pid, window_matcher, control_selector).map(
                |matched| {
                    matched.map(|matched| PreparationOutcome::ControlReady {
                        window_title: matched.window_title,
                        control_id: matched.control_id,
                        class_name: matched.class_name,
                    })
                },
            )
        },
    )
}

fn wait_until_ready(
    tool_name: &str,
    child: &mut Child,
    timeout_ms: u64,
    exit_condition: &str,
    timeout_context: &str,
    mut inspect: impl FnMut() -> Result<Option<PreparationOutcome>, AppError>,
) -> Result<PreparationOutcome, AppError> {
    let timeout = Duration::from_millis(timeout_ms);
    let started = Instant::now();

    loop {
        if let Some(outcome) = inspect()? {
            return Ok(outcome);
        }

        if let Some(status) = child.try_wait().map_err(|source| {
            AppError::io(
                format!("could not inspect companion tool {tool_name} during preparation"),
                source,
            )
        })? {
            return Err(AppError::process_exit(
                format!("companion tool {tool_name} exited before {exit_condition}"),
                status.code(),
            ));
        }

        let elapsed = started.elapsed();
        if elapsed >= timeout {
            return Err(AppError::runtime(format!(
                "timed out after {timeout_ms} ms {timeout_context}"
            )));
        }

        thread::sleep(WINDOW_POLL_INTERVAL.min(timeout - elapsed));
    }
}
