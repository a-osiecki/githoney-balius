use axum::Json;

use crate::tx_builders::{
    protocol::{CreateWithLovelaceParams, PROTOCOL},
    utils::{handle_tx_result, TxHandlerResult},
};

pub async fn create_bounty(
    Json(req): Json<CreateWithLovelaceParams>,
) -> TxHandlerResult {
    log::info!("Received create bounty request: {:?}", req);

    handle_tx_result(PROTOCOL.create_with_lovelace_tx(req).await).await
}
