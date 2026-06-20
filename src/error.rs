use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum AppError {
    Usage(String),
    Runtime(String),
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

    pub fn io(context: impl Into<String>, source: io::Error) -> Self {
        Self::Io {
            context: context.into(),
            source,
        }
    }

    pub fn exit_code(&self) -> u8 {
        match self {
            Self::Usage(_) => 2,
            Self::Runtime(_)
            | Self::Io { .. }
            | Self::ConfigParse { .. }
            | Self::InvalidConfig(_) => 1,
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) | Self::Runtime(message) => {
                write!(formatter, "{message}")
            }
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
            Self::Usage(_) | Self::Runtime(_) | Self::InvalidConfig(_) => None,
        }
    }
}
