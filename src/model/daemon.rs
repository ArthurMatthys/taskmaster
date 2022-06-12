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
    fs::File,
    io::{self, stdin, Result, Write},
    os::unix::prelude::PermissionsExt,
    path::Path,
    process::Command,
};

fn get_info(name: &str) -> Result<String> {
    let pid = unistd::getpid();
    let sid = match unistd::getsid(Some(pid)) {
        Ok(sid) => sid,
        Err(e) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Can't get sid: {e}"),
            ))
        }
    };
    let guid = match unistd::getpgid(Some(pid)) {
        Ok(sid) => sid,
        Err(e) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Can't get pgid: {e}"),
            ))
        }
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
    /// Close all open file descriptors except standard input, output, and error (i.e. the first three file descriptors 0, 1, 2).
    /// This ensures that no accidentally passed file descriptor stays around in the daemon process.
    /// On Linux, this is best implemented by iterating through /proc/self/fd,
    /// with a fallback of iterating from file descriptor 3 to the value returned by getrlimit() for RLIMIT_NOFILE.
    fn close_fds(&self) -> Result<()> {
        let res = Command::new("ls").arg("/proc/self/fd/").output()?;
        let fds: Vec<i32> = match std::str::from_utf8(&res.stdout) {
            Ok(v) => v
                .trim()
                .split('\n')
                .map(|v| v.parse::<i32>())
                .filter_map(|v| v.ok())
                .collect(),
            Err(e) => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Can't convert from utf8 : {e}"),
                ))
            }
        };
        let fd_max: i32 = rlimit::getrlimit(rlimit::Resource::NOFILE)
            .map(|(soft, _)| soft.try_into().expect("Fd should not be bigger than i32"))?;
        let all_fds = 3i32..fd_max.max(3i32);
        // let filtered_fds = fds
        //     .iter()
        //     .filter(|fd| all_fds.contains(fd))
        //     .map(|fd| fd.clone());
        // fds.iter().for_each(|fd| {
        //     if all_fds.contains(fd) {
        //         all_fds.push(*fd)
        //     }
        // });
        // all_fds.append(&mut fds);
        // eprintln!("{all_fds:?}");

        // for fd in all_fds.chain(filtered_fds.clone()) {
        // TODO We got duplicates here, but it shouldn't matter
        for fd in all_fds.chain(fds) {
            if [0, 1, 2].contains(&fd) {
                continue;
            }
            match unistd::close(fd) {
                Ok(_) => self.log(
                    format!("fd {fd} closed for daemon creation"),
                    LogLevel::Info,
                )?,
                Err(nix::Error::Sys(nix::errno::Errno::EBADF)) => (),
                Err(e) => {
                    eprintln!("error with fd {fd} : {e}");
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Can't close fd {fd} : {e}"),
                    ));
                }
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
            Err(e) => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Issue clearing sig mask : {e}"),
                ));
            }
        }
        self.log("Signal mask reset", LogLevel::Info)?;
        Ok(())
    }

    fn reset_env(&self) -> Result<()> {
        unsafe {
            match nix::env::clearenv() {
                Ok(_) => (),
                Err(e) => {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Issue reseting environnement : {e}"),
                    ));
                }
            }
        }
        self.log("Environnement reseted", LogLevel::Info)?;
        Ok(())
    }

    fn connect_stream(&self) -> Result<()> {
        let mode_read = CString::new(b"r" as &[u8]).map_err(|err| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Issue getting raw pointer: {err}"),
            )
        })?;
        let mode_write = CString::new(b"w+" as &[u8]).map_err(|err| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Issue getting raw pointer: {err}"),
            )
        })?;
        let stdin = CString::new(b"stdin" as &[u8]).map_err(|err| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Issue getting raw pointer: {err}"),
            )
        })?;
        let stdout = CString::new(b"stdout" as &[u8]).map_err(|err| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Issue getting raw pointer: {err}"),
            )
        })?;
        let stderr = CString::new(b"stderr" as &[u8]).map_err(|err| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Issue getting raw pointer: {err}"),
            )
        })?;
        let new_std = CString::new(b"/dev/null" as &[u8]).map_err(|err| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Issue getting raw pointer: {err}"),
            )
        })?;
        clear_stream(stdin.as_ptr(), mode_read.as_ptr(), new_std.as_ptr());
        clear_stream(stdout.as_ptr(), mode_write.as_ptr(), new_std.as_ptr());
        clear_stream(stderr.as_ptr(), mode_write.as_ptr(), new_std.as_ptr());
        self.log("Stream connected to /dev/null", LogLevel::Info)?;
        println!("Mdr ? Yop");
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

        daemon.connect_stream()?;

        daemon.log(get_info("daemon")?, LogLevel::Info)?;

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
