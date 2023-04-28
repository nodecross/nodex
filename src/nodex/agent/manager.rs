use std::collections::HashMap;

#[allow(dead_code)]
pub struct Manager {
    sessions: HashMap<String, bool>
}

impl Manager {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Manager {
            sessions: HashMap::new()
        }
    }
}