type Errno = libc::c_int;
pub type Result<T> = std::result::Result<T, Error>;
use std::{fmt::Display, io};

#[derive(Debug)]
pub enum Error {
    ChangeDir(Errno),
    CloseFd(Errno),
    DeleteLock(Errno),
    Env(std::env::VarError),
    FileAlreadyLocked(Errno),
    Io(std::io::Error),
    MpscSend(std::sync::mpsc::SendError<i32>),
    Fork(Errno),
    GetPid(Errno),
    GetPgid(Errno),
    GetSid(Errno),
    InvalidFd { fd: i32, expected: i32 },
    IssueLockFile(Errno),
    MaxFdTooBig,
    Open(Errno),
    RedirectStream(Errno),
    Rlmit(Errno),
    SetSid(Errno),
    SetSig(Errno),
    SigMask(Errno),
    SignalSetting(Errno),
    Sysconf(Errno),
    Unlock(Errno),
    SupervisorError(String),
    ConfigEnvVarNotFound(std::env::VarError),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MpscSend(e) => Display::fmt(e, f),
            Error::ChangeDir(e) => write!(f, "Error changing directory : {e}"),
            Error::CloseFd(e) => write!(f, "Error closing fd : {e}"),
            Error::DeleteLock(e) => write!(f, "Error deleting lock file: {e}"),
            Error::Env(e) => Display::fmt(e, f),
            Error::InvalidFd { fd, expected } => {
                write!(f, "Opening fd {fd}, it should be {expected}")
            }
            Error::FileAlreadyLocked(e) => {
                write!(f, "The lock file is locked by another process: {e}")
            }
            Error::Fork(e) => write!(f, "Error forking : {e}"),
            Error::GetPid(e) => write!(f, "Can't retrieve pid : {e}"),
            Error::GetPgid(e) => write!(f, "Can't retrieve pid : {e}"),
            Error::GetSid(e) => write!(f, "Can't retrieve pid : {e}"),
            Error::Io(e) => Display::fmt(e, f),
            Error::IssueLockFile(e) => write!(f, "Issue with lock file : {e}"),
            Error::MaxFdTooBig => write!(f, "Max fd retrieved with sysconf is too big"),
            Error::Open(e) => write!(f, "Error opening file : {e}"),
            Error::RedirectStream(e) => write!(f, "Error redirecting stream : {e}"),
            Error::Rlmit(e) => write!(f, "Error getting rlimit : {e}"),
            Error::SetSid(e) => write!(f, "Error setting sid : {e}"),
            Error::SetSig(e) => write!(f, "Error getting signal set : {e}"),
            Error::SigMask(e) => write!(f, "Error setting signal mask : {e}"),
            Error::SignalSetting(e) => write!(f, "Error setting signal handler : {e}"),
            Error::Sysconf(e) => write!(f, "Error getting value of sysconf : {e}"),
            Error::Unlock(e) => write!(f, "Error unlocking lock file : {e}"),
            Error::SupervisorError(e) => write!(f, "Supervisor error : {e}"),
            Error::ConfigEnvVarNotFound(e) => write!(f, "Config env var not found : {e}"),
        }
    }
}

pub trait IsErr {
    fn is_err(&self) -> bool;
}
impl IsErr for i32 {
    fn is_err(&self) -> bool {
        *self == -1
    }
}
impl IsErr for i64 {
    fn is_err(&self) -> bool {
        *self == -1
    }
}
impl IsErr for isize {
    fn is_err(&self) -> bool {
        *self == -1
    }
}
impl IsErr for usize {
    fn is_err(&self) -> bool {
        *self == usize::MAX
    }
}

pub fn get_err<V, F>(value: V, f: F) -> Result<V>
where
    V: IsErr,
    F: FnOnce(Errno) -> Error,
{
    if value.is_err() {
        Err(f(get_errno()))
    } else {
        Ok(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<std::sync::mpsc::SendError<i32>> for Error {
    fn from(e: std::sync::mpsc::SendError<i32>) -> Self {
        Self::MpscSend(e)
    }
}
impl From<std::env::VarError> for Error {
    fn from(e: std::env::VarError) -> Self {
        Self::Env(e)
    }
}

pub fn get_errno() -> Errno {
    io::Error::last_os_error()
        .raw_os_error()
        .expect("Errno expected")
}

impl From<supervisor::Error> for Error {
    fn from(e: supervisor::Error) -> Self {
        Error::SupervisorError(e.to_string())
    }
}
