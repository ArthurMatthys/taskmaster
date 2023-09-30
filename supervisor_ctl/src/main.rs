mod model;
mod supervisor_ctl;

use daemonize::{Error, Result};
use supervisor_ctl::supervisor_ctl;

fn main() -> Result<()> {
    if let Err(Error::Io(e)) = supervisor_ctl() {
        eprintln!("{}", e)
    }
    Ok(())
}
