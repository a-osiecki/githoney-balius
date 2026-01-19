use axum::{routing::post, Json, Router};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use tx3_sdk::trp::{Error, TxEnvelope};

use protocol::{
    AddParams, Client, ClientOptions, CloseBeforeContributorParams, CreateWithLovelaceParams,
    DeployParams,
};

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
        .route("/close-before-contributor", post(close_before_contributor))
}

async fn handle_tx_result(
    tx_result: Result<TxEnvelope, Error>,
) -> Json<Result<TxEnvelope, String>> {
    match tx_result {
        Ok(tx) => {
            println!("Generated CBOR: {}", tx.tx);
            let evaluated_tx_or_err = evaluate_tx::evaluate_tx(tx).await;
            Json(evaluated_tx_or_err)
        }
        Err(e) => {
            println!("Error building transaction: {:?}", e);
            Json(Err(format!("Error building transaction: {:?}", e)))
        }
    }
}

async fn create_bounty(
    Json(req): Json<CreateWithLovelaceParams>,
) -> Json<Result<TxEnvelope, String>> {
    println!("Received create bounty request: {:?}", req);

    handle_tx_result(PROTOCOL.create_with_lovelace_tx(req).await).await
}

async fn add_funds(Json(req): Json<AddParams>) -> Json<Result<TxEnvelope, String>> {
    println!("Received add funds request: {:?}", req);

    handle_tx_result(PROTOCOL.add_tx(req).await).await
}

async fn deploy_settings(Json(req): Json<DeployParams>) -> Json<Result<TxEnvelope, String>> {
    println!("Received deploy settings request: {:?}", req);

    handle_tx_result(PROTOCOL.deploy_tx(req).await).await
}

async fn close_before_contributor(
    Json(req): Json<CloseBeforeContributorParams>,
) -> Json<Result<TxEnvelope, String>> {
    println!("Received close before contributor request: {:?}", req);

    handle_tx_result(PROTOCOL.close_before_contributor_tx(req).await).await
}
