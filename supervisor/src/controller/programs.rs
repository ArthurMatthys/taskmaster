use logger::{log, LogInfo};

use crate::Action;
use std::{collections::HashSet, fs::File, io::BufReader};

use crate::model::{Error, Origin, Programs, Result};

impl Programs {
    // loads a new configuration from a file, returns it. Doesn't change the current state
    pub fn new_from_path(path: String, start_process: bool) -> Result<Programs> {
        let mut args = path.split_whitespace();
        match args.next() {
            Some(filename) => {
                let rdr = match File::open(filename) {
                    Ok(file) => BufReader::new(file),
                    Err(e) => return Err(Error::Read(format!("File error : {}", e))),
                };
                match serde_yaml::from_reader::<_, Programs>(rdr) {
                    Ok(mut new_config) => {
                        new_config.programs.iter_mut().for_each(|(name, program)| {
                            program.name = name.clone();
                        });
                        if args.next().is_some() {
                            Err(Error::TooManyArguments)
                        } else {
                            if start_process {
                                new_config.start_all()?;
                            }
                            Ok(new_config)
                        }
                    }
                    Err(e) => Err(Error::De(format!("Deserialise error : {}", e))),
                }
            }
            None => Err(Error::NoFilenameProvided),
        }
    }
    pub fn new(start_process: bool) -> Result<Programs> {
        let path = match std::env::var("TASKMASTER_CONFIG_FILE_PATH") {
            Ok(path) => path,
            Err(e) => {
                log(
                    "Could not find env variable for taskmaster config\n".to_string(),
                    LogInfo::Error,
                )?;
                return Err(Error::ConfigEnvVarNotFound(e));
            }
        };
        Self::new_from_path(path, start_process)
    }

    pub fn check(&mut self) -> Result<()> {
        self.programs.iter_mut().try_for_each(|(_, p)| p.check())
    }

    pub fn update_config(&mut self) -> Result<Programs> {
        let mut new_config = Self::new(false)?;
        let mut dealt = HashSet::new();

        for (name, new_p) in new_config.programs.iter_mut() {
            dealt.insert(name);
            if let Some(p) = self.programs.get_mut(name) {
                p.update_program(new_p)?;
            } else {
                new_p.start_process(Origin::Config)?;
            }
        }
        self.programs
            .iter_mut()
            .filter(|(name, _)| !dealt.contains(name))
            .for_each(|(_, p)| p.kill_processes());
        Ok(new_config)
    }

    pub fn start_all(&mut self) -> Result<()> {
        self.programs
            .iter_mut()
            .try_for_each(|(_, p)| p.start_process(Origin::Config))
    }

    pub fn status(&mut self) -> String {
        format!(
            "{}\n",
            self.programs
                .iter_mut()
                .map(|(_, p)| p.status())
                .collect::<Vec<_>>()
                .join(" // ")
        )
    }

    pub fn stop(&mut self, programs: &[String]) -> Result<()> {
        self.programs
            .iter_mut()
            .filter(|(name, _)| programs.contains(name))
            .try_for_each(|(_, p)| p.stop_processes())?;
        Ok(())
    }

    pub fn start(&mut self, programs: &[String]) -> Result<()> {
        self.programs
            .iter_mut()
            .filter(|(name, _)| programs.contains(name))
            .try_for_each(|(_, p)| p.start_process(Origin::CLI))?;
        Ok(())
    }

    pub fn restart(&mut self, programs: &[String]) -> Result<()> {
        self.programs
            .iter_mut()
            .filter(|(name, _)| programs.contains(name))
            .try_for_each(|(_, p)| p.restart_processes())?;
        // self.stop(programs)?;
        // self.start(programs)?;
        Ok(())
    }

    pub fn handle_action(&mut self, action: Action) -> Result<String> {
        Ok(match action {
            Action::Start(programs) => {
                self.start(&programs)?;
                "Programs started\n".to_string()
            }
            Action::Stop(programs) => {
                self.stop(&programs)?;
                "Programs stopped\n".to_string()
            }
            Action::Restart(programs) => {
                self.restart(&programs)?;
                "Programs restarted\n".to_string()
                // self.relaunch(),
            }
            Action::Status => self.status(),
            // reload the config file
            Action::Reload => {
                self.programs = self.update_config()?.programs;
                "Reload done\n".to_string()
            }
            // clean stop the job control and exit
            // Handled in the server
            Action::Quit => {
                unreachable!();
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::ProgramState;

    use super::*;

    fn sleep(time: u64) {
        std::thread::sleep(Duration::from_secs(time));
    }

    fn config() -> Programs {
        let data = r#"
        programs:
            sleep:
              cmd: "/usr/bin/sleep 10"
              numprocs: 1
              umask: 022
              workingdir: /tmp
              autostart: true
              autorestart: unexpected
              exitcodes:
                - 0
                - 2
              startretries: 3
              startsecs: 5
              stopsignal: TERM
              stoptime: 1
              stdout: "/tmp/nginx.stdout"
              stderr: "/tmp/nginx.stderr"
              env:
                STARTED_BY: taskmaster
                ANSWER: 42
        "#;
        let mut v: Programs = serde_yaml::from_str(data).unwrap();
        v.programs.iter_mut().for_each(|(name, program)| {
            program.name = name.clone();
        });
        v
    }
    fn first_child_state(programs: &Programs) -> ProgramState {
        programs
            .programs
            .get("sleep")
            .unwrap()
            .children
            .first()
            .unwrap()
            .state
            .clone()
    }

    #[test]
    fn test_reload() -> Result<()> {
        let mut programs = config();
        programs.start_all()?;
        assert_eq!(first_child_state(&programs), ProgramState::Starting);
        programs.check()?;
        assert_eq!(first_child_state(&programs), ProgramState::Running);
        programs.restart(&["sleep".to_string()])?;
        assert_eq!(first_child_state(&programs), ProgramState::Restarting);
        programs.check()?;
        assert_eq!(first_child_state(&programs), ProgramState::Restarting);
        sleep(2);
        programs.check()?;
        assert_eq!(first_child_state(&programs), ProgramState::Starting);
        programs.check()?;
        assert_eq!(first_child_state(&programs), ProgramState::Running);
        Ok(())
    }
}
