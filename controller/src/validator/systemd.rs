use std::env;

pub fn is_manage_by_systemd() -> bool {
    env::var("INVOCATION_ID").is_ok()
}

pub fn is_manage_socket_activation() -> bool {
    env::var("LISTEN_PID").is_ok() && env::var("LISTEN_FDS").is_ok()
}
