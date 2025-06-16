use warp::{http::StatusCode, Rejection, Reply};
use core::db::VaporDB;
use core::command::Command;
use cli::utils::{ClientCommand, Response};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct RejectionWrapper(pub core::error::VaporDBError);

impl warp::reject::Reject for RejectionWrapper {}

pub async fn handle_command(
    cmd: ClientCommand,
    db: Arc<Mutex<VaporDB>>,
) -> Result<impl Reply, Rejection> {
    let mut db = db.lock().unwrap();

    let result = match cmd {
        // === String ===
        ClientCommand::Get { key } => {
            db.execute(Command::Get(key.to_string()))
                .map_err(|e| warp::reject::custom(RejectionWrapper(e)))?
        }
        ClientCommand::Set { key, value } => {
            db.execute(Command::Set(key.to_string(), value.to_string()))
                .map_err(|e| warp::reject::custom(RejectionWrapper(e)))?;
            None
        }
        ClientCommand::Del { key } => {
            db.execute(Command::Del(key.to_string()))
                .map_err(|e| warp::reject::custom(RejectionWrapper(e)))?;
            None
        }
        ClientCommand::SetWithExpiration { key, value, ttl_secs } => {
            db.set_with_expiration(key.to_string(), value.to_string(), ttl_secs)
                .map_err(|e| warp::reject::custom(RejectionWrapper(e)))?;
            None
        }

        // === Hash ===
        ClientCommand::HSet { key, field, value } => {
            db.execute(Command::HSet(
                key.to_string(),
                field.to_string(),
                value.to_string(),
            ))
            .map_err(|e| warp::reject::custom(RejectionWrapper(e)))?;
            None
        }
        ClientCommand::HGet { key, field } => {
            db.execute(Command::HGet(
                key.to_string(),
                field.to_string(),
            ))
            .map_err(|e| warp::reject::custom(RejectionWrapper(e)))?
        }
        ClientCommand::HDel { key, field } => {
            db.execute(Command::HDel(
                key.to_string(),
                field.to_string(),
            ))
            .map_err(|e| warp::reject::custom(RejectionWrapper(e)))?;
            None
        }

        // === List ===
        ClientCommand::LPush { key, value } => {
            db.execute(Command::LPush(
                key.to_string(),
                value.to_string(),
            ))
            .map_err(|e| warp::reject::custom(RejectionWrapper(e)))?;
            None
        }
        ClientCommand::RPush { key, value } => {
            db.execute(Command::RPush(
                key.to_string(),
                value.to_string(),
            ))
            .map_err(|e| warp::reject::custom(RejectionWrapper(e)))?;
            None
        }
        ClientCommand::LPop { key } => {
            db.execute(Command::LPop(key.to_string()))
                .map_err(|e| warp::reject::custom(RejectionWrapper(e)))?
        }
        ClientCommand::RPop { key } => {
            db.execute(Command::RPop(key.to_string()))
                .map_err(|e| warp::reject::custom(RejectionWrapper(e)))?
        }
        ClientCommand::LRange { key, start, end } => {
            db.execute(Command::LRange(
                key.to_string(),
                start,
                end,
            ))
            .map_err(|e| warp::reject::custom(RejectionWrapper(e)))?
        }

        // === Set ===
        ClientCommand::SAdd { key, value } => {
            db.execute(Command::SAdd(
                key.to_string(),
                value.to_string(),
            ))
            .map_err(|e| warp::reject::custom(RejectionWrapper(e)))?;
            None
        }
        ClientCommand::SRem { key, value } => {
            db.execute(Command::SRem(
                key.to_string(),
                value.to_string(),
            ))
            .map_err(|e| warp::reject::custom(RejectionWrapper(e)))?;
            None
        }
        ClientCommand::SMembers { key } => {
            db.execute(Command::SMembers(key.to_string()))
                .map_err(|e| warp::reject::custom(RejectionWrapper(e)))?
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
