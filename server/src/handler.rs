use warp::{Rejection, Reply, http::StatusCode};
use core::db::VaporDB;
use core::command::Command;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "lowercase")]
pub enum ClientCommand {
    Get { key: String },
    Set { key: String, value: String },
    Del { key: String },
    SetWithExpiration { key: String, value: String, ttl_secs: u64 },
}

#[derive(Serialize)]
pub struct Response {
    pub result: Option<String>,
    pub error: Option<String>,
}

// Wrap VaporDBError so it can be used with warp rejections
#[derive(Debug)]
pub struct RejectionWrapper(pub core::error::VaporDBError);

impl warp::reject::Reject for RejectionWrapper {}

pub async fn handle_command(
    cmd: ClientCommand,
    db: Arc<Mutex<VaporDB>>,
) -> Result<impl Reply, Rejection> {
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

    let resp = Response {
        result,
        error: None,
    };

    Ok(warp::reply::json(&resp))
}

pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, std::convert::Infallible> {
    if let Some(rejection) = err.find::<RejectionWrapper>() {
        eprintln!("VaporDB error: {:?}", rejection.0);

        let json = warp::reply::json(&Response {
            result: None,
            error: Some(format!("VaporDB error: {:?}", rejection.0)),
        });
        Ok(warp::reply::with_status(json, StatusCode::INTERNAL_SERVER_ERROR))
    } else {
        let json = warp::reply::json(&Response {
            result: None,
            error: Some("Unknown error".into()),
        });
        Ok(warp::reply::with_status(json, StatusCode::INTERNAL_SERVER_ERROR))
    }
}
