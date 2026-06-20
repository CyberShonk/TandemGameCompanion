mod cli;
mod config;
mod error;
mod guardian;
mod launcher;
mod platform;
mod protocol;

use std::process::ExitCode;

use cli::Mode;
use error::AppError;

const PRODUCT_NAME: &str = "Tandem Game Companion";
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Tandem error: {error}");
            ExitCode::from(error.exit_code())
        }
    }
}

fn run() -> Result<(), AppError> {
    let options = cli::parse_args(std::env::args().skip(1))?;

    match options.mode {
        Mode::Help => {
            print_help();
            return Ok(());
        }
        Mode::Version => {
            println!("{PRODUCT_NAME} {VERSION}");
            return Ok(());
        }
        Mode::Launch => {
            return guardian::run(&options.config_path);
        }
        Mode::Worker | Mode::Validate | Mode::DryRun => {}
    }

    let config = config::load_and_resolve(&options.config_path)?;

    match options.mode {
        Mode::Worker => launcher::run_worker(&config),
        Mode::Validate => {
            println!("Configuration is valid: {}", config.config_path.display());
            Ok(())
        }
        Mode::DryRun => {
            launcher::print_plan(&config);
            Ok(())
        }
        Mode::Launch | Mode::Help | Mode::Version => {
            unreachable!("handled before loading configuration")
        }
    }
}

fn print_help() {
    println!(
        "{PRODUCT_NAME} {VERSION}

Usage:
  tandem-game-companion [OPTIONS]

Options:
  -c, --config PATH    Use a configuration file other than Tandem.toml
      --validate       Validate the configuration without launching anything
      --dry-run        Print the resolved launch plan without launching anything
  -h, --help           Show this help
  -V, --version        Show the application version"
    );
}
