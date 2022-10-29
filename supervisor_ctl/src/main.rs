mod model;
mod supervisor_ctl;

use clap::Parser;
use model::Args;

use supervisor::Result;
use crate::supervisor_ctl::supervisor;

fn main() -> Result<()> {
    let args = Args::parse();
    supervisor(args)
}
