use axum::Json;
use crate::tx_builders::{
    protocol::{MintBadgeParams, PROTOCOL},
    utils::{handle_tx_result, TxHandlerResult},
};

pub async fn mint_badge(Json(req): Json<MintBadgeParams>) -> TxHandlerResult {
    log::info!("Received mint badge settings request: {:?}", req);

    handle_tx_result(PROTOCOL.mint_badge_tx(req).await).await
}
