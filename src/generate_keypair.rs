use axum::{debug_handler, http::StatusCode, Json};
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GeneratedKeypairApiResponse {
    pub pubkey: Pubkey,
    pub secret_key: String,
}

#[debug_handler]
pub async fn generate_keypair() -> (StatusCode, Json<GeneratedKeypairApiResponse>) {
    let keypair = Keypair::new();
    let pubkey = keypair.pubkey();
    let secret_key = bs58::encode(keypair.to_bytes()).into_string();

    let response = GeneratedKeypairApiResponse { pubkey, secret_key };

    (StatusCode::CREATED, Json(response))
}
