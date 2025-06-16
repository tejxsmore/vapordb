use crate::utils::{send_request, ClientCommand};

pub fn handle_hset(key: &str, field: &str, value: &str) {
    send_request(ClientCommand::HSet {
        key: key.to_string(),
        field: field.to_string(),
        value: value.to_string(),
    });
}

pub fn handle_hget(key: &str, field: &str) {
    send_request(ClientCommand::HGet {
        key: key.to_string(),
        field: field.to_string(),
    });
}

pub fn handle_hdel(key: &str, field: &str) {
    send_request(ClientCommand::HDel {
        key: key.to_string(),
        field: field.to_string(),
    });
}
