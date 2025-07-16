use std::{str::FromStr, sync::Arc};

use axum::{
    routing::{get, post},
    Router,
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

pub mod fruits;
pub use fruits::*;
pub mod generate_keypair;
pub use generate_keypair::*;

pub struct AppState {
    pub program_id: Pubkey,
    pub rpc_client: Arc<RpcClient>,
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    // configure rpc client
    let rpc_url = "https://api.devnet.solana.com".to_string();
    let rpc_client = Arc::new(RpcClient::new(rpc_url));

    // configure program id
    let program_id_str = "FZQmSamSJdtB9JKxbUH82ZdRQ2UcqqBPGbyce2ZdfviN".to_string();
    let program_id = Pubkey::from_str(&program_id_str).unwrap();

    let app_state = Arc::new(AppState {
        program_id,
        rpc_client,
    });

    // create the router
    let router = Router::new()
        .route("/fruits", get(get_all_fruits))
        .route("/fruit/{name}", get(get_single_fruit))
        .route("/keypair", post(generate_keypair))
        .with_state(app_state);

    Ok(router.into())
}
