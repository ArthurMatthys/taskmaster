pub type Result<T> = std::result::Result<T, Error>;
use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    De(String),
    Read(String),

    NoFilenameProvided,
    TooManyArguments,
    ConfigFileNotFound(String),
    IoError { message: String },
    WaitError(String),
    ConfigEnvVarNotFound(std::env::VarError),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::De(e) => write!(f, "Error reading file : {e}"),
            Error::Read(e) => write!(f, "Error reading file : {e}"),
            Error::NoFilenameProvided => write!(f, "No filename provided"),
            Error::TooManyArguments => write!(f, "Too many arguments"),
            Error::ConfigFileNotFound(e) => write!(f, "Config file not found : {e}"),
            Error::IoError { message } => write!(f, "IO Error : {message}"),
            Error::WaitError(e) => write!(f, "Error waiting for child status : {e}"),
            Error::ConfigEnvVarNotFound(e) => write!(f, "Config env var not found : {e}"),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        // Here you can convert the std::io::Error to your custom Error type
        // This is just an example, replace it with your actual conversion logic
        Error::IoError {
            message: error.to_string(),
        }
    }
}
