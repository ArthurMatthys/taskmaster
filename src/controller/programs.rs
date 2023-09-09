use std::{fs::File, io::BufReader, str::SplitWhitespace};

use crate::model::{Error, Programs, Result};

pub fn load_config(mut args: SplitWhitespace, old: &Programs) -> Result<Programs> {
    match args.next() {
        Some(filename) => {
            let rdr = match File::open(filename) {
                Ok(file) => BufReader::new(file),
                Err(e) => return Err(Error::Read(format!("File error : {e}"))),
            };
            match serde_yaml::from_reader::<_, Programs>(rdr) {
                Ok(new_config) => {
                    old.programs.iter().for_each(|(name, p)| {
                        if !new_config.programs.contains_key(name) {
                            p.kill();
                        };
                    });
                    if args.next().is_some() {
                        Err(Error::TooManyArguments)
                    } else {
                        Ok(new_config)
                    }
                }
                Err(e) => return Err(Error::De(format!("Deserialise error : {e}"))),
            }
        }
        None => Err(Error::NoFilenameProvided),
    }
}

impl Programs {
    pub fn status(&self) {
        self.programs.iter().for_each(|(key, p)| p.status(key));
    }

    pub fn action(&self, action: &str, mut args: SplitWhitespace) {
        let usage = format!(
            "Error: {action} requires a process name
{action} <name>            {action} a process
{action} <name> <name>     {action} multiple processes
{action} all               {action} all processes"
        );

        let mut i = 0;
        loop {
            let arg = args.next();

            match arg {
                None => {
                    if i == 0 {
                        eprintln!("{usage}");
                    }
                    return;
                }
                Some("all") => self.programs.iter().for_each(|(_, p)| p.action_fn(action)),
                Some(name) => {
                    let mut target = self.programs.iter().filter(|(key, _)| *key == name);
                    if let Some((_, p)) = target.next() {
                        p.action_fn(action);
                    } else {
                        eprintln!("{name}: ERROR (no such process)");
                    }
                }
            }
            i += 1;
        }
    }

    // pub fn start(&self, mut args: SplitWhitespace) -> () {
    //     let usage = "Error: start requires a process name
    // start <name>            Start a process
    // start <name> <name>     Start multiple processes
    // start all               Start all processes";

    //     let mut i = 0;
    //     loop {
    //         let arg = args.next();

    //         match arg {
    //             None => {
    //                 if i == 0 {
    //                     eprintln!("{usage}");
    //                 }
    //                 return;
    //             }
    //             Some("all") => self.programs.iter().for_each(|(_, p)| p.start()),
    //             Some(name) => {
    //                 let mut target = self.programs.iter().filter(|(key, _)| *key == name);
    //                 if let Some((_, p)) = target.next() {
    //                     p.start();
    //                 } else {
    //                     eprintln!("{name}: ERROR (no such process)");
    //                 }
    //             }
    //         }
    //         i += 1;
    //     }
    // }
    // pub fn stop(&self, mut args: SplitWhitespace) -> () {
    //     let usage = "Error: stop requires a process name
    // stop <name>            Stop a process
    // stop <name> <name>     Stop multiple processes
    // stop all               Stop all processes";

    //     let mut i = 0;
    //     loop {
    //         let arg = args.next();

    //         match arg {
    //             None => {
    //                 if i == 0 {
    //                     eprintln!("{usage}");
    //                 }
    //                 return;
    //             }
    //             Some("all") => self.programs.iter().for_each(|(_, p)| p.stop()),
    //             Some(name) => {
    //                 let mut target = self.programs.iter().filter(|(key, _)| *key == name);
    //                 if let Some((_, p)) = target.next() {
    //                     p.stop();
    //                 } else {
    //                     eprintln!("{name}: ERROR (no such process)");
    //                 }
    //             }
    //         }
    //         i += 1;
    //     }
    // }
}
