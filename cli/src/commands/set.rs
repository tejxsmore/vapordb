use crate::utils::{send_request, ClientCommand};

pub fn handle_sadd(key: &str, value: &str) {
    send_request(ClientCommand::SAdd {
        key: key.to_string(),
        value: value.to_string(),
    });
}

pub fn handle_srem(key: &str, value: &str) {
    send_request(ClientCommand::SRem {
        key: key.to_string(),
        value: value.to_string(),
    });
}

pub fn handle_smembers(key: &str) {
    send_request(ClientCommand::SMembers {
        key: key.to_string(),
    });
}
