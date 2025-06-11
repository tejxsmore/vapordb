use crate::utils::{send_request, ClientCommand};

pub fn handle_get(key: &str) {
    let cmd = ClientCommand::Get { key };
    send_request(cmd);
}
