use std::{fs, process};

use crate::{
    error::{get_err, Error, Result},
    file_handler::unlock,
    logger::log,
    LogInfo,
};

/// Can't find it in libc, this value has been taken from nyx::sys::signal, but it's the same as in
/// signal.h
const NSIG: libc::c_int = 32;

pub fn handle_sig(value: i32) {
    if let Err(e) = fs::create_dir_all("/var/log/matt_daemon") {
        eprintln!("Cannot create dir to log signal input : {e}");
        return;
    }

    let info = LogInfo::Warn;
    let msg = format!("Received signal {value}. Exiting the daemon\n");
    if let Err(e) = log(msg, info) {
        eprintln!("Error writing logs : {:?}", e);
        return;
    }

    if let Err(e) = unlock("/var/lock/matt_daemon.lock".to_string()) {
        eprintln!("The lock file should be set to `/var/lock/matt_daemon.lock` : {e}");
        return;
    }

    process::exit(0);
}

pub fn set_sig_handlers() -> Result<()> {
    unsafe {
        for i in 1..NSIG {
            // Can't overwrite SIGKILL or SIGSTOP
            // SIGCHLD is up whenever I run a command in remote shell
            if i == libc::SIGKILL || i == libc::SIGSTOP || i == libc::SIGCHLD {
                continue;
            }
            get_err(libc::signal(i, handle_sig as _), Error::SignalSetting)?;
        }
    }
    Ok(())
}
