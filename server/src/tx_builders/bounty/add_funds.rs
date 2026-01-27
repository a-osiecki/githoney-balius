use axum::Json;

use crate::tx_builders::{
    protocol::{AddParams, PROTOCOL},
    tx_result::{handle_tx_result, TxHandlerResult},
};

pub async fn add_funds(Json(req): Json<AddParams>) -> TxHandlerResult {
    log::info!("Received add funds request: {:?}", req);

    handle_tx_result(PROTOCOL.add_tx(req).await, true).await
}
