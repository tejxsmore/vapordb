use serde::{Deserialize, Serialize};
use reqwest::blocking::Client;

/// All client-side commands supported by the CLI and server.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "cmd", rename_all = "lowercase")]
pub enum ClientCommand {
    Get { key: String },
    Set { key: String, value: String },
    Del { key: String },
    SetWithExpiration { key: String, value: String, ttl_secs: u64 },

    // Hash commands
    HSet { key: String, field: String, value: String },
    HGet { key: String, field: String },
    HDel { key: String, field: String },

    // List commands
    LPush { key: String, value: String },
    RPush { key: String, value: String },
    LPop { key: String },
    RPop { key: String },
    LRange { key: String, start: usize, end: usize },

    // Set commands
    SAdd { key: String, value: String },
    SRem { key: String, value: String },
    SMembers { key: String },
}

/// Response from the server.
#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub result: Option<String>,
    pub error: Option<String>,
}

/// Sends a request to the VaporDB server and prints the response.
pub fn send_request(cmd: ClientCommand) -> Response {
    let client = Client::new();

    let res = client
        .post("http://127.0.0.1:3030/cmd")
        .json(&cmd)
        .send();

    match res {
        Ok(r) => match r.json::<Response>() {
            Ok(resp) => resp,
            Err(e) => Response {
                result: None,
                error: Some(format!("Failed to parse response: {}", e)),
            },
        },
        Err(e) => Response {
            result: None,
            error: Some(format!("Request failed: {}", e)),
        },
    }
}
