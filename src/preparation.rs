use std::process::Child;
use std::thread;
use std::time::{Duration, Instant};

use crate::config::{ControlSelector, PreparationStep, WindowTitleMatcher};
use crate::error::AppError;
use crate::platform::{self, ComboBoxSelectionStatus};

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
    ComboBoxIndexSelected {
        window_title: String,
        control_id: i32,
        class_name: String,
        requested_index: u32,
        prior_index: Option<u32>,
        resulting_index: u32,
        notification_sent: bool,
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
            Self::ComboBoxIndexSelected {
                window_title,
                control_id,
                class_name,
                requested_index,
                prior_index,
                resulting_index,
                notification_sent,
            } => {
                let prior = prior_index
                    .map(|index| index.to_string())
                    .unwrap_or_else(|| "none".into());
                let notification = if *notification_sent {
                    "sent one WM_COMMAND/CBN_SELCHANGE notification"
                } else {
                    "no notification; requested index was already selected"
                };
                format!(
                    "selected standard Win32 ComboBox in window {window_title:?} with selector control ID {control_id} and runtime class {class_name:?}: requested index {requested_index}, prior index {prior}, resulting index {resulting_index}, {notification}"
                )
            }
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
        PreparationStep::SelectComboBoxIndex {
            window_matcher,
            control_selector,
            selected_index,
            timeout_ms,
        } => select_combo_box_index(
            tool_name,
            child,
            window_matcher,
            control_selector,
            *selected_index,
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

fn select_combo_box_index(
    tool_name: &str,
    child: &mut Child,
    window_matcher: &WindowTitleMatcher,
    control_selector: &ControlSelector,
    requested_index: u32,
    timeout_ms: u64,
) -> Result<PreparationOutcome, AppError> {
    let pid = child.id();
    let timeout = Duration::from_millis(timeout_ms);
    let started = Instant::now();
    let selector = format!(
        "window whose {} and descendant whose {}",
        window_matcher.description(),
        control_selector.description()
    );
    let mut last_pending_reason = "matching parent window has not appeared".to_owned();

    loop {
        detect_tool_exit(
            tool_name,
            child,
            "the requested ComboBox index was selected and verified",
        )?;

        let elapsed = started.elapsed();
        if elapsed >= timeout {
            return Err(AppError::runtime(format!(
                "timed out after {timeout_ms} ms selecting ComboBox index for companion tool {tool_name}: selector [{selector}], requested index {requested_index}, failure reason: {last_pending_reason}"
            )));
        }

        let remaining = timeout - elapsed;
        let message_timeout_ms =
            u32::try_from(remaining.as_millis().max(1).min(u128::from(u32::MAX)))
                .unwrap_or(u32::MAX);

        match platform::select_combo_box_index(
            pid,
            window_matcher,
            control_selector,
            requested_index,
            message_timeout_ms,
        )
        .map_err(|error| {
            AppError::runtime(format!(
                "could not select ComboBox index for companion tool {tool_name}: selector [{selector}], requested index {requested_index}, failure reason: {error}"
            ))
        })? {
            ComboBoxSelectionStatus::Pending { reason } => last_pending_reason = reason,
            ComboBoxSelectionStatus::Complete(result) => {
                return Ok(PreparationOutcome::ComboBoxIndexSelected {
                    window_title: result.window_title,
                    control_id: result.control_id,
                    class_name: result.class_name,
                    requested_index: result.requested_index,
                    prior_index: result.prior_index,
                    resulting_index: result.resulting_index,
                    notification_sent: result.notification_sent,
                });
            }
        }

        let elapsed = started.elapsed();
        if elapsed >= timeout {
            continue;
        }
        thread::sleep(WINDOW_POLL_INTERVAL.min(timeout - elapsed));
    }
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

        detect_tool_exit(tool_name, child, exit_condition)?;

        let elapsed = started.elapsed();
        if elapsed >= timeout {
            return Err(AppError::runtime(format!(
                "timed out after {timeout_ms} ms {timeout_context}"
            )));
        }

        thread::sleep(WINDOW_POLL_INTERVAL.min(timeout - elapsed));
    }
}

fn detect_tool_exit(
    tool_name: &str,
    child: &mut Child,
    exit_condition: &str,
) -> Result<(), AppError> {
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

    Ok(())
}
