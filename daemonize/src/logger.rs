use std::{fmt::Display, fs, io::Write, path::PathBuf};

use crate::error::{Error, Result};
use chrono::offset::Local;
use lettre::{
    message::{header::ContentType, Attachment, MessageBuilder},
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};

// const LOGFILE: PathBuf = PathBuf::from("/var/log/matt_daemon/matt_daemon.log");

pub enum LogInfo {
    Debug,
    Error,
    Info,
    Warn,
}

impl Display for LogInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogInfo::Debug => write!(f, "\x1B[34mDEBUG\x1B[0m"),
            LogInfo::Error => write!(f, "\x1B[31mERROR\x1B[0m"),
            LogInfo::Info => write!(f, "\x1B[33mINFO\x1B[0m"),
            LogInfo::Warn => write!(f, "\x1B[35mWarn\x1B[0m"),
        }
    }
}

impl LogInfo {
    fn is_debug(&self) -> bool {
        matches!(self, LogInfo::Debug)
    }
}

#[derive(Clone, Debug)]
pub struct MailConfig {
    username: String,
    password: String,
    relay: String,
    dst: MessageBuilder,
    mail_addr: String,
}

#[derive(Clone)]
pub struct TintinReporter {
    pub logfile: PathBuf,
    pub mail: Option<MailConfig>,
}

impl Default for TintinReporter {
    fn default() -> Self {
        Self {
            logfile: PathBuf::from("/var/log/matt_daemon/matt_daemon.log"),
            mail: None,
        }
    }
}

impl TintinReporter {
    pub fn smtp(
        &mut self,
        username: String,
        password: String,
        relay: String,
        dst: String,
    ) -> Result<()> {
        let mail = Message::builder()
            .from(
                "Matt Daemon <matt@daemon.amatthys.gurival.student.42lyon.fr>"
                    .parse()
                    .map_err(|_| Error::ParseError)?,
            )
            .to(dst.parse().map_err(|_| Error::ParseDstError)?);
        self.mail = Some(MailConfig {
            username,
            password,
            relay,
            dst: mail,
            mail_addr: dst,
        });

        Ok(())
    }

    pub fn logfile(mut self, logfile: String) -> Self {
        self.logfile = PathBuf::from(logfile);
        self
    }

    pub fn send_mail(&self) -> Result<()> {
        if self.mail.is_none() {
            return Ok(());
        }

        let mail_config = match &self.mail {
            None => return Ok(()),
            Some(config) => config,
        };

        let smtp_credentials =
            Credentials::new(mail_config.username.clone(), mail_config.password.clone());
        let client = SmtpTransport::relay(&mail_config.relay)
            .map_err(Error::MailSmtpTransport)?
            .credentials(smtp_credentials)
            .build();

        let filebody = fs::read(self.logfile.clone()).map_err(Error::ReadFile)?;
        let attachment = Attachment::new(
            self.logfile
                .to_str()
                .ok_or(Error::ConvertToUTF8)?
                .to_string(),
        )
        .body(filebody, ContentType::TEXT_PLAIN);

        let mail = &mail_config
            .dst
            .clone()
            .subject("Recap Matt Daemon")
            .singlepart(attachment)
            .map_err(Error::MailBuilder)?;
        let mail_addr = &mail_config.mail_addr;

        self.log(
            format!("Sending a recap mail to {mail_addr}\n"),
            LogInfo::Info,
            false,
        )?;
        client.send(mail).map_err(Error::MailSend)?;

        Ok(())
    }
    pub fn log<S>(&self, msg: S, info: LogInfo, debug: bool) -> Result<()>
    where
        S: Display,
    {
        if !debug && info.is_debug() {
            return Ok(());
        }
        fs::create_dir_all("/var/log/matt_daemon").map_err(Error::CreateDir)?;
        let mut f = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&self.logfile)
            .map_err(Error::LogOpen)?;

        let now = Local::now().format("%d / %m / %Y - %H : %M : %S");
        f.write(format!("[{now:}] - {info:5} : {msg}").as_bytes())
            .map_err(Error::Log)?;
        Ok(())
    }
}
