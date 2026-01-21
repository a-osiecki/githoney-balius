use axum::Json;
use log::info;
use tx3_sdk::trp::TxEnvelope;

use crate::tx_builders::{
    protocol::{ClaimParams, PROTOCOL},
    utils::handle_tx_result,
};

pub async fn claim(Json(req): Json<ClaimParams>) -> Json<Result<TxEnvelope, String>> {
    info!("Received claim bounty request: {:?}", req);

    handle_tx_result(PROTOCOL.claim_tx(req).await).await
}