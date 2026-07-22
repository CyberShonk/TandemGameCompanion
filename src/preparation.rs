use std::process::Child;
use std::thread;
use std::time::{Duration, Instant};

use crate::config::{PreparationStep, WindowTitleMatcher};
use crate::error::AppError;
use crate::platform;

const WINDOW_POLL_INTERVAL: Duration = Duration::from_millis(50);

pub enum PreparationOutcome {
    WindowReady { title: String },
}

impl PreparationOutcome {
    pub fn description(&self) -> String {
        match self {
            Self::WindowReady { title } => format!("matched window {title:?}"),
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
    }
}

fn wait_for_window(
    tool_name: &str,
    child: &mut Child,
    matcher: &WindowTitleMatcher,
    timeout_ms: u64,
) -> Result<PreparationOutcome, AppError> {
    let timeout = Duration::from_millis(timeout_ms);
    let started = Instant::now();

    loop {
        if let Some(title) = platform::find_top_level_window(child.id(), matcher)? {
            return Ok(PreparationOutcome::WindowReady { title });
        }

        if let Some(status) = child.try_wait().map_err(|source| {
            AppError::io(
                format!("could not inspect companion tool {tool_name} during preparation"),
                source,
            )
        })? {
            return Err(AppError::process_exit(
                format!("companion tool {tool_name} exited before a matching window appeared"),
                status.code(),
            ));
        }

        let elapsed = started.elapsed();
        if elapsed >= timeout {
            return Err(AppError::runtime(format!(
                "timed out after {timeout_ms} ms waiting for companion tool {tool_name} window whose {}",
                matcher.description()
            )));
        }

        thread::sleep(WINDOW_POLL_INTERVAL.min(timeout - elapsed));
    }
}
