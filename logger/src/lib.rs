use std::{fmt::Display, fs, io::Write, path::PathBuf, sync::OnceLock};

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

static FILE: OnceLock<String> = OnceLock::new();

fn log_file() -> &'static String {
    FILE.get_or_init(|| std::env::var("TASKMASTER_LOGFILE").unwrap_or("taskmaster.log".to_string()))
}

pub fn log<S>(msg: S, info: LogInfo) -> std::io::Result<()>
where
    S: Display,
{
    let file = PathBuf::from(log_file());
    if !cfg!(debug_assertions) && info.is_debug() {
        return Ok(());
    }
    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent)?;
    }

    if let Some(filename) = file.file_name() {
        let mut f = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(filename)?;

        let now = Local::now().format("%d / %m / %Y - %H : %M : %S");
        f.write(format!("[{now:}] - {info:5} : {msg}").as_bytes())?;
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No filename found",
        ))
    }
}
