use super::runtime::{NodexSignal, ProcessManager};
use std::path::Path;

#[derive(Clone)]
pub struct WindowsProcessManager;

impl ProcessManager for WindowsProcessManager {
    fn is_running(&self, process_id: u32) -> bool {
        unimplemented!()
    }
    fn spawn_process(&self, cmd: impl AsRef<Path>, args: &[&str]) -> Result<u32, std::io::Error> {
        unimplemented!()
    }
    fn kill_process(&self, process_id: u32, signal: NodexSignal) -> Result<(), std::io::Error> {
        unimplemented!()
    }
}
