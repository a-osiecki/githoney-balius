use crate::tx_builders::{
    protocol::{ClaimParams, PROTOCOL},
    tx_result::{handle_tx_result, TxHandlerResult},
};
use axum::Json;
use log::info;

pub async fn claim(Json(req): Json<ClaimParams>) -> TxHandlerResult {
    info!("Received claim bounty request: {:?}", req);

    handle_tx_result(PROTOCOL.claim_tx(req).await, true).await
}
