use super::runtime::{RuntimeError, RuntimeInfo, RuntimeInfoStorage, State};
use nix::errno::Errno;
use nix::fcntl::OFlag;
use nix::sys::mman::{
    mlock, mmap, msync, munlock, munmap, shm_open, shm_unlink, MapFlags, MsFlags, ProtFlags,
};
use nix::sys::stat::Mode;
use nix::unistd::ftruncate;
use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

pub struct MmapHandler {
    name: PathBuf,
    ptr: core::ptr::NonNull<core::ffi::c_void>,
    len: core::num::NonZeroUsize,
}

impl Deref for MmapHandler {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr() as *const u8, self.len.into()) }
    }
}

impl DerefMut for MmapHandler {
    #[inline]
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr() as *mut u8, self.len.into()) }
    }
}

impl AsRef<[u8]> for MmapHandler {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.deref()
    }
}

impl AsMut<[u8]> for MmapHandler {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        self.deref_mut()
    }
}

impl std::fmt::Debug for MmapHandler {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("MmapHandler")
            .field("ptr", &self.ptr)
            .field("len", &self.len)
            .finish()
    }
}

unsafe impl Send for MmapHandler {}
unsafe impl Sync for MmapHandler {}

#[inline]
fn _e2e(e: Errno) -> std::io::Error {
    std::io::Error::from_raw_os_error(e as core::ffi::c_int)
}

impl MmapHandler {
    // ref: https://stackoverflow.com/questions/62320764/how-to-create-shared-memory-after-fork
    pub fn new(name: impl AsRef<Path>) -> Result<Self, RuntimeError> {
        // We assume that data is sufficiently small.
        let length = core::num::NonZero::new(10000).unwrap();
        // Open without creation
        let fd = shm_open(name.as_ref(), OFlag::O_RDWR, Mode::S_IRUSR | Mode::S_IWUSR);
        let fd = match fd {
            Ok(fd) => fd,
            Err(Errno::ENOENT) => {
                let fd = shm_open(
                    name.as_ref(),
                    OFlag::O_CREAT | OFlag::O_EXCL | OFlag::O_RDWR,
                    Mode::S_IRUSR | Mode::S_IWUSR,
                )
                .map_err(_e2e)
                .map_err(RuntimeError::FileOpen)?;
                // We must truncate size of shared memory at the time of initial creation.
                ftruncate(&fd, Into::<usize>::into(length) as i64)
                    .map_err(_e2e)
                    .map_err(RuntimeError::FileOpen)?;
                fd
            }
            Err(err) => return Err(RuntimeError::FileOpen(_e2e(err))),
        };
        let ptr = unsafe {
            mmap(
                None,
                length,
                ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
                MapFlags::MAP_NORESERVE | MapFlags::MAP_SHARED,
                fd,
                0,
            )
            .map_err(_e2e)
            .map_err(RuntimeError::FileOpen)?
        };
        Ok(MmapHandler {
            ptr,
            len: length,
            name: name.as_ref().to_path_buf(),
        })
    }

    pub fn close(self) -> Result<(), RuntimeError> {
        unsafe {
            munmap(self.ptr, self.len.into())
                .map_err(_e2e)
                .map_err(RuntimeError::FileRemove)?;
            shm_unlink(&self.name)
                .map_err(_e2e)
                .map_err(RuntimeError::FileRemove)?;
        }
        Ok(())
    }

    fn lock(&self) -> Result<(), RuntimeError> {
        unsafe {
            mlock(self.ptr, self.len.into())
                .map_err(_e2e)
                .map_err(RuntimeError::FileLock)
        }
    }

    fn unlock(&self) -> Result<(), RuntimeError> {
        unsafe {
            munlock(self.ptr, self.len.into())
                .map_err(_e2e)
                .map_err(RuntimeError::FileUnlock)
        }
    }

    fn flush(&self) -> Result<(), RuntimeError> {
        unsafe {
            msync(self.ptr, self.len.into(), MsFlags::MS_SYNC)
                .map_err(_e2e)
                .map_err(RuntimeError::FileWrite)
        }
    }

