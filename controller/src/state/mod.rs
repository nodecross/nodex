pub mod handler;
mod init;
pub mod rollback;
pub mod update;

#[cfg(all(test, unix))]
mod tests {
    use crate::managers::{
        resource::{ResourceError, ResourceManagerTrait},
        runtime::{
            FeatType, ProcessInfo, RuntimeError, RuntimeInfo, RuntimeManager,
            RuntimeManagerWithoutAsync, State,
        },
    };
    use chrono::{FixedOffset, Utc};
    use semver::Version;
    use std::path::{Path, PathBuf};
    use std::sync::Mutex as StdMutex;

    pub struct MockRuntimeManager {
        pub response_version: Version,
        pub runtime_info: RuntimeInfo,
    }

    impl MockRuntimeManager {
        pub fn new(runtime_info: RuntimeInfo) -> Self {
            let current_version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
            Self {
                response_version: current_version,
                runtime_info,
            }
        }
    }

    impl RuntimeManagerWithoutAsync for MockRuntimeManager {
        fn launch_agent(&mut self, _is_first: bool) -> Result<ProcessInfo, RuntimeError> {
            let now = Utc::now().with_timezone(&FixedOffset::east_opt(9 * 3600).unwrap());
            let process_info = ProcessInfo {
                process_id: 1,
                feat_type: FeatType::Agent,
                version: self.response_version.clone(),
                executed_at: now,
            };
            let _ = self.runtime_info.add_process_info(process_info.clone());
            Ok(process_info)
        }

        fn launch_controller(
            &mut self,
            _new_controller_path: impl AsRef<Path>,
        ) -> Result<(), RuntimeError> {
            Ok(())
        }

        fn get_runtime_info(&mut self) -> Result<RuntimeInfo, RuntimeError> {
            Ok(self.runtime_info.clone())
        }

        fn update_state_without_send(&mut self, state: State) -> Result<(), RuntimeError> {
            self.runtime_info.state = state;
            Ok(())
        }

        fn update_state(&mut self, state: State) -> Result<(), RuntimeError> {
            self.runtime_info.state = state;
            Ok(())
        }

        fn kill_process(&mut self, _process_info: &ProcessInfo) -> Result<(), RuntimeError> {
            unimplemented!();
        }

        fn kill_other_agents(&mut self, _target: u32) -> Result<(), RuntimeError> {
            for p in self
                .runtime_info
                .process_infos
                .iter_mut()
                .filter(|p| p.as_ref().map(|q| &q.version) != Some(&self.response_version))
            {
                *p = None;
            }
            Ok(())
        }
    }

    impl RuntimeManager for MockRuntimeManager {
        async fn get_version(&self) -> Result<Version, RuntimeError> {
            Ok(self.response_version.clone())
        }
    }

    pub struct MockResourceManager {
        bundles: Vec<PathBuf>,
        pub rollback_called: StdMutex<bool>,
        pub remove_called: StdMutex<bool>,
    }

    impl MockResourceManager {
        pub fn new(bundles: Vec<PathBuf>) -> Self {
            Self {
                bundles,
                remove_called: StdMutex::new(false),
                rollback_called: StdMutex::new(false),
            }
        }
    }

    impl ResourceManagerTrait for MockResourceManager {
        fn backup(&self) -> Result<(), ResourceError> {
            unimplemented!()
        }

        fn rollback(&self, _backup_file: &std::path::Path) -> Result<(), ResourceError> {
            let mut called = self.rollback_called.lock().unwrap();
            *called = true;
            Ok(())
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
            self.bundles.first().cloned()
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
}
