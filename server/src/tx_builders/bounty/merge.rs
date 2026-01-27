use axum::Json;
use crate::tx_builders::{
    protocol::{MergeParams, PROTOCOL},
    tx_result::{handle_tx_result, TxHandlerResult},
};

pub async fn merge(Json(req): Json<MergeParams>) -> TxHandlerResult {
    log::info!("Received merge request: {:?}", req);

    handle_tx_result(PROTOCOL.merge_tx(req).await, true).await
}
