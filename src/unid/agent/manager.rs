use std::collections::HashMap;

pub struct Manager {
    sessions: HashMap<String, bool>
}

impl Manager {
    pub fn new() -> Self {
        Manager {
            sessions: HashMap::new()
        }
    }
}