use std::io::{BufRead, Write};

use daemonize::Result;
use reedline_repl_rs::clap::{Arg, ArgMatches, Command};
use reedline_repl_rs::Repl;
use supervisor::Action;

use crate::ClientContext;

fn send_action(action: Action, ctx: &mut ClientContext) -> Result<Option<String>> {
    ctx.writer
        .write_all(format!("{}\n", action.to_string()).as_bytes())?;
    ctx.writer.flush()?;
    if action == Action::Quit {
        std::process::exit(0)
    }

    let mut buf = String::new();
    ctx.reader.read_line(&mut buf)?;

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

pub(crate) fn supervisor_ctl(ctx: ClientContext) -> Result<()> {
    let mut repl = Repl::new(ctx)
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
    Ok(repl.run()?)
}
