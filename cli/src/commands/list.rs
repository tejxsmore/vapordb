use crate::utils::{send_request, ClientCommand};

pub fn handle_lpush(key: &str, value: &str) {
    send_request(ClientCommand::LPush {
        key: key.to_string(),
        value: value.to_string(),
    });
}

pub fn handle_rpush(key: &str, value: &str) {
    send_request(ClientCommand::RPush {
        key: key.to_string(),
        value: value.to_string(),
    });
}

pub fn handle_lpop(key: &str) {
    send_request(ClientCommand::LPop {
        key: key.to_string(),
    });
}

pub fn handle_rpop(key: &str) {
    send_request(ClientCommand::RPop {
        key: key.to_string(),
    });
}

pub fn handle_lrange(key: &str, start: usize, end: usize) {
    send_request(ClientCommand::LRange {
        key: key.to_string(),
        start,
        end,
    });
}
