use serde::Deserialize;
use std::collections::HashMap;
// use libc;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AutoRestart {
    Always,
    Never,
    Unexpected,
}

// #[derive(Debug, Deserialize)]
// pub enum ExitCode {
//     Codes(Vec<u8>),
//     Code(u8),
// }

#[derive(Debug, Deserialize)]
pub enum Output {
    File(String),
    Fd(u16),
    None,
}

// TODO : Change it or choose enum from libc ?
#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum StopSignal {
    Exit,
    Usr1,
    Term,
}

#[derive(Debug, Deserialize, Default)]
pub struct Programs {
    pub programs: HashMap<String, Program>,
}

#[derive(Debug, Deserialize)]
pub struct Program {
    pub cmd: String,

    #[serde(alias = "numprocs")]
    pub num_procs: u8,

    #[serde(alias = "autostart")]
    pub auto_start: bool,

    #[serde(alias = "autorestart")]
    pub auto_restart: AutoRestart,

    pub exitcodes: Vec<u8>,

    #[serde(alias = "startretries")]
    pub start_retries: u8,

    #[serde(alias = "starttime")]
    pub start_time: u16,

    #[serde(alias = "stopsignal")]
    pub stop_signal: StopSignal,

    #[serde(alias = "stoptime")]
    pub stop_time: u16,

    pub env: Option<HashMap<String, String>>,

    #[serde(alias = "workingdir")]
    pub working_dir: String,

    pub umask: String,

    pub stdout: Output,
    pub stderr: Output,
}
