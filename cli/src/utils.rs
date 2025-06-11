use serde::{Deserialize, Serialize};
use reqwest::blocking::Client;

#[derive(Serialize)]
#[serde(tag = "cmd", rename_all = "lowercase")]
pub enum ClientCommand<'a> {
    Get { key: &'a str },
    Set { key: &'a str, value: &'a str },
    Del { key: &'a str },
    SetWithExpiration { key: &'a str, value: &'a str, ttl_secs: u64 },
}

#[derive(Deserialize)]
pub struct Response {
    pub result: Option<String>,
    pub error: Option<String>,
}

pub fn send_request(cmd: ClientCommand) {
    let client = Client::new();
    let res = client
        .post("http://127.0.0.1:3030/cmd")
        .json(&cmd)
        .send();

    match res {
        Ok(r) => {
            match r.json::<Response>() {
                Ok(resp) => {
                    if let Some(result) = resp.result {
                        println!("Result: {}", result);
                    }
                    if let Some(err) = resp.error {
                        eprintln!("Error: {}", err);
                    }
                }
                Err(e) => eprintln!("Failed to parse response: {}", e),
            }
        }
        Err(e) => eprintln!("Request failed: {}", e),
    }
}
