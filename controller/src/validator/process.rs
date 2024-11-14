use nix::sys::signal;
use nix::unistd::Pid;
use std::env;

pub fn is_manage_by_systemd() -> bool {
    env::var("INVOCATION_ID").is_ok()
}

pub fn is_manage_socket_activation() -> bool {
    env::var("LISTEN_PID").is_ok() && env::var("LISTEN_FDS").is_ok()
}

pub fn is_running(process_id: u32) -> bool {
    let pid = Pid::from_raw(process_id as i32);
    match signal::kill(pid, None) {
        Ok(_) => true,
        Err(nix::errno::Errno::ESRCH) => false,
        Err(_) => false,
    }
}
