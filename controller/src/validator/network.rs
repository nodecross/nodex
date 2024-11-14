use sysinfo::Networks;

pub fn is_online() -> bool {
    let networks = Networks::new();
    networks
        .iter()
        .any(|(_, network)| network.received() > 0 || network.transmitted() > 0)
}
