use std::sync::{Arc, Mutex};
use warp::Filter;
use core::db::VaporDB;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "lowercase")]
enum ClientCommand {
    Get { key: String },
    Set { key: String, value: String },
    Del { key: String },
    SetWithExpiration { key: String, value: String, ttl_secs: u64 },
}

#[derive(Serialize)]
struct Response {
    result: Option<String>,
    error: Option<String>,
}

pub fn start_server() {
    // Start the server using tokio runtime
    tokio_main();
}

#[tokio::main]
async fn tokio_main() {
    let db = Arc::new(Mutex::new(
        VaporDB::new_with_persistence("vapordb.wal").expect("Failed to start DB"),
    ));

    let db_filter = warp::any().map(move || db.clone());

    let cmd_route = warp::post()
        .and(warp::path("cmd"))
        .and(warp::body::json())
        .and(db_filter)
        .and_then(handle_command);

    println!("VaporDB server running at http://127.0.0.1:3030");
    warp::serve(cmd_route).run(([127, 0, 0, 1], 3030)).await;
}

async fn handle_command(
    cmd: ClientCommand,
    db: Arc<Mutex<VaporDB>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut db = db.lock().unwrap();

    let result = match cmd {
        ClientCommand::Get { key } => db.execute(core::command::Command::Get(key)).unwrap_or(None),
        ClientCommand::Set { key, value } => {
            db.execute(core::command::Command::Set(key, value)).ok();
            None
        }
        ClientCommand::Del { key } => {
            db.execute(core::command::Command::Del(key)).ok();
            None
        }
        ClientCommand::SetWithExpiration { key, value, ttl_secs } => {
            db.set_with_expiration(key, value, ttl_secs).ok();
            None
        }
    };

    let resp = Response {
        result,
        error: None,
    };

    Ok(warp::reply::json(&resp))
}
