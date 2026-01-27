use axum::Json;
use log::info;

use crate::tx_builders::{
    protocol::{AssignParams, PROTOCOL},
    tx_result::{handle_tx_result, TxHandlerResult},
};

pub async fn assign_contributor(Json(req): Json<AssignParams>) -> TxHandlerResult {
    info!("Received assign contributor request: {:?}", req);

    handle_tx_result(PROTOCOL.assign_tx(req).await, true).await
}
