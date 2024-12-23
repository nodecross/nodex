pub mod tasks;
use crate::managers::{
    resource::{ResourceError, ResourceManagerTrait},
    runtime::{RuntimeError, RuntimeManager, State},
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
        Some(State::Init)
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
    current_version: &Version,
) -> Result<Vec<&'b UpdateAction>, UpdateError> {
    let pending_actions: Vec<&'b UpdateAction> = update_actions
        .iter()
        .filter_map(|action| {
            let target_version = Version::parse(&action.version).ok()?;
            if target_version > *current_version {
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

        if version == expected_version.to_string() {
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

        if !runtime_manager.is_agent_running()? {
            return Err(UpdateError::AgentNotRunning);
        }
        let bundles = resource_manager.collect_downloaded_bundles();
        let update_actions = parse_bundles(&bundles)?;
        let pending_update_actions =
            extract_pending_update_actions(&update_actions, &current_version)?;
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
        }
    }

    log::info!("Update completed");

    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::managers::file_storage::FileHandler;
    use crate::managers::{
        resource::ResourceManagerTrait,
        runtime::{FeatType, ProcessInfo, RuntimeInfo, RuntimeManager, State},
    };
    use crate::state::update::tasks::{Task, UpdateAction};
    use chrono::{FixedOffset, Utc};
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex as StdMutex};
    use tempfile::{tempdir, TempDir};
    use tokio::sync::Mutex;

    struct MockAgentManager {
        response_version: String,
    }

    impl AgentManagerTrait for MockAgentManager {
        fn launch_agent(&self) -> Result<ProcessInfo, AgentManagerError> {
            let now = Utc::now().with_timezone(&FixedOffset::east_opt(9 * 3600).unwrap());
            let process_info = ProcessInfo {
                process_id: 1,
                feat_type: FeatType::Agent,
                version: self.response_version.clone(),
                executed_at: now,
            };
            Ok(process_info)
        }
        fn terminate_agent(&self, _process_id: u32) -> Result<(), AgentManagerError> {
            Ok(())
        }

        async fn get_request<T>(&self, _path: &str) -> Result<T, AgentManagerError>
        where
            T: serde::de::DeserializeOwned + Send,
        {
            if _path == "/internal/version/get" {
                let response = serde_json::json!({
                    "version": self.response_version.clone(),
                });
                let json_response = serde_json::to_string(&response).unwrap();
                let deserialized: T = serde_json::from_str(&json_response).unwrap();
                Ok(deserialized)
            } else {
                Err(AgentManagerError::RequestFailed("Invalid path".into()))
            }
        }

        fn cleanup(&self) -> Result<(), std::io::Error> {
            Ok(())
        }
    }

    pub struct MockResourceManager {
        bundles: Vec<PathBuf>,
        remove_called: StdMutex<bool>,
    }

    impl MockResourceManager {
        pub fn new(bundles: Vec<PathBuf>) -> Self {
            Self {
                bundles,
                remove_called: StdMutex::new(false),
            }
        }
    }

    impl ResourceManagerTrait for MockResourceManager {
        fn backup(&self) -> Result<(), ResourceError> {
            unimplemented!()
        }

        fn rollback(&self, _backup_file: &std::path::Path) -> Result<(), ResourceError> {
            unimplemented!()
        }

        fn agent_path(&self) -> &PathBuf {
            unimplemented!()
        }

        fn tmp_path(&self) -> &PathBuf {
            unimplemented!()
        }

        fn get_paths_to_backup(&self) -> Result<Vec<PathBuf>, ResourceError> {
            unimplemented!()
        }

        fn collect_downloaded_bundles(&self) -> Vec<PathBuf> {
            self.bundles.clone()
        }

        fn get_latest_backup(&self) -> Option<PathBuf> {
            unimplemented!()
        }

        fn extract_zip(
            &self,
            _archive_data: bytes::Bytes,
            _output_path: &std::path::Path,
        ) -> Result<(), ResourceError> {
            unimplemented!()
        }

        fn remove_directory(&self, _path: &std::path::Path) -> Result<(), std::io::Error> {
            Ok(())
        }

        fn remove(&self) -> Result<(), ResourceError> {
            let mut called = self.remove_called.lock().unwrap();
            *called = true;
            Ok(())
        }
    }

    fn setup_temp_file() -> (RuntimeManager, tempfile::TempDir, PathBuf) {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let temp_file_path = temp_dir.path().join("runtime_info.json");

        File::create(&temp_file_path).expect("Failed to create temporary runtime_info.json");

        assert!(
            temp_file_path.exists(),
            "Temporary file was not created: {:?}",
            temp_file_path
        );

        let file_handler = FileHandler::new(temp_file_path.clone());
        let runtime_manager = RuntimeManager::new(file_handler);

        (runtime_manager, temp_dir, temp_file_path)
    }

    #[tokio::test]
    async fn test_execute_with_empty_bundles() {
        let current_version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
        let agent = Arc::new(Mutex::new(MockAgentManager {
            response_version: current_version.to_string(),
        }));
        let resource = MockResourceManager::new(vec![]);

        // setup runtime_info.json
        let (runtime, _temp_dir, temp_file_path) = setup_temp_file();
        let initial_runtime_info = RuntimeInfo {
            state: State::Updating,
            process_infos: vec![
                ProcessInfo {
                    process_id: 2,
                    feat_type: FeatType::Controller,
                    version: current_version.to_string(),
                    executed_at: Utc::now()
                        .with_timezone(&FixedOffset::east_opt(9 * 3600).unwrap()),
                },
                ProcessInfo {
                    process_id: 3,
                    feat_type: FeatType::Agent,
                    version: "1.0.0".to_string(),
                    executed_at: Utc::now()
                        .with_timezone(&FixedOffset::east_opt(9 * 3600).unwrap()),
                },
            ],
        };

        let file_handler = FileHandler::new(temp_file_path.clone());

        file_handler
            .write_locked(
                &mut File::create(&temp_file_path).unwrap(),
                &initial_runtime_info,
            )
            .unwrap();

        let state: UpdateState<'_, MockAgentManager, MockResourceManager> =
            UpdateState::new(&agent, resource, &runtime);
        let result = state.execute().await;
        assert!(result.is_ok(), "Update should succeed");

        let runtime_info = runtime.get_process_infos().unwrap();
        assert_eq!(runtime_info.len(), 2);
        assert_eq!(runtime_info[0].process_id, 2);
        assert_eq!(runtime_info[0].feat_type, FeatType::Controller);
        assert_eq!(runtime_info[0].version, current_version.to_string());
        assert_eq!(runtime_info[1].process_id, 1);
        assert_eq!(runtime_info[1].feat_type, FeatType::Agent);
        assert_eq!(runtime_info[1].version, current_version.to_string());
    }

    fn create_test_file(path: &str, content: &str) -> std::io::Result<()> {
        let mut file = fs::File::create(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_with_bundles() {
        let current_version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
        let agent = Arc::new(Mutex::new(MockAgentManager {
            response_version: current_version.to_string(),
        }));
        let (runtime, _temp_dir, temp_file_path) = setup_temp_file();

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

        let yaml_str = serde_yaml::to_string(&action).expect("Failed to serialize action to YAML");
        let bundle_path = _temp_dir.path().join("test_bundle.yaml");
        fs::write(&bundle_path, &yaml_str).expect("Failed to write YAML to file");

        let resource = MockResourceManager::new(vec![bundle_path]);

        // setup runtime_info.json
        let initial_runtime_info = RuntimeInfo {
            state: State::Updating,
            process_infos: vec![
                ProcessInfo {
                    process_id: 2,
                    feat_type: FeatType::Controller,
                    version: current_version.to_string(),
                    executed_at: Utc::now()
                        .with_timezone(&FixedOffset::east_opt(9 * 3600).unwrap()),
                },
                ProcessInfo {
                    process_id: 3,
                    feat_type: FeatType::Agent,
                    version: "1.0.0".to_string(),
                    executed_at: Utc::now()
                        .with_timezone(&FixedOffset::east_opt(9 * 3600).unwrap()),
                },
            ],
            exec_path: std::env::current_exe().unwrap(),
        };
        let file_handler = FileHandler::new(temp_file_path.clone());
        file_handler
            .write_locked(
                &mut File::create(&temp_file_path).unwrap(),
                &initial_runtime_info,
            )
            .unwrap();

        let state = UpdateState::new(&agent, resource, &runtime);
        let result = state.execute().await;
        assert!(result.is_ok(), "Update should succeed");
    }

    #[tokio::test]
    async fn test_execute_without_running_agent() {
        let current_version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
        let agent = Arc::new(Mutex::new(MockAgentManager {
            response_version: current_version.to_string(),
        }));
        let resource = MockResourceManager::new(vec![]);

        let (runtime, _temp_dir, temp_file_path) = setup_temp_file();
        let initial_runtime_info = RuntimeInfo {
            state: State::Updating,
            process_infos: vec![],
            exec_path: std::env::current_exe().unwrap(),
        };
        let file_handler = FileHandler::new(temp_file_path.clone());
        file_handler
            .write_locked(
                &mut File::create(&temp_file_path).unwrap(),
                &initial_runtime_info,
            )
            .unwrap();

        let state = UpdateState::new(&agent, resource, &runtime);
        let result = state.execute().await;
        assert!(
            matches!(result, Err(UpdateError::AgentNotRunning)),
            "Should fail with AgentNotRunning"
        );
    }

    #[tokio::test]
    async fn test_extract_pending_update_actions() {
        let current_version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
        let agent = Arc::new(Mutex::new(MockAgentManager {
            response_version: current_version.to_string(),
        }));
        let (runtime, _temp_dir, temp_file_path) = setup_temp_file();

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
                version: version,
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
        let resource = MockResourceManager::new(bundles.clone());

        let state = UpdateState::new(&agent, resource, &runtime);
        let update_actions = state.parse_bundles(&bundles).unwrap();
        let result =
            state.extract_pending_update_actions(&update_actions, &current_version, &agent_version);

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
