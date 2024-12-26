pub mod tasks;
use crate::managers::{
    resource::{ResourceError, ResourceManagerTrait},
    runtime::{FeatType, RuntimeError, RuntimeManager, State},
};
use crate::state::update::tasks::{UpdateAction, UpdateActionError};
use semver::Version;
use serde_yaml::Error as SerdeYamlError;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::{self, Instant};

#[derive(Debug, thiserror::Error)]
pub enum UpdateError {
    #[error("Failed to find bundle")]
    BundleNotFound,
    #[error("Invalid version format")]
    InvalidVersionFormat,
    #[error("update action error: {0}")]
    UpdateActionFailed(#[from] UpdateActionError),
    #[error("Failed to read YAML file: {0}")]
    YamlReadFailed(#[from] std::io::Error),
    #[error("Failed to parse YAML: {0}")]
    YamlParseFailed(#[source] SerdeYamlError),
    #[error("Failed to update state: {0}")]
    UpdateStateFailed(#[source] RuntimeError),
    #[error("Failed to Agent version check: {0}")]
    AgentVersionCheckFailed(String),
    #[error("runtime operation failed: {0}")]
    RuntimeError(#[from] RuntimeError),
    #[error("resource operation failed: {0}")]
    ResourceError(#[from] ResourceError),
    #[error("Agent not running")]
    AgentNotRunning,
}

impl UpdateError {
    pub fn required_restore_state(&self) -> bool {
        !matches!(self, UpdateError::AgentNotRunning)
    }

    pub fn requires_rollback(&self) -> bool {
        !matches!(
            self,
            UpdateError::ResourceError(ResourceError::RemoveFailed(_))
        )
    }
}
fn get_target_state(update_error: &UpdateError) -> Option<State> {
    if update_error.requires_rollback() {
        Some(State::Rollback)
    } else if update_error.required_restore_state() {
        Some(State::Idle)
    } else {
        None
    }
}

fn parse_bundles(bundles: &[PathBuf]) -> Result<Vec<UpdateAction>, UpdateError> {
    bundles
        .iter()
        .map(|bundle| {
            let yaml_content = fs::read_to_string(bundle)?;
            let update_action: UpdateAction =
                serde_yaml::from_str(&yaml_content).map_err(UpdateError::YamlParseFailed)?;
            Ok(update_action)
        })
        .collect()
}

fn extract_pending_update_actions<'b>(
    update_actions: &'b [UpdateAction],
    current_controller_version: &Version,
    current_agent_version: &Version,
) -> Result<Vec<&'b UpdateAction>, UpdateError> {
    let pending_actions: Vec<&'b UpdateAction> = update_actions
        .iter()
        .filter_map(|action| {
            let target_version = Version::parse(&action.version).ok()?;
            if *current_controller_version >= target_version
                && target_version > *current_agent_version
            {
                Some(action)
            } else {
                None
            }
        })
        .collect();

    Ok(pending_actions)
}

async fn monitor_agent_version<'a, R: RuntimeManager>(
    runtime_manager: &'a R,
    expected_version: &Version,
) -> Result<(), UpdateError> {
    let timeout = Duration::from_secs(180);
    let interval = Duration::from_secs(3);

    let start = Instant::now();
    let mut interval_timer = time::interval(interval);

    while start.elapsed() < timeout {
        interval_timer.tick().await;

        let version = runtime_manager.get_version().await.map_err(|e| {
            log::error!("Error occurred during version check: {}", e);
            UpdateError::AgentVersionCheckFailed(e.to_string())
        })?;

        if version == *expected_version {
            log::info!("Expected version received: {}", expected_version);
            return Ok(());
        } else {
            log::info!("Version did not match expected value.");
        }
    }

    Err(UpdateError::AgentVersionCheckFailed(format!(
        "Expected version '{}' was not received within {:?}.",
        expected_version, timeout
    )))
}

pub async fn execute<'a, R, T>(
    resource_manager: &'a R,
    runtime_manager: &'a mut T,
) -> Result<(), UpdateError>
where
    R: ResourceManagerTrait,
    T: RuntimeManager,
{
    log::info!("Starting update");

    let res: Result<(), UpdateError> = async {
        let current_version = Version::parse(env!("CARGO_PKG_VERSION"))
            .map_err(|_| UpdateError::InvalidVersionFormat)?;
        let runtime_info = runtime_manager.get_runtime_info()?;
        if !runtime_info.is_agent_running() {
            return Err(UpdateError::AgentNotRunning);
        }
        let current_running_agent = runtime_info.filter_by_feat(FeatType::Agent).next().unwrap();
        let bundles = resource_manager.collect_downloaded_bundles();
        let update_actions = parse_bundles(&bundles)?;
        let pending_update_actions = extract_pending_update_actions(
            &update_actions,
            &current_version,
            &current_running_agent.version,
        )?;
        for action in pending_update_actions {
            action.handle()?;
        }
        // launch new version agent
        let latest = runtime_manager.launch_agent(false)?;
        // terminate old version agents
        runtime_manager.kill_other_agents(latest.process_id)?;
        monitor_agent_version(runtime_manager, &current_version).await?;
        // if you test for rollback, comment out a follow line.
        resource_manager.remove()?;
        Ok(())
    }
    .await;

    match res {
        Ok(()) => runtime_manager.update_state(crate::managers::runtime::State::Idle)?,
        Err(update_error) => {
            if let Some(target_state) = get_target_state(&update_error) {
                runtime_manager.update_state(target_state)?;
            }
            return Err(update_error);
        }
    }

    log::info!("Update completed");

    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::super::tests::{MockResourceManager, MockRuntimeManager};
    use super::*;
    use crate::managers::runtime::{FeatType, ProcessInfo, RuntimeInfo, State};
    use crate::state::update::tasks::{Task, UpdateAction};
    use chrono::{FixedOffset, Utc};
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::{tempdir, TempDir};

    #[tokio::test]
    async fn test_execute_with_empty_bundles() {
        let current_version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
        let runtime_info = RuntimeInfo {
            state: State::Update,
            process_infos: [
                Some(ProcessInfo {
                    process_id: 2,
                    feat_type: FeatType::Controller,
                    version: current_version.clone(),
                    executed_at: Utc::now()
                        .with_timezone(&FixedOffset::east_opt(9 * 3600).unwrap()),
                }),
                Some(ProcessInfo {
                    process_id: 3,
                    feat_type: FeatType::Agent,
                    version: Version::parse("0.0.1").unwrap(),
                    executed_at: Utc::now()
                        .with_timezone(&FixedOffset::east_opt(9 * 3600).unwrap()),
                }),
                None,
                None,
            ],
            exec_path: "".into(),
        };
        let mut runtime = MockRuntimeManager {
            response_version: current_version.clone(),
            runtime_info,
        };
        let resource = MockResourceManager::new(vec![]);

        let result = execute(&resource, &mut runtime).await;
        assert!(result.is_ok(), "Update should succeed");

        let runtime_info: Vec<_> = runtime
            .runtime_info
            .process_infos
            .iter()
            .flatten()
            .collect();
        assert_eq!(runtime_info.len(), 2);
        assert_eq!(runtime_info[0].process_id, 2);
        assert_eq!(runtime_info[0].feat_type, FeatType::Controller);
        assert_eq!(runtime_info[0].version, current_version.clone());
        assert_eq!(runtime_info[1].process_id, 1);
        assert_eq!(runtime_info[1].feat_type, FeatType::Agent);
        assert_eq!(runtime_info[1].version, current_version);
    }

    fn create_test_file(path: &str, content: &str) -> std::io::Result<()> {
        let mut file = fs::File::create(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_with_bundles() {
        let current_version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
        let runtime_info = RuntimeInfo {
            state: State::Update,
            process_infos: [
                Some(ProcessInfo {
                    process_id: 2,
                    feat_type: FeatType::Controller,
                    version: current_version.clone(),
                    executed_at: Utc::now()
                        .with_timezone(&FixedOffset::east_opt(9 * 3600).unwrap()),
                }),
                Some(ProcessInfo {
                    process_id: 3,
                    feat_type: FeatType::Agent,
                    version: Version::parse("0.0.1").unwrap(),
                    executed_at: Utc::now()
                        .with_timezone(&FixedOffset::east_opt(9 * 3600).unwrap()),
                }),
                None,
                None,
            ],
            exec_path: "".into(),
        };

        let mut runtime = MockRuntimeManager {
            response_version: current_version.clone(),
            runtime_info,
        };

        // setup bundles
        let source_path = "/tmp/source.txt";
        create_test_file(source_path, "This is source1").expect("Failed to create source1.txt");
        let dest_path = "/tmp/dest";

        let tasks = vec![Task::Move {
            description: "Move file".to_string(),
            src: source_path.to_string(),
            dest: dest_path.to_string(),
        }];

        let action = UpdateAction {
            version: current_version.to_string(),
            description: "Test move tasks".to_string(),
            tasks,
        };

        let _temp_dir = tempdir().expect("Failed to create temporary directory");
        let yaml_str = serde_yaml::to_string(&action).expect("Failed to serialize action to YAML");
        let bundle_path = _temp_dir.path().join("test_bundle.yaml");
        fs::write(&bundle_path, &yaml_str).expect("Failed to write YAML to file");

        let resource = MockResourceManager::new(vec![bundle_path]);

        let result = execute(&resource, &mut runtime).await;
        assert!(result.is_ok(), "Update should succeed");
    }

    #[tokio::test]
    async fn test_execute_without_running_agent() {
        let current_version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
        let runtime_info = RuntimeInfo {
            state: State::Update,
            process_infos: [None, None, None, None],
            exec_path: "".into(),
        };
        let mut runtime = MockRuntimeManager {
            response_version: current_version,
            runtime_info,
        };
        let resource = MockResourceManager::new(vec![]);

        let result = execute(&resource, &mut runtime).await;
        assert!(
            matches!(result, Err(UpdateError::AgentNotRunning)),
            "Should fail with AgentNotRunning"
        );
    }

    #[tokio::test]
    async fn test_extract_pending_update_actions() {
        let current_version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
        let _temp_dir = tempdir().expect("Failed to create temporary directory");

        fn setup_bundle(temp_dir: &TempDir, file_name: &str, version: String) -> PathBuf {
            let source_path = "/tmp/source.txt";
            create_test_file(source_path, "This is source1").expect("Failed to create source1.txt");
            let dest_path = "/tmp/dest";

            let tasks = vec![Task::Move {
                description: "Move file".to_string(),
                src: source_path.to_string(),
                dest: dest_path.to_string(),
            }];

            let action = UpdateAction {
                version,
                description: "Test move tasks".to_string(),
                tasks,
            };

            let yaml_str =
                serde_yaml::to_string(&action).expect("Failed to serialize action to YAML");
            let bundle_path = temp_dir.path().join(file_name);
            fs::write(&bundle_path, &yaml_str).expect("Failed to write YAML to file");

            bundle_path
        }
        let agent_version = Version::parse("1.0.0").unwrap();
        let bundle1 = setup_bundle(&_temp_dir, "bundle1.yml", current_version.to_string());
        let mut cloned_current_version = current_version.clone();
        cloned_current_version.patch += 1;
        let bundle2 = setup_bundle(
            &_temp_dir,
            "bundle2.yml",
            cloned_current_version.to_string(),
        );
        let bundle3 = setup_bundle(&_temp_dir, "bundle3.yml", agent_version.to_string());
        let bundle4 = setup_bundle(&_temp_dir, "bundle4.yml", "1.5.0".to_string());

        let bundles = vec![bundle1, bundle2, bundle3, bundle4];

        let update_actions = parse_bundles(&bundles).unwrap();
        let result =
            extract_pending_update_actions(&update_actions, &current_version, &agent_version);

        assert!(result.is_ok(), "Update should succeed");
        let pending_update_actions = result.unwrap();
        assert!(
            pending_update_actions.len() == 2,
            "Update should have one action"
        );

        let expected_versions = [current_version.to_string(), "1.5.0".to_string()];
        assert!(expected_versions.contains(&pending_update_actions[0].version));
        assert!(expected_versions.contains(&pending_update_actions[1].version));
    }
}
