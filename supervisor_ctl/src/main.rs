mod model;
mod supervisor_ctl;

use daemonize::Result;
use supervisor_ctl::supervisor_ctl;

fn main() -> Result<()> {
    if let Err(e) = supervisor_ctl() {
        eprintln!("{}", e)
    }
    Ok(())
}
