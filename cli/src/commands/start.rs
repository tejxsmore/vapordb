use std::sync::{Arc, Mutex};
use warp::Filter;
use core::db::VaporDB;
use core::command::Command;
use cli::utils::{ClientCommand, Response};

pub fn start_server() {
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
        // === String ===
        ClientCommand::Get { key } => db.execute(Command::Get(key.to_string())).unwrap_or(None),
        ClientCommand::Set { key, value } => {
            db.execute(Command::Set(key.to_string(), value.to_string())).ok();
            None
        }
        ClientCommand::Del { key } => {
            db.execute(Command::Del(key.to_string())).ok();
            None
        }
        ClientCommand::SetWithExpiration { key, value, ttl_secs } => {
            db.set_with_expiration(key.to_string(), value.to_string(), ttl_secs).ok();
            None
        }

        // === Hash ===
        ClientCommand::HSet { key, field, value } => {
            db.execute(Command::HSet(key.to_string(), field.to_string(), value.to_string())).ok();
            None
        }
        ClientCommand::HGet { key, field } => {
            db.execute(Command::HGet(key.to_string(), field.to_string())).unwrap_or(None)
        }
        ClientCommand::HDel { key, field } => {
            db.execute(Command::HDel(key.to_string(), field.to_string())).ok();
            None
        }

        // === List ===
        ClientCommand::LPush { key, value } => {
            db.execute(Command::LPush(key.to_string(), value.to_string())).ok();
            None
        }
        ClientCommand::RPush { key, value } => {
            db.execute(Command::RPush(key.to_string(), value.to_string())).ok();
            None
        }
        ClientCommand::LPop { key } => db.execute(Command::LPop(key.to_string())).unwrap_or(None),
        ClientCommand::RPop { key } => db.execute(Command::RPop(key.to_string())).unwrap_or(None),
        ClientCommand::LRange { key, start, end } => {
            db.execute(Command::LRange(key.to_string(), start, end)).unwrap_or(None)
        }

        // === Set ===
        ClientCommand::SAdd { key, value } => {
            db.execute(Command::SAdd(key.to_string(), value.to_string())).ok();
            None
        }
        ClientCommand::SRem { key, value } => {
            db.execute(Command::SRem(key.to_string(), value.to_string())).ok();
            None
        }
        ClientCommand::SMembers { key } => db.execute(Command::SMembers(key.to_string())).unwrap_or(None),
    };

    let resp = Response {
        result,
        error: None,
    };

    Ok(warp::reply::json(&resp))
}
