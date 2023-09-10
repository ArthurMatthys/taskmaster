use std::{fmt::Display, fs, io::Write, path::PathBuf};

use crate::error::{Error, Result};
use chrono::offset::Local;

const LOGFILE: &str = "/var/log/taskmaster/taskmaster.log";
const LOGDIR: &str = "/var/log/taskmaster/";

pub enum LogInfo {
    Debug,
    Error,
    Info,
    Warn,
}

impl Display for LogInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogInfo::Debug => write!(f, "\x1B[34mDEBUG\x1B[0m"),
            LogInfo::Error => write!(f, "\x1B[31mERROR\x1B[0m"),
            LogInfo::Info => write!(f, "\x1B[33mINFO\x1B[0m"),
            LogInfo::Warn => write!(f, "\x1B[35mWarn\x1B[0m"),
        }
    }
}

impl LogInfo {
    fn is_debug(&self) -> bool {
        matches!(self, LogInfo::Debug)
    }
}

pub fn log<S>(msg: S, info: LogInfo) -> Result<()>
where
    S: Display,
{
    if !cfg!(debug_assertions) && info.is_debug() {
        return Ok(());
    }
    fs::create_dir_all(LOGDIR).map_err(Error::CreateDir)?;
    let mut f = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(LOGFILE)
        .map_err(Error::LogOpen)?;

    let now = Local::now().format("%d / %m / %Y - %H : %M : %S");
    f.write(format!("[{now:}] - {info:5} : {msg}").as_bytes())
        .map_err(Error::Log)?;
    Ok(())
}
