use daemonize::Result;
use std::{
    io::{stdin, stdout, BufRead, BufReader, BufWriter, Write},
    net::TcpStream,
};

const EXIT_STR: &str = "exit\n";

pub fn supervisor_ctl() -> Result<()> {
    let client = TcpStream::connect(std::env::var("SERVER_ADDRESS")?)?;

    let mut stream_writer = BufWriter::new(client.try_clone()?);
    let mut stream_reader = BufReader::new(client.try_clone()?);
    let mut stdin_reader = BufReader::new(stdin());
    let mut stdout_writer = BufWriter::new(stdout());
    let mut cmd = String::new();

    loop {
        let mut buf = vec![];
        stream_reader.read_until(b'>', &mut buf)?;
        let prompt = String::from_utf8(buf.clone()).unwrap();
        stdout_writer.write_all(format!("{}\x1B[0m", prompt).as_bytes())?;
        stdout_writer.flush()?;
        stdin_reader.read_line(&mut cmd)?;

        stream_writer.write_all(cmd.as_bytes())?;
        stream_writer.flush()?;
        if cmd == *EXIT_STR {
            eprintln!("Exiting");
            break;
        }
        cmd.clear();
    }
    Ok(())
}
