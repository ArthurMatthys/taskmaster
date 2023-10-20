use std::time::Instant;

use logger::{log, LogInfo};

use crate::model::ChildProcess;
use crate::model::Origin;
use crate::model::Program;
use crate::model::ProgramState;
use crate::model::Result;
use crate::ChildExitStatus;
use crate::Error;

// use logger::{log, LogInfo};

impl Program {
    pub fn check(&mut self) -> Result<()> {
        let self_clone = self.clone();

        for (index, child_process) in self.children.iter_mut().enumerate() {
            match child_process.check(&self_clone, index as u8) {
                Ok(_) => continue,
                Err(Error::WaitError(e)) => {
                    return Err(Error::WaitError(e));
                }
                Err(_e) => {
                    continue;
                }
            }
        }

        self.reconcile_state();

        Ok(())
    }

    pub fn reconcile_state(&mut self) {
        let mut killed = false;
        let mut fatal = false;
        let mut stopped = true;
        let mut pending = false;

        for child_process in &self.children {
            match child_process.state {
                ProgramState::Killed => killed = true,
                ProgramState::Fatal => fatal = true,
                ProgramState::Stopped => stopped = true,
                ProgramState::Pending => pending = true,
                _ => (),
            }
        }

        if killed {
            for child_process in &mut self.children {
                child_process.kill_program();
                child_process.state = ProgramState::Killed;
            }
        } else if fatal {
            for child_process in &mut self.children {
                child_process.kill_program();
                child_process.state = ProgramState::Fatal;
            }
        } else if stopped {
            for child_process in &mut self.children {
                child_process.kill_program();
                child_process.state = ProgramState::Stopped;
            }
        } else if pending {
            for child_process in &mut self.children {
                child_process.state = ProgramState::Pending;
            }
        }
    }

    pub fn start_process(&mut self, origin: Origin) -> Result<()> {
        if origin == Origin::Config && !self.auto_start {
            return Ok(());
        }

        let amount_of_process_running = self.children.len() as u8;
        for num_proc in amount_of_process_running..self.num_procs {
            match ChildProcess::start(self, num_proc) {
                Ok(child_process) => self.children.push(child_process),
                Err(e) => {
                    let _ = log(format!("Failed to rerun program: {}", e), LogInfo::Error);
                    self.children.push(ChildProcess {
                        child: None,
                        state: ProgramState::Backoff,
                        exit_status: ChildExitStatus::NonExistent,
                        start_secs: Some(Instant::now()),
                        end_time: None,
                        restart_count: 0,
                    })
                }
            }
        }
        Ok(())
    }

    pub fn kill_processes(&mut self) {
        self.children.iter_mut().for_each(|c| c.kill_program());
        self.children.clear();
    }

    pub fn stop_processes(&mut self) {
        // self.children
        //     .iter_mut()
        //     .for_each(|p| p.stop(self.stop_signal as libc::c_int))
    }

    pub fn update_program(&mut self, new_program: &Program) -> Result<()> {
        let current_num_procs = self.children.len();
        let new_num_procs = new_program.num_procs as usize;

        // if any of these parameters change, we need to restart the program
        if self.name != new_program.name
            || self.cmd != new_program.cmd
            || self.auto_restart != new_program.auto_restart
            || self.exitcodes != new_program.exitcodes
            || self.start_retries != new_program.start_retries
            || self.auto_start != new_program.auto_start
            || self.stop_signal != new_program.stop_signal
            || self.env != new_program.env
            || self.working_dir != new_program.working_dir
            || self.umask != new_program.umask
            || self.stdout != new_program.stdout
            || self.stderr != new_program.stderr
        {
            for child_process in &mut self.children {
                child_process.kill_program();
            }

            self.children.clear();
            self.start_process(Origin::Config)?;
        // if the number of processes is less, we need to kill the extra processes
        } else if self.num_procs > new_program.num_procs {
            for _ in new_num_procs..current_num_procs {
                if let Some(child_process) = &mut self.children.pop() {
                    child_process.kill_program();
                }
            }
        // if the number of processes is more, we need to start the extra processes
        } else if self.num_procs < new_program.num_procs {
            if let Err(e) = self.start_process(Origin::Config) {
                let _ = log(format!("Failed to start program: {}", e), LogInfo::Error);
            }
        }

        if self.stop_signal != new_program.stop_signal {
            self.stop_signal = new_program.stop_signal.clone();
        }

        if self.start_secs != new_program.start_secs {
            self.start_secs = new_program.start_secs;
        }

        Ok(())
    }

    pub fn status(&mut self) -> String {
        format!(
            "{:?} : {}",
            self.name,
            if let Some(s) = self.children.first().map(|c| c.state.clone()) {
                s.to_string()
            } else {
                "Inactive program".to_string()
            }
        )
    }
}
