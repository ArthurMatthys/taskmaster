mod error;
mod file_handler;
mod fork;
mod signal;

use file_handler::{lock, redirect_stream, unlock};
use fork::{execute_fork, ForkResult};
use libc::exit;
use logger::{log, LogInfo};
use std::path;

pub use error::{get_err, get_errno, Error, Result};
use signal::set_sig_handlers;

use crate::file_handler::close_fds;

const LOCKFILE: &str = "/var/lock/taskmaster.lock";

struct Mask {
    inner: libc::mode_t,
}

impl From<u32> for Mask {
    fn from(mask: u32) -> Self {
        Mask {
            inner: mask as libc::mode_t,
        }
    }
}
fn get_pid() -> Result<String> {
    unsafe {
        let pid = get_err(libc::getpid(), Error::GetPid)?;
        Ok(format!("Starting with pid {pid}\n"))
    }
}

pub struct Daemon {
    lock_file: String,
    umask: Mask,
    func: fn() -> Result<()>,
}

impl Drop for Daemon {
    fn drop(&mut self) {
        if log("deleting lock file\n", LogInfo::Info).is_err() {
            eprintln!("Exiting daemon : Could not log the deletion of the lock file");
        }

        if unlock(self.lock_file.clone()).is_err() {
            eprintln!("Unable to delete lock file");
        }

        if log("Daemon quitted\n", LogInfo::Info).is_err() {
            eprintln!("Could not log the exit of the daemon");
        }
    }
}

impl Daemon {
    pub fn new(f: fn() -> Result<()>) -> Result<Daemon> {
        if path::Path::new(LOCKFILE).exists() {
            Err(Error::FileAlreadyLocked(0))
        } else {
            Ok(Daemon {
                lock_file: LOCKFILE.to_string(),
                umask: 0.into(),
                func: f,
            })
        }
    }

    pub fn umask(mut self, mask: u32) -> Self {
        self.umask = mask.into();
        self
    }

    pub fn start(self) -> Result<()> {
        unsafe {
            log("Entering daemon mode\n", LogInfo::Info)?;

            log(get_pid()?, LogInfo::Info)?;
            match execute_fork()? {
                ForkResult::Child => (),
                ForkResult::Parent(_) => exit(libc::EXIT_SUCCESS),
            }

            get_err(libc::setsid(), Error::SetSid)?;

            match execute_fork()? {
                ForkResult::Child => (),
                ForkResult::Parent(_) => exit(libc::EXIT_SUCCESS),
            }

            log("Creating lock file\n", LogInfo::Debug)?;
            lock(self.lock_file.clone())?;

            log("Changing file mode creation\n", LogInfo::Debug)?;
            libc::umask(self.umask.inner);

            log("Changing working directory\n", LogInfo::Debug)?;
            get_err(libc::chdir(b"/\0" as *const u8 as _), Error::ChangeDir)?;

            log("Closing all open files\n", LogInfo::Debug)?;
            close_fds()?;

            log("Seting signal handlers\n", LogInfo::Debug)?;
            set_sig_handlers()?;

            redirect_stream()?;
            log(
                "Redirecting standard streams to /dev/null\n",
                LogInfo::Debug,
            )?;

            log("Daemon started properly\n", LogInfo::Info)?;

            (self.func)()?;
        }
        Ok(())
    }
}
