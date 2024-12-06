#[cfg(unix)]
use nix::sys::signal;
#[cfg(unix)]
use nix::unistd::Pid;
use std::env;

pub fn is_manage_by_systemd() -> bool {
    env::var("INVOCATION_ID").is_ok()
}

pub fn is_manage_socket_activation() -> bool {
    env::var("LISTEN_PID").is_ok() && env::var("LISTEN_FDS").is_ok()
}

#[cfg(unix)]
pub fn is_running(process_id: u32) -> bool {
    let pid = Pid::from_raw(process_id as i32);
    match signal::kill(pid, None) {
        Ok(_) => true,
        Err(nix::errno::Errno::ESRCH) => false,
        Err(_) => false,
    }
}

#[cfg(windows)]
pub fn is_running(process_id: u32) -> bool {
    unimplemented!("implemented for Windows.");
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;

    #[test]
    #[serial]
    fn test_is_manage_by_systemd() {
        env::set_var("INVOCATION_ID", "dummy_id");
        assert!(is_manage_by_systemd(), "Expected to be managed by systemd");

        env::remove_var("INVOCATION_ID");
        assert!(
            !is_manage_by_systemd(),
            "Expected not to be managed by systemd"
        );
    }

    #[test]
    #[serial]
    fn test_is_manage_socket_activation() {
        env::set_var("LISTEN_PID", "12345");
        env::set_var("LISTEN_FDS", "2");
        assert!(
            is_manage_socket_activation(),
            "Expected to be managed by socket activation"
        );

        env::remove_var("LISTEN_PID");
        assert!(
            !is_manage_socket_activation(),
            "Expected not to be managed by socket activation"
        );

        env::set_var("LISTEN_PID", "12345");
        env::remove_var("LISTEN_FDS");
        assert!(
            !is_manage_socket_activation(),
            "Expected not to be managed by socket activation"
        );
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_is_running() {
        use nix::unistd::{fork, ForkResult};

        match unsafe { fork() } {
            Ok(ForkResult::Child) => {
                std::process::exit(0);
            }
            Ok(ForkResult::Parent { child }) => {
                assert!(
                    is_running(child.as_raw() as u32),
                    "Expected child process to be running"
                );

                let status =
                    nix::sys::wait::waitpid(child, None).expect("Failed to wait for child process");
                assert!(
                    !is_running(child.as_raw() as u32),
                    "Expected child process to have exited, but it is still running"
                );

                if let nix::sys::wait::WaitStatus::Exited(_, exit_code) = status {
                    assert_eq!(exit_code, 0, "Child process did not exit cleanly");
                }
            }
            Err(_) => panic!("Fork failed"),
        }
    }
}
