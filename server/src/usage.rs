use clap::Parser;

#[derive(Parser, Debug)]
pub struct MattDaemonArgs {
    /// Tell the email adress you want to send the logfile
    /// Must have the following format : "name <email@address>"
    #[clap(short)]
    pub mail_send: Option<String>,
}
