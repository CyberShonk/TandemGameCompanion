use std::process::Child;
use std::thread;
use std::time::{Duration, Instant};

use crate::config::{ControlSelector, PreparationStep, WindowTitleMatcher};
use crate::error::AppError;
use crate::platform::{
    self, ButtonInvocationStatus, CheckboxStateStatus, ComboBoxSelectionStatus, EditTextStatus,
};

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
    ButtonInvoked {
        window_title: String,
        control_id: i32,
        class_name: String,
        button_style: u32,
    },
    CheckboxStateSet {
        window_title: String,
        control_id: i32,
        class_name: String,
        button_style: u32,
        requested_checked: bool,
        prior_checked: bool,
        resulting_checked: bool,
        clicked: bool,
    },
    EditTextSet {
        window_title: String,
        control_id: i32,
        class_name: String,
        requested_utf16_units: usize,
        prior_utf16_units: usize,
        resulting_utf16_units: usize,
        changed: bool,
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
            Self::ButtonInvoked {
                window_title,
                control_id,
                class_name,
                button_style,
            } => format!(
                "invoked standard Win32 push button in window {window_title:?} with selector control ID {control_id}, runtime class {class_name:?}, and button type style 0x{button_style:04x} using one bounded BM_CLICK"
            ),
            Self::CheckboxStateSet {
                window_title,
                control_id,
                class_name,
                button_style,
                requested_checked,
                prior_checked,
                resulting_checked,
                clicked,
            } => {
                let mutation = if *clicked {
                    "sent one bounded BM_CLICK"
                } else {
                    "no click; requested state was already set"
                };
                format!(
                    "set standard Win32 auto-checkbox state in window {window_title:?} with selector control ID {control_id}, runtime class {class_name:?}, and button type style 0x{button_style:04x}: requested checked={requested_checked}, prior checked={prior_checked}, resulting checked={resulting_checked}, {mutation}"
                )
            }
            Self::EditTextSet {
                window_title,
                control_id,
                class_name,
                requested_utf16_units,
                prior_utf16_units,
                resulting_utf16_units,
                changed,
            } => {
                let mutation = if *changed {
                    "sent one bounded WM_SETTEXT and verified exact text"
                } else {
                    "no WM_SETTEXT; requested text was already set"
                };
                format!(
                    "set standard Win32 Edit text in window {window_title:?} with selector control ID {control_id} and runtime class {class_name:?}: requested UTF-16 units {requested_utf16_units}, prior UTF-16 units {prior_utf16_units}, resulting UTF-16 units {resulting_utf16_units}, {mutation}"
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
        PreparationStep::InvokeButton {
            window_matcher,
            control_selector,
            timeout_ms,
        } => invoke_button(
            tool_name,
            child,
            window_matcher,
            control_selector,
            *timeout_ms,
        ),
        PreparationStep::SetCheckboxState {
            window_matcher,
            control_selector,
            checked,
            timeout_ms,
        } => set_checkbox_state(
            tool_name,
            child,
            window_matcher,
            control_selector,
            *checked,
            *timeout_ms,
        ),
        PreparationStep::SetEditText {
            window_matcher,
            control_selector,
            text,
            timeout_ms,
        } => set_edit_text(
            tool_name,
            child,
            window_matcher,
            control_selector,
            text,
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

fn invoke_button(
    tool_name: &str,
    child: &mut Child,
    window_matcher: &WindowTitleMatcher,
    control_selector: &ControlSelector,
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
        detect_tool_exit(tool_name, child, "the requested button was invoked")?;
        let elapsed = started.elapsed();
        if elapsed >= timeout {
            return Err(AppError::runtime(format!(
                "timed out after {timeout_ms} ms invoking button for companion tool {tool_name}: selector [{selector}], failure reason: {last_pending_reason}"
            )));
        }
        let remaining = timeout - elapsed;
        let message_timeout_ms =
            u32::try_from(remaining.as_millis().max(1).min(u128::from(u32::MAX)))
                .unwrap_or(u32::MAX);
        match platform::invoke_button(
            pid,
            window_matcher,
            control_selector,
            message_timeout_ms,
        )
        .map_err(|error| {
            AppError::runtime(format!(
                "could not invoke button for companion tool {tool_name}: selector [{selector}], failure reason: {error}"
            ))
        })? {
            ButtonInvocationStatus::Pending { reason } => last_pending_reason = reason,
            ButtonInvocationStatus::Complete(result) => {
                return Ok(PreparationOutcome::ButtonInvoked {
                    window_title: result.window_title,
                    control_id: result.control_id,
                    class_name: result.class_name,
                    button_style: result.button_style,
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

fn set_checkbox_state(
    tool_name: &str,
    child: &mut Child,
    window_matcher: &WindowTitleMatcher,
    control_selector: &ControlSelector,
    requested_checked: bool,
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
            "the requested checkbox state was set and verified",
        )?;
        let elapsed = started.elapsed();
        if elapsed >= timeout {
            return Err(AppError::runtime(format!(
                "timed out after {timeout_ms} ms setting checkbox state for companion tool {tool_name}: selector [{selector}], requested checked={requested_checked}, failure reason: {last_pending_reason}"
            )));
        }
        let remaining = timeout - elapsed;
        let message_timeout_ms =
            u32::try_from(remaining.as_millis().max(1).min(u128::from(u32::MAX)))
                .unwrap_or(u32::MAX);
        match platform::set_checkbox_state(
            pid,
            window_matcher,
            control_selector,
            requested_checked,
            message_timeout_ms,
        )
        .map_err(|error| {
            AppError::runtime(format!(
                "could not set checkbox state for companion tool {tool_name}: selector [{selector}], requested checked={requested_checked}, failure reason: {error}"
            ))
        })? {
            CheckboxStateStatus::Pending { reason } => last_pending_reason = reason,
            CheckboxStateStatus::Complete(result) => {
                return Ok(PreparationOutcome::CheckboxStateSet {
                    window_title: result.window_title,
                    control_id: result.control_id,
                    class_name: result.class_name,
                    button_style: result.button_style,
                    requested_checked: result.requested_checked,
                    prior_checked: result.prior_checked,
                    resulting_checked: result.resulting_checked,
                    clicked: result.clicked,
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
fn set_edit_text(
    tool_name: &str,
    child: &mut Child,
    window_matcher: &WindowTitleMatcher,
    control_selector: &ControlSelector,
    requested_text: &str,
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
    let requested_utf16_units = requested_text.encode_utf16().count();
    let mut last_pending_reason = "matching parent window has not appeared".to_owned();
    loop {
        detect_tool_exit(
            tool_name,
            child,
            "the requested edit text was set and verified",
        )?;
        let elapsed = started.elapsed();
        if elapsed >= timeout {
            return Err(AppError::runtime(format!(
                "timed out after {timeout_ms} ms setting Edit text for companion tool {tool_name}: selector [{selector}], requested UTF-16 units {requested_utf16_units}, failure reason: {last_pending_reason}"
            )));
        }
        let remaining = timeout - elapsed;
        let message_timeout_ms =
            u32::try_from(remaining.as_millis().max(1).min(u128::from(u32::MAX)))
                .unwrap_or(u32::MAX);
        match platform::set_edit_text(
            pid,
            window_matcher,
            control_selector,
            requested_text,
            message_timeout_ms,
        )
        .map_err(|error| {
            AppError::runtime(format!(
                "could not set Edit text for companion tool {tool_name}: selector [{selector}], requested UTF-16 units {requested_utf16_units}, failure reason: {error}"
            ))
        })? {
            EditTextStatus::Pending { reason } => last_pending_reason = reason,
            EditTextStatus::Complete(result) => {
                return Ok(PreparationOutcome::EditTextSet {
                    window_title: result.window_title,
                    control_id: result.control_id,
                    class_name: result.class_name,
                    requested_utf16_units: result.requested_utf16_units,
                    prior_utf16_units: result.prior_utf16_units,
                    resulting_utf16_units: result.resulting_utf16_units,
                    changed: result.changed,
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