    fn handle_err<'a, E>(
        &'a mut self,
        error: impl Fn(E) -> RuntimeError + 'a,
    ) -> impl Fn(E) -> RuntimeError + 'a {
        move |e| {
            let res = self.unlock();
            if let Err(res) = res {
                return res;
            }
            error(e)
        }
    }

    fn handle_err_id(&mut self) -> impl Fn(RuntimeError) -> RuntimeError + '_ {
        self.handle_err(|x| x)
    }

    fn write_locked(&mut self, runtime_info: &RuntimeInfo) -> Result<(), RuntimeError> {
        let json_data = serde_json::to_string(runtime_info).map_err(RuntimeError::JsonSerialize)?;
        let mut json_data = json_data.into_bytes();
        json_data.push(0);
        (&mut self[..])
            .write(&json_data)
            .map_err(RuntimeError::FileWrite)?;
        self.flush()?;
        log::info!("File written successfully");
        Ok(())
    }
}

impl RuntimeInfoStorage for MmapHandler {
    fn read(&mut self) -> Result<RuntimeInfo, RuntimeError> {
        self.lock()?;
        let cstr = std::ffi::CStr::from_bytes_until_nul(self)
            .ok()
            .and_then(|s| s.to_str().ok())
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to read runtime info",
            ))
            .map_err(RuntimeError::FileRead);
        self.unlock()?;
        let cstr = cstr?.trim();
        if cstr.is_empty() {
            // We assume that memmap is empty means that it is the first execution.
            let process_infos = [None, None, None, None];
            return Ok(RuntimeInfo {
                state: State::Init,
                process_infos,
                exec_path: std::env::current_exe().map_err(RuntimeError::FailedCurrentExe)?,
            });
        }
        serde_json::from_str(cstr).map_err(RuntimeError::JsonDeserialize)
    }

    fn apply_with_lock<F>(&mut self, operation: F) -> Result<(), RuntimeError>
    where
        F: FnOnce(&mut RuntimeInfo) -> Result<(), RuntimeError>,
    {
        self.lock()?;
        let mut runtime_info = self.read().map_err(self.handle_err_id())?;

        operation(&mut runtime_info).map_err(self.handle_err_id())?;

        self.write_locked(&runtime_info)
            .map_err(self.handle_err_id())?;
        self.unlock()?;
        Ok(())
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::managers::runtime::{
        FeatType, ProcessInfo, RuntimeInfo, RuntimeManagerImpl, RuntimeManagerWithoutAsync,
    };
    use crate::managers::unix_process_manager::UnixProcessManager;

    #[test]
    fn test_read_write_runtime_info() {
        let initial_runtime_info = RuntimeInfo {
            state: State::Init,
            process_infos: [None, None, None, None],
            exec_path: std::env::current_exe().unwrap(),
        };

        let mut mmap_handler = MmapHandler::new("test_shm").unwrap();

        mmap_handler
            .apply_with_lock(|runtime_info| {
                *runtime_info = initial_runtime_info.clone();
                Ok(())
            })
            .unwrap();

        let read_runtime_info = mmap_handler.read().unwrap();
        assert_eq!(read_runtime_info, initial_runtime_info);
        mmap_handler.close().unwrap();
    }

    #[test]
    fn test_update_state() {
        let mmap_handler = MmapHandler::new("test_shm_state").unwrap();
        let mut runtime_manager =
            RuntimeManagerImpl::new_by_agent(mmap_handler, UnixProcessManager);

        runtime_manager
            .update_state_without_send(State::Update)
            .unwrap();

        let state = runtime_manager.get_runtime_info().unwrap().state;

        assert_eq!(state, State::Update);
        MmapHandler::new("test_shm_state").unwrap().close().unwrap();
    }

    #[test]
    fn test_cleanup_process_info() {
        let process_info = ProcessInfo::new((1 << 22) + 1, FeatType::Agent);
        let runtime_info = RuntimeInfo {
            state: State::Init,
            process_infos: [Some(process_info.clone()), None, None, None],
            exec_path: std::env::current_exe().unwrap(),
        };
        let mut mmap_handler = MmapHandler::new("test_cleanup_process_info_shm").unwrap();
        mmap_handler.write_locked(&runtime_info).unwrap();
        let mut runtime_manager = RuntimeManagerImpl::new_by_controller(
            mmap_handler,
            UnixProcessManager,
            "/tmp/nodex.sock",
        )
        .unwrap()
        .0;

        let process_infos: Vec<_> = runtime_manager
            .get_runtime_info()
            .unwrap()
            .process_infos
            .into_iter()
            .flatten()
            .collect();
        assert!(!process_infos.contains(&process_info));
        MmapHandler::new("test_cleanup_process_info_shm")
            .unwrap()
            .close()
            .unwrap();
    }
}
