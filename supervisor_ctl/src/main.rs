mod model;
mod supervisor_ctl;

use daemonize::Result;
use std::io::{BufReader, BufWriter};
use std::net::TcpStream;
use supervisor_ctl::supervisor_ctl;
struct ClientContext {
    pub(crate) writer: BufWriter<TcpStream>,
    pub(crate) reader: BufReader<TcpStream>,
}

fn main() -> Result<()> {
    let addr = match std::env::var("SERVER_ADDRESS") {
        Ok(addr) => addr,
        Err(_) => {
            logger::log(
                "SERVER_ADDRESS environment variable is not set, using localhost:4242 default\n",
                logger::LogInfo::Error,
            )?;
            "127.0.0.1:4242".to_string()
        }
    };
    let client = TcpStream::connect(addr)?;

    let stream_writer = BufWriter::new(client.try_clone()?);
    let stream_reader = BufReader::new(client.try_clone()?);
    if let Err(e) = supervisor_ctl(ClientContext {
        writer: stream_writer,
        reader: stream_reader,
    }) {
        eprintln!("{}", e)
    }
    Ok(())
}
