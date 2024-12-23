use super::runtime::{RuntimeError, RuntimeInfo, RuntimeInfoStorage, State};
use fs2::FileExt;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, Write};
use std::path::PathBuf;

#[derive(Debug)]
pub struct FileHandler {
    file: File,
}

impl RuntimeInfoStorage for FileHandler {
    fn read(&mut self) -> Result<RuntimeInfo, RuntimeError> {
        let mut content = String::new();
        self.file
            .read_to_string(&mut content)
            .map_err(RuntimeError::FileRead)?;
        if content.trim().is_empty() {
            // We assume that the file is empty means that it is the first execution.
            let process_infos = [None, None, None, None];
            return Ok(RuntimeInfo {
                state: State::Init,
                process_infos,
                exec_path: std::env::current_exe().map_err(RuntimeError::FailedCurrentExe)?,
            });
        }
        serde_json::from_str(&content).map_err(RuntimeError::JsonDeserialize)
    }

    fn apply_with_lock<F>(&mut self, operation: F) -> Result<(), RuntimeError>
    where
        F: FnOnce(&mut RuntimeInfo) -> Result<(), RuntimeError>,
    {
        self.file
            .lock_exclusive()
            .map_err(self.handle_err(RuntimeError::FileLock))?;

        let mut runtime_info = self.read().map_err(self.handle_err_id())?;

        operation(&mut runtime_info).map_err(self.handle_err_id())?;

        self.write_locked(&runtime_info)
            .map_err(self.handle_err_id())?;
        self.file.unlock().map_err(RuntimeError::FileUnlock)?;

        Ok(())
    }
}

impl FileHandler {
    pub fn new(path: PathBuf) -> Result<Self, RuntimeError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .map_err(RuntimeError::FileOpen)?;
        Ok(FileHandler { file })
    }

    fn handle_err_id(&mut self) -> impl Fn(RuntimeError) -> RuntimeError + '_ {
        self.handle_err(|x| x)
    }

    fn handle_err<'a, E>(
        &'a mut self,
        error: impl Fn(E) -> RuntimeError + 'a,
    ) -> impl Fn(E) -> RuntimeError + 'a {
        move |e| {
            let res = self.file.unlock().map_err(RuntimeError::FileUnlock);
            if let Err(res) = res {
                return res;
            }
            error(e)
        }
    }

    fn write_locked(&mut self, runtime_info: &RuntimeInfo) -> Result<(), RuntimeError> {
        let json_data =
            serde_json::to_string_pretty(runtime_info).map_err(RuntimeError::JsonSerialize)?;

        self.file.set_len(0).map_err(RuntimeError::FileWrite)?;

        self.file
            .seek(std::io::SeekFrom::Start(0))
            .map_err(RuntimeError::FileWrite)?;

        self.file
            .write_all(json_data.as_bytes())
            .map_err(RuntimeError::FileWrite)?;

        log::info!("File written successfully");
        Ok(())
    }
}
