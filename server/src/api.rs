use warp::Filter;
use std::sync::{Arc, Mutex};
use core::db::VaporDB;
use crate::{handler::{handle_command, handle_rejection}};

pub fn routes(
    db: Arc<Mutex<VaporDB>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let db_filter = warp::any().map(move || db.clone());

    warp::post()
        .and(warp::path("cmd"))
        .and(warp::body::json()) // this returns Result<T, warp::Rejection>
        .and(db_filter)
        .and_then(handle_command) // must return Result<impl Reply, warp::Rejection>
        .recover(handle_rejection)
        .boxed() // Box the filter to help type inference
}
