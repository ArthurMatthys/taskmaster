use crate::Program;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Default)]
pub struct Programs {
    pub programs: HashMap<String, Program>,
}
