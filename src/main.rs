mod controller;
mod model;
mod supervisor;

use crate::model::daemon::Daemon;

pub const SOCKET: &str = "/tmp/taskmaster_socket";
fn main() -> std::io::Result<()> {
    // let args = Args::parse();
    // supervisor(args);
    let daemon = Daemon::create("/tmp/taskmaster.log".to_string());

    if daemon.is_err() {
        eprintln!("Error creating daemon : {daemon:#?}");
    }

    // eprintln!(
    //     "nb max fd : {:#?}",
    //     rlimit::getrlimit(rlimit::Resource::NOFILE)
    // );

    Ok(())
}
