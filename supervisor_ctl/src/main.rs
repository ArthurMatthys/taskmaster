mod model;
mod supervisor_ctl;

use daemonize::{Error, Result};
use supervisor_ctl::supervisor_ctl;

fn main() -> Result<()> {
    match supervisor_ctl() {
        Err(Error::Io(e)) => eprintln!("{}", e),
        _ => (),
    };
    Ok(())
}
