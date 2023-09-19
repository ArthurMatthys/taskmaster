use daemonize::{log, Daemon, LogInfo, Result};

mod connections;

mod server;
use server::server;

fn main() -> Result<()> {
    server()
    // let daemon = match Daemon::new(server) {
    //     Ok(d) => d,
    //     Err(e) => {
    //         eprintln!("{e}");
    //         log(format!("{e}\n"), LogInfo::Error)?;
    //         return Err(e);
    //     }
    // };

    // match daemon.start() {
    //     Ok(_) => Ok(()),
    //     Err(e) => {
    //         if let Err(e) = log(format!("Error : {e}\n"), LogInfo::Error) {
    //             eprintln!("Failed to log error in daemon : {e}");
    //         }
    //         Err(e)
    //     }
    // }
}
