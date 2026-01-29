use axum::Json;

use crate::tx_builders::{
    protocol::{UpdateParams, PROTOCOL},
    tx_result::{handle_tx_result, TxHandlerResult},
};

pub async fn update_settings(Json(req): Json<UpdateParams>) -> TxHandlerResult {
    log::info!("Received update settings request: {:?}", req);

    handle_tx_result(PROTOCOL.update_tx(req).await, true).await
}
