use crate::model::Program;
// use crate::model::{Error, Programs, Result};
// use logger::{log, LogInfo};

impl Program {

    // pub fn reconcile_state(&mut self, new_program: Program) -> Result<()> {
    //     let current_num_procs = self.children.len();
    //     let new_num_procs = new_program.num_procs as usize;

    //     if new_num_procs < current_num_procs {
    //         // Stop excess processes
    //         for _ in new_num_procs..current_num_procs {
    //             self.stop_process()?;
    //         }
    //     } else if new_num_procs > current_num_procs {
    //         // Start additional processes
    //         for _ in current_num_procs..new_num_procs {
    //             self.start_process()?;
    //         }
    //     }

    //     // If the configuration has changed, restart all processes
    //     if *self != new_program {
    //         self.restart_all_processes()?;
    //     }

    //     Ok(())
    // }
}
