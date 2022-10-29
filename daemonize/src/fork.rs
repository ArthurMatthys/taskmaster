use crate::error::{get_err, Error, Result};

pub(crate) enum ForkResult {
    Child,
    Parent(libc::pid_t),
}

pub(crate) unsafe fn execute_fork() -> Result<ForkResult> {
    let pid = get_err(libc::fork(), Error::Fork)?;
    if pid == 0 {
        Ok(ForkResult::Child)
    } else {
        Ok(ForkResult::Parent(pid))
    }
}
