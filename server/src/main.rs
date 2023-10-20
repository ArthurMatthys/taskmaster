mod controller;
mod model;

pub use controller::*;
use daemonize::Daemon;
pub use daemonize::{Error, Result};
use logger::LogInfo;
pub use model::*;

mod server;
use server::server;

fn main() -> Result<()> {
    let daemon = Daemon::new(server)?;
    match daemon.start() {
        Ok(_) => Ok(()),
        Err(e) => {
            if let Err(e) = logger::log(format!("Error : {e}\n"), LogInfo::Error) {
                eprintln!("Failed to log error in daemon : {e}");
            }
            Err(e)
        }
    }
}
