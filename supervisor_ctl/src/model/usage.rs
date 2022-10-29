use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    /// Path to the fiel containing the initial config
    pub path: String,
}
