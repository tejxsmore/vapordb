use std::sync::{Arc, Mutex};
use core::db::VaporDB;
use crate::api::routes;

pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    let db = Arc::new(Mutex::new(
        VaporDB::new_with_persistence("vapordb.wal")?,
    ));

    let api = routes(db);

    println!("VaporDB server running on http://127.0.0.1:3030");
    warp::serve(api).run(([127, 0, 0, 1], 3030)).await;

    Ok(())
}
