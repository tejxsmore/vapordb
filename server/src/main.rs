use warp::{Filter, Rejection, Reply, http::StatusCode};
use core::db::VaporDB;
use core::command::Command;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use warp::http::Method;
use core::ttl_daemon::start_ttl_daemon;
use std::time::Duration;

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

// Custom wrapper to use VaporDBError with warp rejections
#[derive(Debug)]
struct RejectionWrapper(pub core::error::VaporDBError);
impl warp::reject::Reject for RejectionWrapper {}

#[tokio::main]
async fn main() {
    let db = Arc::new(Mutex::new(
        VaporDB::new_with_persistence("vapordb.wal").expect("Failed to init DB"),
    ));

    let db_locked = db.lock().unwrap();
    let memtable = db_locked.memtable();             
    let expirations = db_locked.expiration_table();
    let sstable = db_locked.sstable();

    drop(db_locked);

    // Spawn the TTL background task
    start_ttl_daemon(expirations, memtable, sstable, Duration::from_millis(100), false);

    let db_filter = warp::any().map(move || db.clone());

    let cmd_route = warp::post()
        .and(warp::path("cmd"))
        .and(warp::body::json())
        .and(db_filter)
        .and_then(handle_command)
        .recover(handle_rejection);

    // ✅ Add CORS support
    let cors = warp::cors()
        .allow_origin("http://localhost:5173")
        .allow_methods(&[Method::POST])
        .allow_headers(vec!["Content-Type"]);

    println!("🚀 VaporDB server running on http://127.0.0.1:3030");
    warp::serve(cmd_route.with(cors)).run(([127, 0, 0, 1], 3030)).await;
}

async fn handle_command(
    cmd: ClientCommand,
    db: Arc<Mutex<VaporDB>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut db = db.lock().unwrap();

    let result = match cmd {
        ClientCommand::Get { key } => {
            db.execute(Command::Get(key))
                .map_err(|e| warp::reject::custom(RejectionWrapper(e)))?
        }
        ClientCommand::Set { key, value } => {
            db.execute(Command::Set(key, value))
                .map_err(|e| warp::reject::custom(RejectionWrapper(e)))?;
            None
        }
        ClientCommand::Del { key } => {
            db.execute(Command::Del(key))
                .map_err(|e| warp::reject::custom(RejectionWrapper(e)))?;
            None
        }
        ClientCommand::SetWithExpiration { key, value, ttl_secs } => {
            db.set_with_expiration(key, value, ttl_secs)
                .map_err(|e| warp::reject::custom(RejectionWrapper(e)))?;
            None
        }
    };

    Ok(warp::reply::json(&Response {
        result,
        error: None,
    }))
}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, std::convert::Infallible> {
    if let Some(rejection) = err.find::<RejectionWrapper>() {
        let json = warp::reply::json(&Response {
            result: None,
            error: Some(format!("VaporDB error: {:?}", rejection.0)),
        });
        Ok(warp::reply::with_status(json, StatusCode::INTERNAL_SERVER_ERROR))
    } else {
        let json = warp::reply::json(&Response {
            result: None,
            error: Some("Unknown error".to_string()),
        });
        Ok(warp::reply::with_status(json, StatusCode::INTERNAL_SERVER_ERROR))
    }
}