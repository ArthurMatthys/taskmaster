use crate::controller::load_config;
use crate::model::{Args, Error, Programs, Result};
use std::{
    fs::File,
    io::{stdin, stdout, BufReader, Write},
};

pub fn supervisor(args: Args) -> Result<()> {
    let mut programs = match File::open(args.path) {
        Ok(f) => {
            let rdr = BufReader::new(f);
            match serde_yaml::from_reader::<_, Programs>(rdr) {
                Ok(p) => p,
                Err(err) => return Err(Error::De(format!("{err}"))),
            }
        }
        Err(err) => return Err(Error::Read(format!("{err}"))),
    };
    let mut stdout = stdout();

    loop {
        print!("Supervisor> ");
        stdout.flush();

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        let mut parts = input.split_whitespace();
        let cmd = if let Some(cmd) = parts.next() {
            cmd.to_lowercase()
        } else {
            eprintln!("No command supplied");
            continue;
        };

        let args = parts;
        match cmd.as_str() {
            "status" => programs.status(),
            "start" => programs.action("start", args),
            "stop" => programs.action("stop", args),
            "relaunch" => programs.action("relaunch", args),
            "reload" => programs = load_config(args, &programs)?,
            "exit" | "quit" => break,
            _ => {
                eprintln!("Supervisor: Unknown command : {cmd}.");
                continue;
            }
        };
    }
    Ok(())
}
