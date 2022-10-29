type Errno = libc::c_int;
pub type Result<T> = std::result::Result<T, Error>;
use std::{fmt::Display, io};

#[derive(Debug)]
pub enum Error {
    AcceptClient(std::io::Error),
    ChangeDir(Errno),
    CloseFd(Errno),
    ConnectionShutdown(std::io::Error),
    CreateDir(std::io::Error),
    ClientErrorBinding(std::io::Error),
    ClientGetter,
    CommandFailed(std::io::Error),
    ConvertToUTF8,
    DeleteLock(Errno),
    DotEnv(dotenv::Error),
    DotEnvUsername(std::env::VarError),
    DotEnvPassword(std::env::VarError),
    DotEnvRelay(std::env::VarError),
    FileAlreadyLocked(Errno),
    Fork(Errno),
    GetPid(Errno),
    GetPgid(Errno),
    GetSid(Errno),
    InvalidFd { fd: i32, expected: i32 },
    IssueLockFile(Errno),
    Log(std::io::Error),
    LogOpen(std::io::Error),
    MailBuilder(lettre::error::Error),
    MailSend(lettre::transport::smtp::Error),
    MailSmtpTransport(lettre::transport::smtp::Error),
    MaxFdTooBig,
    NoArgumentProvided,
    Open(Errno),
    ParseError,
    ParseDstError,
    Quit,
    Read(Errno),
    ReadFile(std::io::Error),
    RedirectStream(Errno),
    Rlmit(Errno),
    SetSid(Errno),
    SetSig(Errno),
    ShellModeOverflow,
    SigMask(Errno),
    SignalSetting(Errno),
    Sysconf(Errno),
    Unlock(Errno),
    WriteToStream(std::io::Error),
    WrongFd,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::AcceptClient(e) => write!(f, "Error accepting a new client : {e}")?,
            Error::ChangeDir(e) => write!(f, "Error changing directory : {e}")?,
            Error::CloseFd(e) => write!(f, "Error closing fd : {e}")?,
            Error::ClientErrorBinding(e) => write!(f, "Cannot bind to address : {e}")?,
            Error::ClientGetter => write!(f, "Cannot retrieve client")?,
            Error::CommandFailed(e) => write!(f, "The command failed : {e}")?,
            Error::ConnectionShutdown(e) => write!(f, "Cannot shutdown the TCP connection : {e}")?,
            Error::CreateDir(e) => write!(f, "Error creating dir : {e}")?,
            Error::ConvertToUTF8 => write!(f, "Cannot convert the logfile name to UTF8")?,
            Error::DeleteLock(e) => write!(f, "Error deleting lock file: {e}")?,
            Error::DotEnv(e) => write!(f, "Error getting .env file : {e}")?,
            Error::DotEnvUsername(e) => write!(f, "SMTPUSERNAME not found in .env file : {e}")?,
            Error::DotEnvPassword(e) => write!(f, "SMTPUSERNAME not found in .env file : {e}")?,
            Error::DotEnvRelay(e) => write!(f, "SMTPUSERNAME not found in .env file : {e}")?,
            Error::InvalidFd { fd, expected } => {
                write!(f, "Opening fd {fd}, it should be {expected}")?
            }
            Error::FileAlreadyLocked(e) => {
                write!(f, "The lock file is locked by another process: {e}")?
            }
            Error::Fork(e) => write!(f, "Error forking : {e}")?,
            Error::GetPid(e) => write!(f, "Can't retrieve pid : {e}")?,
            Error::GetPgid(e) => write!(f, "Can't retrieve pid : {e}")?,
            Error::GetSid(e) => write!(f, "Can't retrieve pid : {e}")?,
            Error::IssueLockFile(e) => write!(f, "Issue with lock file : {e}")?,
            Error::Log(e) => write!(f, "Error while logging : {e}")?,
            Error::LogOpen(e) => write!(f, "Error trying to open logfile : {e}")?,
            Error::MaxFdTooBig => write!(f, "Max fd retrieved with sysconf is too big")?,
            Error::MailBuilder(e) => write!(f, "Cannot create the email : {e}")?,
            Error::MailSend(e) => write!(f, "Cannot send the mail : {e}")?,
            Error::MailSmtpTransport(e) => write!(f, "Cannot create the smtp transporter: {e}")?,
            Error::NoArgumentProvided => write!(f, "No argument provided while in shell mode")?,
            Error::Open(e) => write!(f, "Error opening file : {e}")?,
            Error::ParseError => write!(f, "Cannot parse email from string")?,
            Error::ParseDstError => write!(f, "Cannot parse recipients's mail from string")?,
            Error::Read(e) => write!(f, "Error reading file : {e}")?,
            Error::ReadFile(e) => write!(f, "Error reading file : {e}")?,
            Error::RedirectStream(e) => write!(f, "Error redirecting stream : {e}")?,
            Error::Rlmit(e) => write!(f, "Error getting rlimit : {e}")?,
            Error::SetSid(e) => write!(f, "Error setting sid : {e}")?,
            Error::SetSig(e) => write!(f, "Error getting signal set : {e}")?,
            Error::ShellModeOverflow => write!(f, "Error changing shell mode")?,
            Error::SigMask(e) => write!(f, "Error setting signal mask : {e}")?,
            Error::SignalSetting(e) => write!(f, "Error setting signla handler : {e}")?,
            Error::Sysconf(e) => write!(f, "Error getting value of sysconf : {e}")?,
            Error::Unlock(e) => write!(f, "Error unlocking lock file : {e}")?,
            Error::Quit => write!(f, "Quitting the daemon")?,
            Error::WriteToStream(e) => write!(f, "Error writing to stream : {e}")?,
            Error::WrongFd => write!(f, "No stream corresponding to this fd")?,
        };
        Ok(())
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

pub fn get_errno() -> Errno {
    io::Error::last_os_error()
        .raw_os_error()
        .expect("Errno expected")
}
