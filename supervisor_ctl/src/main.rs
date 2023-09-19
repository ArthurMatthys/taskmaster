mod model;
mod supervisor_ctl;

use clap::Parser;
use model::Args;

use crate::supervisor_ctl::supervisor;
use supervisor::Result;

fn main() -> Result<()> {
    let args = Args::parse();
    supervisor(args)
}
