use crate::model::{AutoRestart, ChildProcess, Output, Program, ProgramState};

use crate::model::{Error, Programs, Result};
use logger::{log, LogInfo};
use std::fs::OpenOptions;
use std::path::Path;
use std::time::Instant;

// use crate::model::{Error, Programs, Result};
use libc::umask;
// use std::env;
use std::fs::File;
use std::process::{Command, Stdio};

use super::program;

// FnOnce limits the amount of time a closure can be called
// as we run it in a loop, it's always a new one
// not limiting then
pub fn with_umask<F: FnOnce() -> Result<ChildProcess>>(mask: u16, f: F) -> Result<ChildProcess> {
    unsafe {
        let old_mask = umask(mask);
        let result = f();
        umask(old_mask);
        match result {
            Ok(child_process) => Ok(child_process),
            Err(e) => Err(e),
        }
    }
}

impl ChildProcess {
    pub fn start(program: &Program, process_number: u8) -> Result<ChildProcess> {
        let umask = u16::from_str_radix(&program.umask, 8).unwrap_or(0o022);

        with_umask(umask, || {
            let mut command = Command::new(&program.cmd.0);

            if !program.cmd.1.is_empty() {
                command.args(&program.cmd.1);
            }

            command.current_dir(&program.working_dir);

            if let Some(env_vars) = &program.env {
                command.envs(env_vars);
            }

            match &program.stdout {
                Output::File(path) => {
                    let file = File::create(path).unwrap();
                    command.stdout(Stdio::from(file));
                }
                _ => {
                    let log_dir = "logger";
                    let file_name = format!("{}_stdout_{}", program.name, process_number);
                    let log_path = Path::new(log_dir).join(file_name);
                    let io = match OpenOptions::new().append(true).create(true).open(&log_path) {
                        Ok(file) => Stdio::from(file),
                        Err(e) => {
                            _ = log(
                                format!(
                                    "Failed to open log file: {}, at path {}",
                                    e,
                                    log_path.to_str().unwrap()
                                ),
                                LogInfo::Error,
                            );
                            Stdio::null()
                        }
                    };
                    command.stdout(io);
                }
            }

            match &program.stderr {
                Output::File(path) => {
                    let file = File::create(path).unwrap();
                    command.stderr(Stdio::from(file));
                }
                _ => {
                    let log_dir = "logger";
                    let file_name = format!("{}_stderr_{}", program.name, process_number);
                    let log_path = Path::new(log_dir).join(file_name);
                    let io = match OpenOptions::new().append(true).create(true).open(&log_path) {
                        Ok(file) => Stdio::from(file),
                        Err(e) => {
                            _ = log(
                                format!(
                                    "Failed to open log file: {}, at path {}",
                                    e,
                                    log_path.to_str().unwrap()
                                ),
                                LogInfo::Error,
                            );
                            Stdio::null()
                        }
                    };
                    command.stderr(io);
                }
            }

            let child = command.spawn()?;

            _ = log(
                format!(
                    "Started process: {}, number {}",
                    program.name, process_number
                ),
                LogInfo::Info,
            );

            Ok(ChildProcess {
                child: Some(child),
                state: ProgramState::Starting,
                exit_status: None,
                start_secs: Some(Instant::now()),
                end_time: None, // killed, fatal, stopped, exited -- state that cannot be changed
                restart_count: 0,
            })
        })
    }

    pub fn get_child_exit_status(&mut self) -> Option<i32> {
        match self.child.as_mut()?.try_wait() {
            Ok(Some(status)) => Some(status.code().unwrap_or(1)),
            Ok(None) => None,
            Err(e) => {
                eprintln!("Failed to wait on child: {}", e);
                Some(-1) // internal exit status that doesn't exist
            }
        }
    }

    pub fn is_exit_status_in_config(&self, config: &Program) -> bool {
        match self.exit_status {
            Some(status) => (status >= 0) && config.exitcodes.contains(&(status as u8)),
            None => false,
        }
    }

