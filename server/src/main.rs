mod controller;
mod model;

pub use controller::*;
use daemonize::Daemon;
pub use daemonize::{Error, Result};
pub use model::*;

mod server;
use server::server;

fn main() -> Result<()> {
    let daemon = Daemon::new(server)?;
    daemon.start()
}
