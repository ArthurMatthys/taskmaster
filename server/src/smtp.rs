use std::env;

use daemonize::{Error, Result, TintinReporter};

pub fn get_smtp(reporter: &mut TintinReporter, dst: String) -> Result<()> {
    match dotenv::dotenv() {
        Ok(_) => (),
        Err(e) => {
            return Err(Error::DotEnv(e));
        }
    };
    let username = match env::var("SMTPUSERNAME") {
        Ok(v) => v,
        Err(e) => {
            return Err(Error::DotEnvUsername(e));
        }
    };
    let password = match env::var("SMTPPASSWORD") {
        Ok(v) => v,
        Err(e) => {
            return Err(Error::DotEnvPassword(e));
        }
    };
    let relay = match env::var("SMTPRELAY") {
        Ok(v) => v,
        Err(e) => {
            return Err(Error::DotEnvRelay(e));
        }
    };
    reporter.smtp(username, password, relay, dst)?;
    Ok(())
}
