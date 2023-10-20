mod controller;
mod model;

pub use controller::*;
pub use daemonize::{Error, Result};
// use logger::{log, LogInfo};
pub use daemonize::{Error, Result};
// use logger::{log, LogInfo};
pub use model::*;
use supervisor::Programs;
use supervisor::Programs;

mod server;
use server::server;

fn main() -> Result<()> {
    let config_file_path = match std::env::var("TASKMASTER_CONFIG_FILE_PATH") {
        Ok(path) => path,
        Err(e) => return Err(daemonize::Error::ConfigEnvVarNotFound(e)),
    };

    let initial_programs = Programs::default();
    match initial_programs.load_config(config_file_path.split_whitespace()) {
        Ok(programs) => server(programs),
        Err(e) => return Err(supervisor::Error::ConfigFileNotFound(e.to_string()).into()),
    }
}
