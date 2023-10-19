use crate::model::ChildProcess;
use crate::model::Program;
use crate::model::ProgramState;
use crate::model::Result;

// use logger::{log, LogInfo};

impl Program {
    pub fn start_process(&mut self) -> Result<()> {
        if !self.auto_start {
            return Ok(());
        }

        for num_proc in 0..self.num_procs {
            match ChildProcess::start(&self, num_proc) {
                Ok(child_process) => self.children.push(child_process),
                Err(e) => return Err(e),
            }
        }

        // start process ensures that the process is started, but it does not ensure that the process is ready to receive requests.
        // This is a hack to wait for the process to be ready to receive requests.
        std::thread::sleep(std::time::Duration::from_millis(500));
        Ok(())
    }

    pub fn check(&mut self) -> Result<()> {
        let self_clone = self.clone(); // Improvement: remove clone?

        for (index, child_process) in self.children.iter_mut().enumerate() {
            match child_process.check(&self_clone, index as u8) {
                Ok(_) => continue,
                Err(e) => return Err(e),
            }
        }

        self.reconcile_state()?;

        Ok(())
    }

    pub fn reconcile_state(&mut self) -> Result<()> {
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

        Ok(())
    }

    pub fn update_program(&mut self, new_program: &Program) -> Result<()> {
        let current_num_procs = self.children.len();
        let new_num_procs = new_program.num_procs as usize;

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
            for num_proc in 0..new_num_procs {
                match ChildProcess::start(&self, num_proc as u8) {
                    Ok(child_process) => self.children.push(child_process),
                    Err(e) => return Err(e),
                }
            }
        } else if self.num_procs > new_program.num_procs {
            for _ in new_num_procs..current_num_procs {
                if let Some(child_process) = &mut self.children.pop() {
                    child_process.kill_program();
                }
            }
        } else if self.num_procs < new_program.num_procs {
            let len = new_program.num_procs - self.num_procs;
            for num_proc in 0..len {
                match ChildProcess::start(&self, num_proc) {
                    Ok(child_process) => self.children.push(child_process),
                    Err(e) => return Err(e),
                }
            }
        }

        if self.stop_signal != new_program.stop_signal {
            self.stop_signal = new_program.stop_signal.clone();
        }

        if self.start_secs != new_program.start_secs {
            self.start_secs = new_program.start_secs.clone();
        }

        Ok(())
    }
}
