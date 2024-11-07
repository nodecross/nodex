use crate::state::resource::ResourceManager;

#[derive(Debug, thiserror::Error)]
pub enum RollbackError {
    #[error("Failed to find backup")]
    BackupNotFound,
    #[error("Failed to perform rollback: {0}")]
    RollbackFailed(#[from] std::io::Error),
}

pub struct RollbackState {
    resource_manager: ResourceManager,
}

impl RollbackState {
    pub fn new(resource_manager: ResourceManager) -> Self {
        Self { resource_manager }
    }
    pub fn handle(
        &self,
    ) -> Result<(), RollbackError> {
        let latest_backup = self.resource_manager.get_latest_backup();
        match latest_backup {
            Some(backup_file) => {
                self.resource_manager.rollback(&backup_file)?;
                Ok(())
            }
            None => Err(RollbackError::BackupNotFound),
        }
    }
}
