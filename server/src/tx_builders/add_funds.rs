use axum::Json;
use tx3_sdk::trp::TxEnvelope;

use crate::tx_builders::{protocol::{PROTOCOL, AddParams}, utils::handle_tx_result};

pub async fn add_funds(Json(req): Json<AddParams>) -> Json<Result<TxEnvelope, String>> {
    log::info!("Received add funds request: {:?}", req);

    handle_tx_result(PROTOCOL.add_tx(req).await).await
}
