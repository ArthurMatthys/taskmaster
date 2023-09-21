pub type Result<T> = std::result::Result<T, Error>;
use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    De(String),
    Read(String),

    NoFilenameProvided,
    TooManyArguments,
    ConfigFileNotFound(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::De(e) => write!(f, "Error reading file : {e}"),
            Error::Read(e) => write!(f, "Error reading file : {e}"),
            Error::NoFilenameProvided => write!(f, "No filename provided"),
            Error::TooManyArguments => write!(f, "Too many arguments"),
            Error::ConfigFileNotFound(e) => write!(f, "Config file not found : {e}"),
        }
    }
}
