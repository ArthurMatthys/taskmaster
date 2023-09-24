use daemonize::Result;
use std::{
    io::{stdin, stdout, BufRead, BufReader, BufWriter, Write},
    net::TcpStream,
};

const EXIT_STR: &str = "exit\n";

pub fn supervisor_ctl() -> Result<()> {
    let mut client = TcpStream::connect(std::env::var("SERVER_ADDRESS")?)?;

    let mut stream_writer = BufWriter::new(client.try_clone()?);
    let mut stream_reader = BufReader::new(client.try_clone()?);
    let mut stdin_reader = BufReader::new(stdin());
    let mut stdout_writer = BufWriter::new(stdout());
    let mut buf = vec![];
    let mut cmd = String::new();

    loop {
        println!("Reading prompt");
        let read = stream_reader.read_until(b'>', &mut buf)?;
        let prompt = String::from_utf8(buf.clone()).unwrap();
        // stdout().write_all(format!("{}\x1B[0m", prompt).as_bytes())?;
        // stdout().flush()?;
        println!("writing prompt");
        stdout_writer.write_all(format!("{}\x1B[0m", prompt).as_bytes())?;
        stdout_writer.flush()?;
        // print!("{}\x1B[0m", prompt);
        println!("Reading stdin");
        stdin_reader.read_line(&mut cmd)?;

        println!("Sending cmd");
        stream_writer.write(cmd.as_bytes())?;
        stream_writer.flush()?;
        // stream_writer.write_all(cmd.as_bytes())?;
        println!("Done");
        if cmd == EXIT_STR.to_string() {
            eprintln!("Exiting");
            break;
        }
        cmd.clear();
        // writer.write_all(buf.as_bytes())?;
    }
    // loop {
    //     print!("Supervisor> ");
    //
    //     let mut input = String::new();
    //     stdin().read_line(&mut input).unwrap();
    //
    //     let mut parts = input.split_whitespace();
    //     let cmd = if let Some(cmd) = parts.next() {
    //         cmd.to_lowercase()
    //     } else {
    //         eprintln!("No command supplied");
    //         continue;
    //     };
    //
    //     let args = parts;
    //     match cmd.as_str() {
    //         "status" => programs.status(),
    //         "start" => programs.action("start", args),
    //         "stop" => programs.action("stop", args),
    //         "relaunch" => programs.action("relaunch", args),
    //         // "reload" => programs = load_config(args, &programs)?,
    //         "exit" | "quit" => break,
    //         _ => {
    //             eprintln!("Supervisor: Unknown command : {cmd}.");
    //             continue;
    //         }
    //     };
    // }
    Ok(())
}
