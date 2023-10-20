mod controller;
mod model;

pub use controller::*;
pub use daemonize::{Error, Result};
// use logger::{log, LogInfo};
pub use model::*;
use supervisor::Programs;

mod server;
use server::server;

fn main() -> Result<()> {
    match Programs::new() {
        Ok(mut programs) => server(&mut programs),
        Err(e) => Err(supervisor::Error::ConfigFileNotFound(e.to_string()).into()),
    }
}
