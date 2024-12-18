pub mod tasks;
use crate::managers::runtime::ProcessInfo;
use crate::managers::{
    agent::{AgentManagerError, AgentManagerTrait},
    resource::{ResourceError, ResourceManagerTrait},
    runtime::{FeatType, RuntimeError, RuntimeInfoStorage, RuntimeManager, State},
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
    #[error("agent operation failed: {0}")]
    AgentError(#[from] AgentManagerError),
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

fn launch_new_version_agent<'a, A, H>(
    agent_manager: &'a mut A,
    runtime_manager: &'a mut RuntimeManager<H>,
) -> Result<ProcessInfo, UpdateError>
where
    A: AgentManagerTrait,
    H: RuntimeInfoStorage,
{
    let process_info = agent_manager.launch_agent(false)?;
    runtime_manager.add_process_info(process_info.clone())?;

    Ok(process_info)
}

async fn terminate_old_version_agent<'a, A, H>(
    agent_manager: &'a mut A,
    runtime_manager: &'a mut RuntimeManager<H>,
    latest: ProcessInfo,
) -> Result<(), UpdateError>
where
    A: AgentManagerTrait,
    H: RuntimeInfoStorage,
{
    let agent_processes = runtime_manager.filter_process_infos(FeatType::Agent)?;

    for agent_process in agent_processes {
        if agent_process.process_id == latest.process_id {
            continue;
        }
        agent_manager.terminate_agent(agent_process.process_id)?;
    }

    Ok(())
}

async fn monitor_agent_version<'a, A>(
    agent_manager: &'a A,
    expected_version: &Version,
) -> Result<(), UpdateError>
where
    A: AgentManagerTrait,
{
    let timeout = Duration::from_secs(180);
    let interval = Duration::from_secs(3);

    let start = Instant::now();
    let mut interval_timer = time::interval(interval);

    while start.elapsed() < timeout {
        interval_timer.tick().await;

        let version = agent_manager.get_version().await.map_err(|e| {
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

pub async fn execute<'a, A, R, H>(
    agent_manager: &'a mut A,
    resource_manager: &'a R,
    runtime_manager: &'a mut RuntimeManager<H>,
) -> Result<(), UpdateError>
where
    A: AgentManagerTrait,
    R: ResourceManagerTrait,
    H: RuntimeInfoStorage,
{
    log::info!("Starting update");

    let res: Result<(), UpdateError> = async {
        let current_version = Version::parse(env!("CARGO_PKG_VERSION"))
            .map_err(|_| UpdateError::InvalidVersionFormat)?;

        if runtime_manager
            .filter_process_infos(FeatType::Agent)?
            .is_empty()
        {
            return Err(UpdateError::AgentNotRunning);
        }
        let bundles = resource_manager.collect_downloaded_bundles();
        let update_actions = parse_bundles(&bundles)?;
        let pending_update_actions =
            extract_pending_update_actions(&update_actions, &current_version)?;
        for action in pending_update_actions {
            action.handle()?;
        }
        let latest = launch_new_version_agent(agent_manager, runtime_manager)?;
        terminate_old_version_agent(agent_manager, runtime_manager, latest).await?;
        monitor_agent_version(agent_manager, &current_version).await?;
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
