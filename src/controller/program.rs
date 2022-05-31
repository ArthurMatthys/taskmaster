use crate::model::Program;

impl Program {
    pub fn kill(&self) {
        // pub fn kill(&self) -> Result<(), ()> {
        todo!()
    }

    pub fn start(&self) {
        todo!()
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

    pub fn action_fn(&self, action: &str) {
        match action {
            "start" => self.start(),
            "stop" => self.stop(),
            "relaunch" => self.relaunch(),
            _ => unimplemented!(),
        }
    }
}
