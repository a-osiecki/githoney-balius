use axum::Json;
use tx3_sdk::trp::TxEnvelope;

use crate::tx_builders::{protocol::{PROTOCOL, MergeParams}, utils::handle_tx_result};

pub async fn merge(Json(req): Json<MergeParams>) -> Json<Result<TxEnvelope, String>> {
    log::info!("Received merge request: {:?}", req);

    handle_tx_result(PROTOCOL.merge_tx(req).await).await
}