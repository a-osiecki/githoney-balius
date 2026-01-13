use std::collections::HashMap;

use axum::{
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;

use transfer::{Client, ClientOptions, TransferParams};

use once_cell::sync::Lazy;

#[derive(Serialize)]
struct TransferResponse {
    cbor_hex: String,
}

fn build_client() -> Client {
    const TRP_ENDPOINT: &str = "https://cardano-preprod.trp-m1.demeter.run";
    const HEADERS: &[(&str, &str)] = &[("dmtr-api-key", "trp1plzft88zdrf2gktm8ad")];
    // Build the TRP client with custom endpoint and headers
    let mut headers: HashMap<String, String> = HashMap::new();
    for (key, value) in HEADERS {
        headers.insert(key.to_string(), value.to_string());
    }
    let client_options = ClientOptions {
        endpoint: TRP_ENDPOINT.to_string(),
        headers: Some(headers),
    };
    Client::new(client_options)
}

static PROTOCOL: Lazy<Client> = Lazy::new(|| build_client());

pub fn router() -> Router {
    Router::new().route("/transfer", post(transfer))
}

async fn transfer(Json(req): Json<TransferParams>) -> Json<Result<TransferResponse, String>> {
    println!("Received transfer request: {:?}", req);

    let cbor = match PROTOCOL.transfer_tx(req).await {
        Ok(tx) => tx.tx,
        Err(e) => {
            panic!("Error creating transfer transaction: {:?}", e);
        }
    };
    println!("Generated CBOR: {}", cbor);
    Json(Ok(TransferResponse { cbor_hex: cbor }))
}
