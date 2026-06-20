use std::path::PathBuf;

use crate::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Launch,
    Worker,
    Validate,
    DryRun,
    Help,
    Version,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CliOptions {
    pub config_path: PathBuf,
    pub mode: Mode,
}

pub fn parse_args<I>(args: I) -> Result<CliOptions, AppError>
where
    I: IntoIterator<Item = String>,
{
    let mut arguments = args.into_iter();
    let mut config_path = PathBuf::from("Tandem.toml");
    let mut mode = Mode::Launch;

    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "-h" | "--help" => select_mode(&mut mode, Mode::Help)?,
            "-V" | "--version" => select_mode(&mut mode, Mode::Version)?,
            "--worker" => select_mode(&mut mode, Mode::Worker)?,
            "--validate" => select_mode(&mut mode, Mode::Validate)?,
            "--dry-run" => select_mode(&mut mode, Mode::DryRun)?,
            "-c" | "--config" => {
                let value = arguments
                    .next()
                    .ok_or_else(|| AppError::usage("--config requires a file path"))?;
                config_path = PathBuf::from(value);
            }
            unknown => {
                return Err(AppError::usage(format!(
                    "unknown argument: {unknown}\nRun with --help to see the available options."
                )));
            }
        }
    }

    Ok(CliOptions { config_path, mode })
}

fn select_mode(current: &mut Mode, requested: Mode) -> Result<(), AppError> {
    if *current != Mode::Launch {
        return Err(AppError::usage(
            "only one of --help, --version, --validate, or --dry-run may be used",
        ));
    }

    *current = requested;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{CliOptions, Mode, parse_args};
    use std::path::PathBuf;

    #[test]
    fn defaults_to_launching_tandem_toml() {
        let options = parse_args(Vec::<String>::new()).expect("default arguments should parse");

        assert_eq!(
            options,
            CliOptions {
                config_path: PathBuf::from("Tandem.toml"),
                mode: Mode::Launch,
            }
        );
    }

    #[test]
    fn accepts_custom_config_and_dry_run() {
        let options = parse_args([
            "--config".to_owned(),
            "Custom.toml".to_owned(),
            "--dry-run".to_owned(),
        ])
        .expect("valid arguments should parse");

        assert_eq!(options.config_path, PathBuf::from("Custom.toml"));
        assert_eq!(options.mode, Mode::DryRun);
    }

    #[test]
    fn accepts_internal_worker_mode() {
        let options = parse_args(["--worker".to_owned()]).expect("worker mode should parse");

        assert_eq!(options.mode, Mode::Worker);
    }

    #[test]
    fn rejects_unknown_arguments() {
        let error =
            parse_args(["--potato".to_owned()]).expect_err("unknown argument should be rejected");

        assert!(error.to_string().contains("unknown argument"));
    }
}
