use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum AppError {
    Usage(String),
    Runtime(String),
    ProcessExit {
        context: String,
        code: Option<i32>,
    },
    Io {
        context: String,
        source: io::Error,
    },
    ConfigParse {
        path: PathBuf,
        source: toml::de::Error,
    },
    InvalidConfig(Vec<String>),
}

impl AppError {
    pub fn usage(message: impl Into<String>) -> Self {
        Self::Usage(message.into())
    }

    pub fn runtime(message: impl Into<String>) -> Self {
        Self::Runtime(message.into())
    }

    pub fn process_exit(context: impl Into<String>, code: Option<i32>) -> Self {
        Self::ProcessExit {
            context: context.into(),
            code,
        }
    }

    pub fn io(context: impl Into<String>, source: io::Error) -> Self {
        Self::Io {
            context: context.into(),
            source,
        }
    }

    pub fn exit_code(&self) -> u8 {
        match self {
            Self::Usage(_) => 2,
            Self::ProcessExit {
                code: Some(code), ..
            } if (1..=u8::MAX as i32).contains(code) => *code as u8,
            Self::Runtime(_)
            | Self::ProcessExit { .. }
            | Self::Io { .. }
            | Self::ConfigParse { .. }
            | Self::InvalidConfig(_) => 1,
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) | Self::Runtime(message) => write!(formatter, "{message}"),
            Self::ProcessExit {
                context,
                code: Some(code),
            } => write!(formatter, "{context} with exit code {code}"),
            Self::ProcessExit {
                context,
                code: None,
            } => write!(formatter, "{context} without an exit code"),
            Self::Io { context, source } => write!(formatter, "{context}: {source}"),
            Self::ConfigParse { path, source } => {
                write!(
                    formatter,
                    "could not parse configuration {}: {source}",
                    path.display()
                )
            }
            Self::InvalidConfig(problems) => {
                writeln!(formatter, "configuration is invalid:")?;
                for problem in problems {
                    writeln!(formatter, "  - {problem}")?;
                }
                Ok(())
            }
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::ConfigParse { source, .. } => Some(source),
            Self::Usage(_)
            | Self::Runtime(_)
            | Self::ProcessExit { .. }
            | Self::InvalidConfig(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AppError;

    #[test]
    fn preserves_normal_process_exit_codes() {
        assert_eq!(AppError::process_exit("failed", Some(42)).exit_code(), 42);
    }

    #[test]
    fn normalizes_missing_or_out_of_range_process_exit_codes() {
        assert_eq!(AppError::process_exit("failed", None).exit_code(), 1);
        assert_eq!(AppError::process_exit("failed", Some(0)).exit_code(), 1);
        assert_eq!(AppError::process_exit("failed", Some(3010)).exit_code(), 1);
    }
}
