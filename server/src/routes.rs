use axum::{routing::post, Json, Router};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use tx3_sdk::trp::TxEnvelope;

use protocol::{AddParams, Client, ClientOptions, CreateWithLovelaceParams, DeployParams};

use crate::evaluate_tx;

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
    Router::new()
        .route("/deploy-settings", post(deploy_settings))
        .route("/create-bounty", post(create_bounty))
        .route("/add-funds", post(add_funds))
}

async fn create_bounty(
    Json(req): Json<CreateWithLovelaceParams>,
) -> Json<Result<TxEnvelope, String>> {
    println!("Received create bounty request: {:?}", req);

    match PROTOCOL.create_with_lovelace_tx(req).await {
        Ok(tx) => {
            println!("Generated CBOR: {}", tx.tx);
            let evaluated_tx_or_err = evaluate_tx::evaluate_tx(tx).await;
            Json(evaluated_tx_or_err)
        }
        Err(e) => {
            println!("Error creating bounty: {:?}", e);
            Json(Err(format!("Error creating bounty: {:?}", e)))
        }
    }
}

async fn add_funds(Json(req): Json<AddParams>) -> Json<Result<TxEnvelope, String>> {
    println!("Received add funds request: {:?}", req);

    match PROTOCOL.add_tx(req).await {
        Ok(tx) => {
            println!("Generated CBOR: {}", tx.tx);
            let evaluated_tx_or_err = evaluate_tx::evaluate_tx(tx).await;
            Json(evaluated_tx_or_err)
        }
        Err(e) => {
            println!("Error adding funds: {:?}", e);
            Json(Err(format!("Error adding funds: {:?}", e)))
        }
    }
}

async fn deploy_settings(Json(req): Json<DeployParams>) -> Json<Result<TxEnvelope, String>> {
    println!("Received deploy settings request: {:?}", req);

    match PROTOCOL.deploy_tx(req).await {
        Ok(tx) => {
            println!("Generated CBOR: {}", tx.tx);
            Json(Ok(tx))
        }
        Err(e) => {
            println!("Error deploying settings: {:?}", e);
            Json(Err(format!("Error deploying settings: {:?}", e)))
        }
    }
}