    pub fn kill_program(&mut self) {
        if let Some(child) = &mut self.child {
            let _ = child.kill();
            // ensure state has been updated at the process level
            // std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    pub fn rerun_program(&mut self, program: &Program, process_number: u8) -> Result<ChildProcess> {
        ChildProcess::start(program, process_number)
    }

    pub fn increment_start_retries(&mut self) {
        self.restart_count += 1;
    }

    pub fn check(&mut self, config: &Program, process_number: u8) {
        let elapsed_time = self.start_secs.map_or(0, |start_time| {
            Instant::now().duration_since(start_time).as_secs()
        });

        match &self.state {
            ProgramState::Starting => {
                if elapsed_time < (config.start_secs as u64) {
                    return;
                }

                self.exit_status = self.get_child_exit_status();
                if self.exit_status.is_some() {
                    if self.is_exit_status_in_config(config) {
                        // exit with expected status
                        self.state = ProgramState::Exited;
                    } else if config.auto_restart == AutoRestart::Never {
                        // cannot be restarted
                        self.state = ProgramState::Pending;
                    } else {
                        // backoff
                        self.kill_program();
                        let _ = self.rerun_program(config, process_number);
                        self.increment_start_retries();
                        self.state = ProgramState::Backoff;
                    }
                } else {
                    // running
                    self.restart_count = 0;
                    self.state = ProgramState::Running;
                }
            }
            ProgramState::Running => {
                println!("Process is running");
            }
            ProgramState::Backoff => {
                if elapsed_time < (config.start_secs as u64) {
                    return;
                }

                self.exit_status = self.get_child_exit_status();
                if self.exit_status.is_some() {
                    if self.restart_count >= config.start_retries {
                        self.kill_program();
                        self.state = ProgramState::Fatal;
                    } else {
                        self.kill_program();
                        let _ = self.rerun_program(config, process_number);
                        self.increment_start_retries();
                        self.state = ProgramState::Backoff;
                    }
                } else {
                    self.restart_count = 0;
                    self.state = ProgramState::Running;
                }
            }
            ProgramState::Stopping => {
                println!("Process is stopping");
            }
            ProgramState::Stopped => {
                println!("Process is stopped");
            }
            ProgramState::Exited => {
                println!("Process has exited");
            }
            ProgramState::Fatal => {
                println!("Process has encountered a fatal error");
            }
            ProgramState::Killed => {
                println!("Process was killed");
            }
            _ => {
                panic!("Unknown program state");
            }
        }
    }

    // pub fn stop(&mut self) -> Result<()> {
    //     self.child.kill()?;
    //     self.state = ProgramState::Stopped;
    //     self.end_time = Some(Instant::now());
    //     Ok(())
    // }

    // pub fn kill(&mut self) -> Result<()> {
    //     self.child.kill()?;
    //     self.state = ProgramState::Killed;
    //     self.end_time = Some(Instant::now());
    //     Ok(())
    // }
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

    // use crate::model::Program::AutoRestart;
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
                    exit_status: None,
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
                    exit_status: None,
                    start_secs: Some(Instant::now()),
                    end_time: None,
                    restart_count: 0,
                })
            });
        }
    }

    #[test]
    fn test_check_starting_running() {
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
            stdout: Output::None,
            stderr: Output::None,
            children: vec![],
        };

        let mut child_process = ChildProcess::start(&program, 0).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(2));
        child_process.check(&program, 0);

        assert_eq!(child_process.state, ProgramState::Running);
    }

    #[test]
    fn test_check_starting_exited() {
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
            stdout: Output::None,
            stderr: Output::None,
            children: vec![],
        };

        let mut child_process = ChildProcess::start(&program, 0).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        child_process.check(&program, 0);

        assert_eq!(child_process.state, ProgramState::Exited);
    }

    #[test]
    fn test_check_starting_backoff() {
        let program = Program {
            name: "sleep_backoff".to_string(),
            cmd: ("/bin/sleep".to_string(), vec!["1".to_string()]),
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
            stdout: Output::None,
            stderr: Output::None,
            children: vec![],
        };

        let mut child_process = ChildProcess::start(&program, 0).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));

        use libc::{kill, SIGKILL};
        let pid = child_process.child.as_ref().unwrap().id() as libc::pid_t;
        let _ = unsafe { kill(pid, SIGKILL) };
        std::thread::sleep(std::time::Duration::from_millis(100));

        child_process.check(&program, 0);

        assert_eq!(child_process.state, ProgramState::Backoff);
    }

    #[test]
    fn test_check_starting_pending() {
        let program = Program {
            name: "sleep_pending".to_string(),
            cmd: ("/bin/sleep".to_string(), vec!["1".to_string()]),
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
            stdout: Output::None,
            stderr: Output::None,
            children: vec![],
        };

        let mut child_process = ChildProcess::start(&program, 0).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));

        use libc::{kill, SIGKILL};
        let pid = child_process.child.as_ref().unwrap().id() as libc::pid_t;
        let _ = unsafe { kill(pid, SIGKILL) };
        std::thread::sleep(std::time::Duration::from_millis(100));

        child_process.check(&program, 0);

        assert_eq!(child_process.state, ProgramState::Pending);
    }
}
