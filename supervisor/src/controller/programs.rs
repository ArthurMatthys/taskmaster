use crate::Action;
use std::{fs::File, io::BufReader};

use crate::model::{Error, Programs, Result};

impl Programs {
    // loads a new configuration from a file, returns it. Doesn't change the current state
    pub fn new() -> Result<Programs> {
        let path = match std::env::var("TASKMASTER_CONFIG_FILE_PATH") {
            Ok(path) => path,
            Err(e) => return Err(Error::ConfigEnvVarNotFound(e)),
        };
        let mut args = path.split_whitespace();
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
                    Err(e) => Err(Error::De(format!("Deserialise error : {}", e))),
                }
            }
            None => Err(Error::NoFilenameProvided),
        }
    }

    pub fn check(&mut self) -> Result<()> {
        self.programs.iter_mut().try_for_each(|(_, p)| p.check())
    }

    pub fn update_config(&mut self) -> Result<()> {
        todo!();
    }

    pub fn status(&mut self) -> String {
        self.programs
            .iter_mut()
            .map(|(_, p)| p.status())
            .collect::<Vec<_>>()
            .join(" // ")
    }

    pub fn stop(&mut self, programs: Vec<String>) -> String {
        todo!();
    }

    pub fn start(&mut self, programs: Vec<String>) -> String {
        todo!();
    }

    pub fn restart(&mut self, programs: Vec<String>) -> String {
        todo!();
    }

    pub fn handle_action(&mut self, action: Action) -> Result<String> {
        Ok(match action {
            Action::Start(programs) => {
                self.start(programs);
                "Programs started".to_string()
            }
            Action::Stop(programs) => {
                self.stop(programs);
                "Programs stopped".to_string()
            }
            Action::Restart(programs) => {
                self.restart(programs);
                "Programs restarted".to_string()
                // self.relaunch(),
            }
            Action::Status => {
                self.status()
                // self.status(),
            }
            // reload the config file
            Action::Reload => {
                self.update_config()?;
                "Reload done".to_string()
                // self.reload(),
            }
            // clean stop the job control and exit
            // Handled in the server
            Action::Quit => {
                unreachable!();
            }
        })
    }
}
