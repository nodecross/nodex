mod actions;

use std::fs;
use std::error::Error;
use crate::state::updating::actions::UpdateYaml;
use crate::state::updating::actions::run_actions;

pub struct UpdatingState;

impl UpdatingState {
    pub fn handle(&self) -> Result<(), Box<dyn Error>> {
        println!("Updating");
        println!("downloading update bundles");
        println!("filtering bundles");
        let yaml_content = fs::read_to_string("example.yaml")?;
        let update_yaml: UpdateYaml = serde_yaml::from_str(&yaml_content)?;
        run_actions(update_yaml.steps)?;

        println!("downloading binary");

        Ok(())
    }
}
