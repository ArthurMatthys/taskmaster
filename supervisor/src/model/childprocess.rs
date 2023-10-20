use std::process::Child;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

#[derive(Debug, Clone)]
pub enum ChildExitStatus {
    Exited(i32),
    Running,
    NonExistent,
    WaitError(String),
}

// https://docs.red-dove.com/supervisor/events.html#process-state-event-type
#[derive(Debug, PartialEq, Clone)]
pub enum ProgramState {
    // trying to start the process
    Starting,
    // process has successfully started
    Running,
    // process did not successfully enter the RUNNING state.
    // Taskmaster is going to try to restart it unless it has exceeded its “startretries” configuration limit.
    // Ends up in running or Fatal
    Backoff,
    // process will be gracefully stopped using the configured signal
    // stopping ends up in stopped or Killed
    Stopping,
    // process has been successfully stopped
    Stopped,
    // process was running but has exited (or exited and was restarted)
    // exist status == one of the exitcodes of user
    Exited,
    // Taskmaster tried startretries number of times unsuccessfully to start the process,
    // and gave up attempting to restart it.
    Fatal,
    // process was stopped using the sigkill signal
    Killed,
    // program has auto_restart set to Never
    Pending,
    // Unknown state (should never happen)
    Error,
}

#[derive(Debug, Clone)]
pub struct ChildProcess {
    pub child: Option<Arc<Mutex<Child>>>,
    pub state: ProgramState,
    pub exit_status: ChildExitStatus,
    pub start_secs: Option<Instant>,
    pub end_time: Option<Instant>,
    pub restart_count: u8,
}
