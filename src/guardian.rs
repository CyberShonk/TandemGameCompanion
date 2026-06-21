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
    let mut protocol_error = None;
    let reader = BufReader::new(worker_stdout);

    for line in reader.lines() {
        let line = match line {
            Ok(line) => line,
            Err(source) => {
                protocol_error = Some(AppError::io(
                    "could not read status from the Tandem worker",
                    source,
                ));
                break;
            }
        };

        if let Some(pid) = protocol::parse_game_pid(&line) {
            if game_waiter.is_some() {
                protocol_error = Some(AppError::runtime(
                    "Tandem worker reported more than one game process",
                ));
                continue;
            }

            match ProcessWaiter::open(pid) {
                Ok(waiter) => game_waiter = Some(waiter),
                Err(error) => protocol_error = Some(error),
            }
            continue;
        }

        println!("{line}");
        if let Err(source) = io::stdout().flush() {
            protocol_error = Some(AppError::io(
                "could not forward Tandem worker output",
                source,
            ));
            break;
        }
    }

    let worker_status = worker
        .wait()
        .map_err(|source| AppError::io("could not wait for the Tandem worker", source))?;

    if worker_status.success() {
        return match protocol_error {
            Some(error) => Err(error),
            None => Ok(()),
        };
    }

    let worker_error =
        AppError::process_exit("Tandem worker exited unexpectedly", worker_status.code());

    if let Some(error) = protocol_error {
        eprintln!("Tandem guardian protocol error: {error}");
    }

    let Some(game_waiter) = game_waiter else {
        return Err(worker_error);
    };

    eprintln!(
        "Tandem worker exited unexpectedly with {worker_status}. \
The guardian will remain active until game process {} exits.",
        game_waiter.pid()
    );

    if let Err(error) = game_waiter.wait() {
        eprintln!(
            "Tandem guardian could not complete its game-process wait: {error}. \
The worker failure exit code will still be preserved."
        );
        return Err(worker_error);
    }

    eprintln!(
        "Game process {} has exited. The Tandem guardian may now close.",
        game_waiter.pid()
    );

    Err(worker_error)
}
