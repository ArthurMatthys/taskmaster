use crate::model::Program;
// use daemonize::{log, LogInfo};

impl Program {
    pub fn kill(&mut self) {
        // match &mut self.child {
        //     Some(child) => {
        //         child.kill().unwrap();
        //     }
        //     None => unreachable!(),
        // }
    }

    fn start_one(&mut self, vec: &mut Vec<Program>) {
        match std::process::Command::new(&self.cmd).spawn() {
            Ok(child) => {
                self.count = 0;
                daemonize::log(format!("Program {} started", self.cmd), LogInfo::Info);
                vec.push(child)
            }
            Err(e) => {
                self.count += 1;
                daemonize::log(
                    format!("Error while starting program: {}", e),
                    LogInfo::Error,
                );
                None
            }
        }
    }

    pub fn start(&mut self) {
        let mut vec = Vec::with_capacity(self.num_procs as usize);

        for _ in 0..self.num_procs {
            self.start_one(&mut vec)
        }
    }

    pub fn stop(&self) {
        todo!()
    }

    pub fn relaunch(&self) {
        todo!()
    }
    pub fn status(&self, name: &str) {
        println!("-------------");
        println!("{name}:");
        println!("-------------");
    }

    pub fn action_fn(&mut self, action: &str) {
        match action {
            "start" => self.start(),
            "stop" => self.stop(),
            "relaunch" => self.relaunch(),
            _ => unimplemented!(),
        }
    }
}
