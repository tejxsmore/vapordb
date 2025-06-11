use crate::utils::{send_request, ClientCommand};

pub fn handle_set(key: &str, value: &str) {
    let cmd = ClientCommand::Set { key, value };
    send_request(cmd);
}

pub fn handle_del(key: &str) {
    let cmd = ClientCommand::Del { key };
    send_request(cmd);
}

pub fn handle_set_expiring(key: &str, value: &str, ttl_secs: u64) {
    let cmd = ClientCommand::SetWithExpiration {
        key,
        value,
        ttl_secs,
    };
    send_request(cmd);
}
