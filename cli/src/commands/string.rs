use crate::utils::{send_request, ClientCommand};

pub fn handle_get(key: &str) {
    send_request(ClientCommand::Get { key: key.to_string() });
}

pub fn handle_set(key: &str, value: &str) {
    send_request(ClientCommand::Set { key: key.to_string(), value: value.to_string() });
}

pub fn handle_del(key: &str) {
    send_request(ClientCommand::Del { key: key.to_string() });
}

pub fn handle_set_expiring(key: &str, value: &str, ttl_secs: u64) {
    send_request(ClientCommand::SetWithExpiration {
        key: key.to_string(),
        value: value.to_string(),
        ttl_secs,
    });
}
