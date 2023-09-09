use chrono;
use libc::{self, freopen};
use nix::{
    self,
    sys::signal,
    unistd::{self, fork, ForkResult},
};
use std::{
    ffi::CString,
    fmt::{Debug, Display},
    fs::{self, read_dir, File},
    io::{self, stderr, stdin, stdout, Result, Write},
    os::unix::prelude::{AsRawFd, PermissionsExt},
    path::Path,
    process::Command,
};

fn to_io_err<E>(msg: &str, err: E) -> io::Error
where
    E: Display,
{
    io::Error::new(io::ErrorKind::Other, format!("{msg}: {err}"))
}

fn get_info(name: &str) -> Result<String> {
    let pid = unistd::getpid();
    let sid = match unistd::getsid(Some(pid)) {
        Ok(sid) => sid,
        Err(e) => return Err(to_io_err("Can't get sid", e)),
    };
    let guid = match unistd::getpgid(Some(pid)) {
        Ok(sid) => sid,
        Err(e) => return Err(to_io_err("Can't get pgid", e)),
    };

    Ok(format!(
        "{name:20} || pid : {pid} || sid : {sid} || guid : {guid}"
    ))
}

fn clear_stream(filename: *const i8, mode: *const i8, file: *const i8) {
    unsafe {
        let _ = freopen(filename, mode, libc::fopen(file, mode));
    }
}

#[derive(Debug)]
pub struct Daemon {
    logfile: String,
}

pub enum LogLevel {
    Debug,
    Error,
    Info,
    Warn,
}

impl Daemon {
    fn close_fds(&self) -> Result<()> {
        let fds: Vec<i32> = match read_dir(Path::new("/proc/self/fd/")) {
            Ok(entries) => entries
                .filter_map(|file| file.ok())
                .filter_map(|file| {
                    file.file_name()
                        .into_string()
                        .ok()
                        .map(|f| f.parse::<i32>().ok())
                        .flatten()
                })
                .collect(),
            Err(e) => {
                let fd_max: i32 =
                    rlimit::getrlimit(rlimit::Resource::NOFILE).map(|(soft, _)| {
                        soft.try_into()
                            .expect(&*format!("Fd should not be bigger than i32 : {e}"))
                    })?;
                (3i32..fd_max.max(3i32)).collect()
            }
        };
        for fd in fds {
            if [0, 1, 2].contains(&fd) {
                continue;
            }
            match unistd::close(fd) {
                Ok(_) => self.log(
                    format!("fd {fd} closed for daemon creation"),
                    LogLevel::Info,
                )?,
                Err(nix::Error::Sys(nix::errno::Errno::EBADF)) => (),
                Err(e) => return Err(to_io_err(&*format!("Can't close fd {fd}"), e)),
            };
        }

        self.log("File descriptor closed", LogLevel::Info)?;
        Ok(())
    }
    fn clear_signals(&self) -> Result<()> {
        let mut sigset = signal::SigSet::all();
        sigset.clear();
        self.log("Signals cleared", LogLevel::Info)?;
        Ok(())
    }

    fn reset_signal_mask(&self) -> Result<()> {
        match signal::sigprocmask(
            signal::SigmaskHow::SIG_SETMASK,
            Some(&signal::SigSet::all()),
            None,
        ) {
            Ok(_) => (),
            Err(e) => return Err(to_io_err("Issue clearing sig mask", e)),
        }
        self.log("Signal mask reset", LogLevel::Info)?;
        Ok(())
    }

    fn reset_env(&self) -> Result<()> {
        unsafe {
            match nix::env::clearenv() {
                Ok(_) => (),
                Err(e) => return Err(to_io_err("Issue resetting environnement", e)),
            }
        }
        self.log("Environnement reseted", LogLevel::Info)?;
        Ok(())
    }

    fn connect_stream(&self) -> Result<()> {
        unsafe {
            let null_file = libc::open(b"/dev/null\0" as *const [u8; 10] as _, libc::O_RDWR);

            libc::dup2(0, null_file);
            libc::dup2(1, null_file);
            libc::dup2(2, null_file);

            libc::close(null_file);
            println!("This message means that stdout was not redirected to /dev/null");
            eprintln!("This message means that stderr was not redirected to /dev/null");
        }

        self.log("Stream connected to /dev/null", LogLevel::Info)?;
        Ok(())
    }
}

impl Daemon {
    pub fn create(logfile: String) -> std::io::Result<Daemon> {
        let daemon = Daemon {
            logfile: logfile.clone(),
        };
        let _ = File::options()
            .write(true)
            .create(true)
            .open(Path::new(&logfile))?;

        daemon.log("Logfile created", LogLevel::Info)?;
        daemon.log(get_info("parent")?, LogLevel::Info)?;

        // Create a daemon the SysV way (
        // http://0pointer.de/public/systemd-man/daemon.html#SysV%20Daemons )
        daemon.close_fds()?;

        // Clear signal handlers
        // Does this work ?
        daemon.clear_signals()?;

        // Reset the signal mask using sigprocmask
        daemon.reset_signal_mask()?;

        // Reset environnement
        daemon.reset_env()?;

        match fork() {
            Ok(ForkResult::Parent { child: _ }) => return Ok(daemon),
            Ok(ForkResult::Child) => (),
            Err(e) => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to fork : {e}"),
                ))
            }
        };

        match unistd::setsid() {
            Ok(_) => (),
            Err(e) => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to set sid : {e}"),
                ))
            }
        }

        match fork() {
            Ok(ForkResult::Parent { child: _ }) => std::process::exit(0),
            Ok(ForkResult::Child) => (),
            Err(e) => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to fork : {e}"),
                ))
            }
        };

        // TODO Not working
        daemon.connect_stream()?;

        daemon.log(get_info("daemon")?, LogLevel::Info)?;

        // In the daemon process, reset the umask to 0, so that the file modes passed to open(), mkdir() and suchlike directly control the access mode of the created files and directories.

        fs::create_dir("./bonjour/");

        Ok(daemon)
    }

    pub fn log<M>(&self, msg: M, level: LogLevel) -> std::io::Result<()>
    where
        M: Display,
    {
        let mut f = File::options()
            .write(true)
            .append(true)
            .open(Path::new(&self.logfile))?;

        let helper = match level {
            LogLevel::Debug => "\x1B[34mDEBUG\x1B[0m",
            LogLevel::Error => "\x1B[31mERROR\x1B[0m",
            LogLevel::Info => "\x1B[33mINFO\x1B[0m",
            LogLevel::Warn => "\x1B[35mWarn\x1B[0m",
        };
        let msg_timestamped = format!("{} {helper:5}: {msg}\n", chrono::offset::Local::now());
        f.write_all(msg_timestamped.as_bytes())?;
        Ok(())
    }

    pub fn kill(&self) -> std::io::Result<()> {
        Ok(())
    }
}
