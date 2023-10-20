use crate::model::{AutoRestart, ChildExitStatus, ChildProcess, Program, ProgramState};

use crate::model::{Error, Result};
use logger::{log, LogInfo};
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

use libc::kill;
use libc::umask;
use std::fs::File;
use std::os::unix::process::ExitStatusExt;
use std::process::{Command, Stdio};

// FnOnce limits the amount of time a closure can be called
// as we run it in a loop, it's always a new one
// not limiting then
pub fn with_umask<F: FnOnce() -> Result<ChildProcess>>(mask: u32, f: F) -> Result<ChildProcess> {
    unsafe {
        let old_mask = umask(mask);
        let result = f();
        umask(old_mask);
        result
    }
}

impl ChildProcess {
    pub fn start(program: &Program, process_number: u8) -> Result<ChildProcess> {
        let umask = u32::from_str_radix(&program.umask, 8).unwrap_or(0o022);

        with_umask(umask, || {
            let mut command = Command::new(&program.cmd.0);

            if !program.cmd.1.is_empty() {
                command.args(&program.cmd.1);
            }

            command.current_dir(&program.working_dir);

            if let Some(env_vars) = &program.env {
                command.envs(env_vars);
            }

            let out_path = &program.stdout;
            let path = Path::new(&out_path);
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir)?;
            }
            let file = File::create(path)?;
            command.stdout(Stdio::from(file));

            let err_path = &program.stderr;
            let path = Path::new(&err_path);
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir)?;
            }
            let file = File::create(path)?;
            command.stderr(Stdio::from(file));

            let child = command.spawn()?;

            _ = log(
                format!(
                    "Started process: {}, number {}\n",
                    program.name, process_number
                ),
                LogInfo::Info,
            );

            Ok(ChildProcess {
                child: Some(Arc::new(Mutex::new(child))),
                state: ProgramState::Starting,
                exit_status: ChildExitStatus::Running,
                start_secs: Some(Instant::now()),
                end_time: None, // killed, fatal, stopped, exited -- state that cannot be changed
                restart_count: 0,
            })
        })
    }

    // lost in state transition between None that has no update and none where started didn't work
    pub fn get_child_exit_status(&mut self) -> Result<ChildExitStatus> {
        match self.child.as_mut() {
            Some(child) => {
                match child.lock() {
                    Ok(mut child) => match child.try_wait() {
                        Ok(Some(status)) => Ok(status.code().map_or_else(
                            || ChildExitStatus::Exited(status.signal().unwrap_or_default()),
                            ChildExitStatus::Exited,
                        )),
                        Ok(None) => Ok(ChildExitStatus::Running),
                        Err(e) => {
                            let _ =
                                log(format!("Failed to wait on child: {}\n", e), LogInfo::Error);
                            Ok(ChildExitStatus::WaitError(e.to_string())) // internal exit status that doesn't exist
                        }
                    },
                    Err(_) => {
                        let _ = log(
                            "Failed to lock child process.\n".to_string(),
                            LogInfo::Error,
                        );
                        Err(Error::IoError {
                            message: "Failed to lock child process.".to_string(),
                        })
                    }
                }
            }
            None => Ok(ChildExitStatus::NonExistent),
        }
    }

    pub fn is_exit_status_in_config(&self, config: &Program) -> bool {
        match self.exit_status {
            ChildExitStatus::Exited(status) => config.exitcodes.contains(&(status as u8)),
            _ => false,
        }
    }

    pub fn kill_program(&mut self) {
        if let Some(child) = &mut self.child {
            if let Ok(mut child) = child.lock() {
                let _ = child.kill();
            }
        }
    }

    pub fn stop(&mut self, sig: libc::c_int) {
        self.state = ProgramState::Stopping;
        let pid = self.child.as_ref().unwrap().lock().unwrap().id() as libc::pid_t;
        let _ = unsafe { kill(pid, sig) };
    }

    pub fn rerun_program(&mut self, program: &Program, process_number: u8) -> Result<ChildProcess> {
        let restart_count = self.restart_count;
        let updated_child = ChildProcess::start(program, process_number);
        self.restart_count = restart_count;
        updated_child
    }

    pub fn increment_start_retries(&mut self) {
        self.restart_count += 1;
    }

    pub fn check(&mut self, config: &Program, process_number: u8) -> Result<()> {
        let elapsed_start_time = self.start_secs.map_or(0, |start_time| {
            Instant::now().duration_since(start_time).as_secs()
        });

        match &self.state {
            ProgramState::Starting => {
                self.exit_status = self.get_child_exit_status()?;
                match &self.exit_status {
                    ChildExitStatus::Exited(_) => {
                        if self.is_exit_status_in_config(config) {
                            // exit with expected status
                            self.state = ProgramState::Exited;
                        } else if config.auto_restart == AutoRestart::Never {
                            if self.restart_count < config.start_retries {
                                self.state = ProgramState::Pending;
                            } else {
                                // cannot be restarted
                                self.state = ProgramState::Fatal;
                            }
                        } else {
                            if elapsed_start_time >= (config.start_secs as u64) {
                                self.increment_start_retries();
                                self.state = ProgramState::Backoff;
                            }
                            // backoff
                            self.kill_program();
                            if let Err(e) = self.rerun_program(config, process_number) {
                                let _ =
                                    log(format!("Failed to rerun program: {}", e), LogInfo::Error);
                                return Err(e);
                            }
                        }
                        Ok(())
                    }
                    ChildExitStatus::Running => {
                        if elapsed_start_time >= (config.start_secs as u64) {
                            self.restart_count = 0;
                            self.state = ProgramState::Running;
                        }
                        Ok(())
                    }
                    ChildExitStatus::NonExistent => unreachable!(),
                    ChildExitStatus::WaitError(e) => Err(Error::WaitError(e.clone())),
                }
            }
            ProgramState::Running => {
                self.exit_status = self.get_child_exit_status()?;
                match &self.exit_status {
                    ChildExitStatus::Exited(_) => {
                        if self.is_exit_status_in_config(config) {
                            // exit with expected status
                            self.state = ProgramState::Exited;
                        } else if config.auto_restart == AutoRestart::Never {
                            if self.restart_count < config.start_retries {
                                self.state = ProgramState::Pending;
                            } else {
                                // cannot be restarted
                                self.state = ProgramState::Fatal;
                            }
                        } else {
                            if elapsed_start_time >= (config.start_secs as u64) {
                                self.increment_start_retries();
                                self.state = ProgramState::Backoff;
                            }
                            // backoff
                            self.kill_program();
                            if let Err(e) = self.rerun_program(config, process_number) {
                                let _ =
                                    log(format!("Failed to rerun program: {}", e), LogInfo::Error);
                                return Err(e);
                            }
                        }
                        Ok(())
                    }
                    ChildExitStatus::Running => Ok(()),
                    ChildExitStatus::NonExistent => unreachable!(),
                    ChildExitStatus::WaitError(e) => Err(Error::WaitError(e.clone())),
                }
            }
            ProgramState::Backoff => {
                self.exit_status = self.get_child_exit_status()?;
                match &self.exit_status {
                    ChildExitStatus::Exited(_) => {
                        if !self.is_exit_status_in_config(config) {
                            self.kill_program();
                            if self.restart_count >= config.start_retries {
                                self.state = ProgramState::Fatal;
                            } else {
                                if let Err(e) = self.rerun_program(config, process_number) {
                                    let _ = log(
                                        format!("Failed to rerun program: {}", e),
                                        LogInfo::Error,
                                    );
                                    return Err(e);
                                }
                                self.increment_start_retries();
                                self.state = ProgramState::Backoff;
                            }
                        }
                        Ok(())
                    }
                    ChildExitStatus::Running => {
                        if elapsed_start_time >= (config.start_secs as u64) {
                            self.restart_count = 0;
                            self.state = ProgramState::Running;
                        }
                        Ok(())
                    }
                    ChildExitStatus::NonExistent => {
                        // starting previously failed
                        if elapsed_start_time >= (config.start_secs as u64) {
                            return Ok(());
                        }
                        if let Err(e) = self.rerun_program(config, process_number) {
                            let _ = log(format!("Failed to rerun program: {}", e), LogInfo::Error);
                            return Err(e);
                        }
                        self.increment_start_retries();
                        self.state = ProgramState::Backoff;
                        Ok(())
                    }
                    ChildExitStatus::WaitError(e) => Err(Error::WaitError(e.clone())),
                }
            }
            ProgramState::Stopping => {
                self.exit_status = self.get_child_exit_status()?;
                match &self.exit_status {
                    ChildExitStatus::Exited(_) => {
                        if self.is_exit_status_in_config(config) {
                            self.state = ProgramState::Stopped;
                        } else {
                            self.state = ProgramState::Fatal;
                        }
                        Ok(())
                    }
                    ChildExitStatus::WaitError(e) => Err(Error::WaitError(e.clone())),
                    _ => {
                        unreachable!()
                    }
                }
            }
            // final states that cannot be changed:
            // ProgramState::Exited
            // ProgramState::Killed
            // ProgramState::Stopped
            // ProgramState::Fatal
            // ProgramState::Pending
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::OpenOptionsExt;
    use std::os::unix::fs::PermissionsExt;

    use crate::with_umask;
    use crate::AutoRestart;
    use crate::ChildProcess;
    use crate::StopSignal;

    use crate::ProgramState;
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::time::Instant;

    struct Defer<F: FnOnce()>(Option<F>);

    impl<F: FnOnce()> Drop for Defer<F> {
        fn drop(&mut self) {
            if let Some(f) = self.0.take() {
                f();
            }
        }
    }

    #[test]
    fn test_with_multiple_umasks() {
        // Test with umask 0o022
        {
            let _defer = Defer(Some(|| {
                fs::remove_file("test_file_022").unwrap();
            }));

            let _ = with_umask(0o022, || {
                let mut file = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .mode(0o666)
                    .open("test_file_022")
                    .unwrap();
                file.write_all(b"content").unwrap();
                let metadata = fs::metadata("test_file_022").unwrap();
                let permissions = metadata.permissions();
                assert_eq!(permissions.mode() & 0o777, 0o644);
                Ok(ChildProcess {
                    child: None,
                    state: ProgramState::Running,
                    exit_status: ChildExitStatus::NonExistent,
                    start_secs: Some(Instant::now()),
                    end_time: None,
                    restart_count: 0,
                })
            });
        }

        // Test with umask 0o042
        {
            let _defer = Defer(Some(|| {
                fs::remove_file("test_file_042").unwrap();
            }));

            let _ = with_umask(0o042, || {
                let mut file = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .mode(0o666)
                    .open("test_file_042")
                    .unwrap();
                file.write_all(b"content").unwrap();
                let metadata = fs::metadata("test_file_042").unwrap();
                let permissions = metadata.permissions();
                assert_eq!(permissions.mode() & 0o777, 0o624);
                Ok(ChildProcess {
                    child: None,                  // This is the mock child process
                    state: ProgramState::Running, // or any other valid state
                    exit_status: ChildExitStatus::NonExistent,
                    start_secs: Some(Instant::now()),
                    end_time: None,
                    restart_count: 0,
                })
            });
        }
    }

    #[test]
    fn test_check_starting_running() -> Result<()> {
        let program = Program {
            name: "sleep_working".to_string(),
            cmd: ("/bin/sleep".to_string(), vec!["5".to_string()]),
            num_procs: 1,

            auto_start: false,
            auto_restart: AutoRestart::Always,

            exitcodes: vec![0],

            start_retries: 3,
            start_secs: 1,

            stop_signal: StopSignal::Usr1,
            stop_time: 1,
            env: None,
            working_dir: ".".to_string(),
            umask: "0o022".to_string(),
            stdout: "abc".to_string(),
            stderr: "abc".to_string(),
            children: vec![],
        };

        let mut child_process = ChildProcess::start(&program, 0).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        child_process.check(&program, 0)?;

        assert_eq!(child_process.state, ProgramState::Running);
        Ok(())
    }

    #[test]
    fn test_check_starting_exited() -> Result<()> {
        let program = Program {
            name: "sleep_exiting".to_string(),
            cmd: ("/bin/sleep".to_string(), vec!["0".to_string()]),
            num_procs: 1,

            auto_start: false,
            auto_restart: AutoRestart::Always,

            exitcodes: vec![0],

            start_retries: 3,
            start_secs: 1,

            stop_signal: StopSignal::Usr1,
            stop_time: 1,
            env: None,
            working_dir: ".".to_string(),
            umask: "0o022".to_string(),
            stdout: "abc".to_string(),
            stderr: "abc".to_string(),
            children: vec![],
        };

        let mut child_process = ChildProcess::start(&program, 0).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        child_process.check(&program, 0)?;

        assert_eq!(child_process.state, ProgramState::Exited);
        Ok(())
    }

    #[test]
    fn test_check_starting_backoff() -> Result<()> {
        let program = Program {
            name: "sleep_backoff".to_string(),
            cmd: ("/bin/sleep".to_string(), vec!["2".to_string()]),
            num_procs: 1,

            auto_start: false,
            auto_restart: AutoRestart::Always,

            exitcodes: vec![0],

            start_retries: 3,
            start_secs: 1,

            stop_signal: StopSignal::Usr1,
            stop_time: 1,
            env: None,
            working_dir: ".".to_string(),
            umask: "0o022".to_string(),
            stdout: "abc".to_string(),
            stderr: "abc".to_string(),
            children: vec![],
        };

        let mut child_process = ChildProcess::start(&program, 0).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));

        use libc::{kill, SIGKILL};
        let pid = child_process.child.as_ref().unwrap().lock().unwrap().id() as libc::pid_t;
        let _ = unsafe { kill(pid, SIGKILL) };
        std::thread::sleep(std::time::Duration::from_millis(100));

        child_process.check(&program, 0)?;

        assert_eq!(child_process.state, ProgramState::Backoff);
        Ok(())
    }

    #[test]
    fn test_check_starting_pending() -> Result<()> {
        let program = Program {
            name: "sleep_pending".to_string(),
            cmd: ("/bin/sleep".to_string(), vec!["3".to_string()]),
            num_procs: 1,

            auto_start: false,
            auto_restart: AutoRestart::Never,

            exitcodes: vec![0],

            start_retries: 3,
            start_secs: 1,

            stop_signal: StopSignal::Usr1,
            stop_time: 1,
            env: None,
            working_dir: ".".to_string(),
            umask: "0o022".to_string(),
            stdout: "abc".to_string(),
            stderr: "abc".to_string(),
            children: vec![],
        };

        let mut child_process = ChildProcess::start(&program, 0).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));

        use libc::{kill, SIGKILL};
        let pid = child_process.child.as_ref().unwrap().lock().unwrap().id() as libc::pid_t;
        let _ = unsafe { kill(pid, SIGKILL) };
        std::thread::sleep(std::time::Duration::from_millis(100));

        child_process.check(&program, 0)?;

        assert_eq!(child_process.state, ProgramState::Pending);
        Ok(())
    }

    #[test]
    fn test_check_backoff_fatal() -> Result<()> {
        let program = Program {
            name: "sleep_fatal".to_string(),
            cmd: ("/bin/sleep".to_string(), vec!["3".to_string()]),
            num_procs: 1,

            auto_start: false,
            auto_restart: AutoRestart::Always,

            exitcodes: vec![0],

            start_retries: 3,
            start_secs: 1,

            stop_signal: StopSignal::Usr1,
            stop_time: 1,
            env: None,
            working_dir: ".".to_string(),
            umask: "0o022".to_string(),
            stdout: "abc".to_string(),
            stderr: "abc".to_string(),
            children: vec![],
        };

        let mut child_process = ChildProcess::start(&program, 0).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));

        use libc::{kill, SIGKILL};
        let pid = child_process.child.as_ref().unwrap().lock().unwrap().id() as libc::pid_t;
        let _ = unsafe { kill(pid, SIGKILL) };
        std::thread::sleep(std::time::Duration::from_millis(100));

        // fakely put it in backoff mode
        child_process.state = ProgramState::Backoff;
        child_process.restart_count = 3;

        child_process.check(&program, 0)?;

        assert_eq!(child_process.state, ProgramState::Fatal);
        Ok(())
    }

    #[test]
    fn test_check_backoff_backoff() -> Result<()> {
        let program = Program {
            name: "sleep_fatal".to_string(),
            cmd: ("/bin/sleep".to_string(), vec!["3".to_string()]),
            num_procs: 1,

            auto_start: false,
            auto_restart: AutoRestart::Always,

            exitcodes: vec![0],

            start_retries: 3,
            start_secs: 1,

            stop_signal: StopSignal::Usr1,
            stop_time: 1,
            env: None,
            working_dir: ".".to_string(),
            umask: "0o022".to_string(),
            stdout: "abc".to_string(),
            stderr: "abc".to_string(),
            children: vec![],
        };

        let mut child_process = ChildProcess::start(&program, 0).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));

        let pid = child_process.child.as_ref().unwrap().lock().unwrap().id() as libc::pid_t;
        let _ = unsafe { kill(pid, SIGKILL) };
        std::thread::sleep(std::time::Duration::from_millis(100));

        // fakely put it in backoff mode
        child_process.state = ProgramState::Backoff;
        child_process.restart_count = 2;

        child_process.check(&program, 0)?;

        assert_eq!(child_process.state, ProgramState::Backoff);
        assert_eq!(child_process.restart_count, 3);
        Ok(())
    }
}
