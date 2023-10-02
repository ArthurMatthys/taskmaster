use crate::Action;
use std::{fs::File, io::BufReader, str::SplitWhitespace};

use crate::model::{Error, Programs, Result};

impl Programs {
    // loads a new configuration from a file, returns it. Doesn't change the current state
    pub fn load_config(&self, mut args: SplitWhitespace) -> Result<Programs> {
        match args.next() {
            Some(filename) => {
                let rdr = match File::open(filename) {
                    Ok(file) => BufReader::new(file),
                    Err(e) => return Err(Error::Read(format!("File error : {}", e))),
                };
                match serde_yaml::from_reader::<_, Programs>(rdr) {
                    Ok(new_config) => {
                        if args.next().is_some() {
                            Err(Error::TooManyArguments)
                        } else {
                            Ok(new_config)
                        }
                    }
                    Err(e) => return Err(Error::De(format!("Deserialise error : {}", e))),
                }
            }
            None => Err(Error::NoFilenameProvided),
        }
    }

    // pub fn reconcile_state(&mut self, new_config: Programs) -> Result<()> {
    //     // Iterate over the new configuration
    //     for (name, new_program) in new_config.programs {
    //         // If the program is not in the current state, start it
    //         if let Some(current_program) = self.programs.get_mut(&name) {
    //             current_program.reconcile_state(new_program)?;
    //         } else {
    //             // If the program is not in the current state, start it
    //             self.action_fn(Action::Start(vec![new_program.name]));
    //         }
    //     }

    //     // Iterate over the current state
    //     for (name, current_program) in &mut self.programs {
    //         // If the program is not in the new configuration, stop it
    //         if !new_config.programs.contains_key(name) {
    //             self.action_fn(Action::Stop(vec![name.to_string()]));
    //         }
    //     }

    //     Ok(())
    // }

    pub fn action_fn(&mut self, action: Action) {
        match action {
            Action::Start(programs) => {
                for program in programs.iter() {
                    _ = program;
                }
            }
            Action::Stop(programs) => {
                for program in programs.iter() {
                    _ = program;
                }
                unimplemented!();
                // self.stop()
            }
            Action::Restart(programs) => {
                for program in programs.iter() {
                    _ = program;
                }
                unimplemented!();
                // self.relaunch(),
            }
            Action::Status => {
                unimplemented!();
                // self.status(),
            }
            // reload the config file
            Action::Reload => {
                unimplemented!();
                // self.reload(),
            }
            // clean stop the job control and exit
            Action::Quit => {
                unimplemented!();
                // self.quit(),
            }
        }
    }
}
