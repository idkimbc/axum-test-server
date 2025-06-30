use axum::{routing::get, Router};

pub mod fruits;
pub use fruits::*;

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    // create the router
    let router = Router::new()
        .route("/fruits", get(get_all_fruits))
        .route("/fruit/{name}", get(get_single_fruit));

    Ok(router.into())
}
