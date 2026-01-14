use std::collections::HashMap;

use axum::{
    routing::{post},
    Json, Router,
};
use serde::Serialize;

use protocol::{Client, ClientOptions, CreateWithLovelaceParams};

use once_cell::sync::Lazy;

#[derive(Serialize)]
struct CreateBountyResponse {
    cbor_hex: String,
}

fn build_client() -> Client {
    let trp_endpoint = std::env::var("TRP_ENDPOINT").unwrap();
    let dmtr_api_key: String = std::env::var("DMTR_API_KEY").unwrap();
    let headers: &[(&str, &str)] = &[("dmtr-api-key", dmtr_api_key.as_str())];
    // Build the TRP client with custom endpoint and headers
    let mut headers_hm: HashMap<String, String> = HashMap::new();
    for (key, value) in headers {
        headers_hm.insert(key.to_string(), value.to_string());
    }
    let client_options = ClientOptions {
        endpoint: trp_endpoint,
        headers: Some(headers_hm),
    };
    Client::new(client_options)
}

static PROTOCOL: Lazy<Client> = Lazy::new(|| build_client());

pub fn router() -> Router {
    Router::new().route("/protocol", post(create_bounty))
}

async fn create_bounty(Json(req): Json<CreateWithLovelaceParams>) -> Json<Result<CreateBountyResponse, String>> {
    println!("Received create bounty request: {:?}", req);

    let cbor = match PROTOCOL.create_with_lovelace_tx(req).await {
        Ok(tx) => tx.tx,
        Err(e) => {
            panic!("Error creating bounty: {:?}", e);
        }
    };
    println!("Generated CBOR: {}", cbor);
    Json(Ok(CreateBountyResponse { cbor_hex: cbor }))
}
