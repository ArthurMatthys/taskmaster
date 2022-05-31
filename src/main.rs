mod controller;
mod model;
mod supervisor;

use clap::Parser;
use model::Args;

use crate::model::Result;
use crate::supervisor::supervisor;

fn main() -> Result<()> {
    let args = Args::parse();
    supervisor(args)
}
