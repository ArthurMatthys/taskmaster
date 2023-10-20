use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::TcpStream;
use std::process::ExitCode;

use reedline_repl_rs::clap::{Arg, ArgMatches, Command};
use reedline_repl_rs::{Error, Repl, Result};
use supervisor::Action;

const EXIT_STR: &str = "exit\n";

struct ClientContext {
    pub(crate) writer: BufWriter<TcpStream>,
    pub(crate) reader: BufReader<TcpStream>,
}

fn send_action(action: Action, ctx: &mut ClientContext) -> Result<Option<String>> {
    ctx.writer
        .write_all(format!("{}\n", action.to_string()).as_bytes())
        .unwrap();
    ctx.writer.flush().unwrap();
    if action == Action::Quit {
        std::process::exit(0)
    }

    let mut buf = String::new();
    ctx.reader.read_line(&mut buf).unwrap();

    // stdout_writer.write_all(format!("{}\x1B[0m", prompt).as_bytes())?;
    // stdout_writer.flush()?;
    // stdin_reader.read_line(&mut cmd)?;
    //
    // stream_writer.write_all(cmd.as_bytes())?;
    // stream_writer.flush()?

    Ok(Some(format!("action : {:?}", buf)))
}

fn quit(_args: ArgMatches, context: &mut ClientContext) -> Result<Option<String>> {
    send_action(Action::Quit, context)
}
fn reload(_args: ArgMatches, context: &mut ClientContext) -> Result<Option<String>> {
    send_action(Action::Reload, context)
}
fn status(_args: ArgMatches, context: &mut ClientContext) -> Result<Option<String>> {
    send_action(Action::Status, context)
}
fn restart(args: ArgMatches, context: &mut ClientContext) -> Result<Option<String>> {
    let programs = args
        .get_many::<String>("programs")
        .map(|v| v.cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    send_action(Action::Restart(programs), context)
}
fn start(args: ArgMatches, context: &mut ClientContext) -> Result<Option<String>> {
    let programs = args
        .get_many::<String>("programs")
        .map(|v| v.cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    send_action(Action::Start(programs), context)
}
fn stop(args: ArgMatches, context: &mut ClientContext) -> Result<Option<String>> {
    let programs = args
        .get_many::<String>("programs")
        .map(|v| v.cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    send_action(Action::Stop(programs), context)
}

pub fn supervisor_ctl() -> Result<()> {
    let client = TcpStream::connect(std::env::var("SERVER_ADDRESS").unwrap()).unwrap();

    let mut stream_writer = BufWriter::new(client.try_clone().unwrap());
    let mut stream_reader = BufReader::new(client.try_clone().unwrap());
    let mut repl = Repl::new(ClientContext {
        reader: stream_reader,
        writer: stream_writer,
    })
    .with_name("Supervisor_ctl")
    .with_version("v0.1.0")
    .with_description("Interactive shell to control supervisor")
    .with_command(
        Command::new("quit").about("Exit the REPL and supervisor"),
        quit,
    )
    .with_command(
        Command::new("reload").about("Reload the configuration file"),
        reload,
    )
    .with_command(
        Command::new("restart")
            .arg(Arg::new("programs").num_args(1..).required(true))
            .about("Restart the given list of programs"),
        restart,
    )
    .with_command(
        Command::new("status").about("Return the status of the programs handled by supervisor"),
        status,
    )
    .with_command(
        Command::new("start")
            .arg(Arg::new("programs").num_args(1..).required(true))
            .about("Start the given list of programs"),
        start,
    )
    .with_command(
        Command::new("stop")
            .arg(Arg::new("programs").num_args(1..).required(true))
            .about("Stop the given list of programs"),
        stop,
    );
    repl.run()
}
