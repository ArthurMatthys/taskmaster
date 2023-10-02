use crate::ChildProcess;
use serde::{Deserialize, Deserializer};

use std::collections::HashMap;
// use libc;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AutoRestart {
    Always,
    Never,
    Unexpected,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum Output {
    File(String),
    Fd(u16),
    None,
}

// TODO : Change it or choose enum from libc ?
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum StopSignal {
    Exit,
    Usr1,
    Term,
}

// default umask
fn default_umask() -> String {
    "0o022".to_string()
}

// validate octal format of the umask
fn deserialize_octal_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    match u16::from_str_radix(&s[2..], 8) {
        Ok(_) => Ok(s),
        Err(_) => Err(serde::de::Error::custom("Invalid octal format")),
    }
}

fn split_cmd_and_args<'de, D>(deserializer: D) -> Result<(String, Vec<String>), D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let mut split = s.split_whitespace();
    let cmd = split.next().unwrap_or("").to_string();
    let args = split.map(String::from).collect();
    Ok((cmd, args))
}

#[derive(Debug, Deserialize)]
pub struct Program {
    pub name: String,

    // command to execute and its arguments
    #[serde(deserialize_with = "split_cmd_and_args")]
    pub cmd: (String, Vec<String>),

    // number of process to start
    #[serde(alias = "numprocs")]
    pub num_procs: u8,

    // auto start the program
    // default : true
    // otherwise, the program will be started only
    // with the CLI
    #[serde(alias = "autostart")]
    pub auto_start: bool,

    // auto restart the program
    // when the program exit, it will be restarted
    // unless the exit code is in the exitcodes list
    // or it is stopped by the user, using the CLI
    #[serde(alias = "autorestart")]
    pub auto_restart: AutoRestart,

    // all exit codes that will be considered as
    // a normal exit (no restart)
    pub exitcodes: Vec<u8>,

    // number of times the program will be restarted
    // before giving up
    #[serde(alias = "startretries")]
    pub start_retries: u8,

    // number of seconds which the program needs to stay
    // running after a startup to consider the start successful
    #[serde(alias = "startsecs")]
    pub start_secs: u16,

    // signal sent by job control to all the process to stop it
    #[serde(alias = "stopsignal")]
    pub stop_signal: StopSignal,

    // number of seconds to wait before sending a SIGKILL
    // to all the process
    // before, shall consume all the retries
    #[serde(alias = "stoptime")]
    pub stop_time: u16,

    // environment variables to set
    pub env: Option<HashMap<String, String>>,

    // working directory to set
    #[serde(alias = "workingdir")]
    pub working_dir: String,

    // umask to set
    #[serde(
        default = "default_umask",
        deserialize_with = "deserialize_octal_string"
    )]
    pub umask: String,

    // stdout and stderr redirection
    pub stdout: Output,
    pub stderr: Output,

    // below part is internal, it will contain all the state fields
    // used by the supervisor to manage the program

    // keep track of all the child pids, to check exit status
    // and everything in the main server loop
    #[serde(skip)]
    pub children: Vec<ChildProcess>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_cmd() {
        let cmd_str = "/usr/local/bin/nginx -c /etc/nginx/test.conf";
        let deserializer = serde_yaml::Deserializer::from_str(cmd_str);
        let cmd = split_cmd_and_args(deserializer).unwrap();
        assert_eq!(cmd.0, "/usr/local/bin/nginx");
    }

    #[test]
    fn test_split_args() {
        let cmd_str = "/usr/local/bin/nginx -c /etc/nginx/test.conf";
        let deserializer = serde_yaml::Deserializer::from_str(cmd_str);
        let cmd = split_cmd_and_args(deserializer).unwrap();
        assert_eq!(cmd.1, vec!["-c", "/etc/nginx/test.conf"]);
    }

    #[test]
    fn test_program_deserialization() {
        let yaml = r#"
        name: "test_program"
        cmd: "/usr/local/bin/nginx -c /etc/nginx/test.conf"
        numprocs: 1
        autostart: true
        autorestart: "always"
        exitcodes: [0]
        startretries: 3
        startsecs: 10
        stopsignal: "TERM"
        stoptime: 10
        env: {"key": "value"}
        workingdir: "/path/to/dir"
        stdout: "None"
        stderr: "None"
        "#;

        let program: Program = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(program.cmd.0, "/usr/local/bin/nginx");
        assert_eq!(program.cmd.1, vec!["-c", "/etc/nginx/test.conf"]);
        assert_eq!(program.name, "test_program");
        assert_eq!(program.num_procs, 1);
        assert_eq!(program.auto_start, true);
        assert_eq!(program.auto_restart, AutoRestart::Always);
        assert_eq!(program.exitcodes, [0]);
        assert_eq!(program.start_retries, 3);
        assert_eq!(program.start_secs, 10);
        assert_eq!(program.stop_signal, StopSignal::Term);
        assert_eq!(program.stop_time, 10);

        let mut expected_env = HashMap::new();
        expected_env.insert("key".to_string(), "value".to_string());
        assert_eq!(program.env, Some(expected_env));

        assert_eq!(program.working_dir, "/path/to/dir");
        assert_eq!(program.umask, "0o022");
        assert_eq!(program.stdout, Output::None);
        assert_eq!(program.stderr, Output::None);
    }

    #[test]
    fn test_program_deserialization_tricky() {
        let yaml = r#"
    name: "test_program"
    cmd: "/usr/local/bin/nginx"
    numprocs: 2
    autostart: false
    autorestart: "never"
    exitcodes: [0, 1, 2]
    startretries: 5
    workingdir: "/tmp"
    startsecs: 15
    stopsignal: "USR1"
    stoptime: 20
    env: {"key1": "value1", "key2": "value2"}
    stdout: "None"
    stderr: "None"
    "#;

        let program: Program = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(program.cmd.0, "/usr/local/bin/nginx");
        assert!(program.cmd.1.is_empty());
        assert_eq!(program.name, "test_program");
        assert_eq!(program.num_procs, 2);
        assert_eq!(program.auto_start, false);
        assert_eq!(program.auto_restart, AutoRestart::Never);
        assert_eq!(program.exitcodes, [0, 1, 2]);
        assert_eq!(program.start_retries, 5);
        assert_eq!(program.start_secs, 15);
        assert_eq!(program.stop_signal, StopSignal::Usr1);
        assert_eq!(program.stop_time, 20);

        let mut expected_env = HashMap::new();
        expected_env.insert("key1".to_string(), "value1".to_string());
        expected_env.insert("key2".to_string(), "value2".to_string());
        assert_eq!(program.env, Some(expected_env));

        assert_eq!(program.umask, "0o022");
        assert_eq!(program.stdout, Output::None);
        assert_eq!(program.stderr, Output::None);
    }

    #[test]
    fn test_deserialize_octal_string() {
        // Test a valid octal string
        let s = "0o755".to_string();
        let deserializer = serde_yaml::Deserializer::from_str(&s);
        let result = deserialize_octal_string(deserializer).unwrap();
        assert_eq!(result, "0o755");

        // Test an invalid octal string
        let s = "0o999".to_string();
        let deserializer = serde_yaml::Deserializer::from_str(&s);
        let result = deserialize_octal_string(deserializer);
        assert!(result.is_err());

        // Test an invalid octal string
        let s = "898".to_string();
        let deserializer = serde_yaml::Deserializer::from_str(&s);
        let result = deserialize_octal_string(deserializer);
        assert!(result.is_err());
    }
}
