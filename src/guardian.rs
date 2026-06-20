use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use crate::error::AppError;
use crate::platform::ProcessWaiter;
use crate::protocol;

pub fn run(config_path: &Path) -> Result<(), AppError> {
    let executable = std::env::current_exe()
        .map_err(|source| AppError::io("could not locate the Tandem executable", source))?;

    let mut worker = Command::new(executable)
        .arg("--worker")
        .arg("--config")
        .arg(config_path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|source| AppError::io("could not start the Tandem worker", source))?;

    let worker_stdout = worker
        .stdout
        .take()
        .ok_or_else(|| AppError::runtime("Tandem worker stdout was not available"))?;

    let mut game_waiter = None;
    let reader = BufReader::new(worker_stdout);

    for line in reader.lines() {
        let line = line.map_err(|source| {
            AppError::io("could not read status from the Tandem worker", source)
        })?;

        if let Some(pid) = protocol::parse_game_pid(&line) {
            game_waiter = Some(ProcessWaiter::open(pid)?);
            continue;
        }

        println!("{line}");
        io::stdout()
            .flush()
            .map_err(|source| AppError::io("could not forward Tandem worker output", source))?;
    }

    let worker_status = worker
        .wait()
        .map_err(|source| AppError::io("could not wait for the Tandem worker", source))?;

    if worker_status.success() {
        return Ok(());
    }

    let Some(game_waiter) = game_waiter else {
        return Err(AppError::runtime(format!(
            "Tandem worker exited unexpectedly before reporting a game process: {worker_status}"
        )));
    };

    eprintln!(
        "Tandem worker exited unexpectedly with {worker_status}. \
The guardian will remain active until game process {} exits.",
        game_waiter.pid()
    );

    game_waiter.wait()?;

    eprintln!(
        "Game process {} has exited. The Tandem guardian may now close.",
        game_waiter.pid()
    );

    Ok(())
}
