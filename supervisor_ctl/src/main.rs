mod model;
mod supervisor_ctl;

use daemonize::Result;
use supervisor_ctl::supervisor_ctl;

fn main() -> Result<()> {
    supervisor_ctl()
}
