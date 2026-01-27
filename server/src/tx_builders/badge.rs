use crate::tx_builders::{
    protocol::{MintBadgeParams, PROTOCOL, UpdateBadgeParams, PayBadgesToParams},
    utils::{handle_tx_result, TxHandlerResult},
};
use axum::Json;
use githoney::DebugParams;

pub async fn mint_badge(Json(req): Json<MintBadgeParams>) -> TxHandlerResult {
    log::info!("Received mint badge settings request: {:?}", req);

    handle_tx_result(PROTOCOL.mint_badge_tx(req).await, true).await
}

pub async fn update_badge(Json(req): Json<UpdateBadgeParams>) -> TxHandlerResult {
    log::info!("Received mint badge settings request: {:?}", req);

    handle_tx_result(PROTOCOL.update_badge_tx(req).await, true).await
}

pub async fn pay_badges_to(Json(req): Json<PayBadgesToParams>) -> TxHandlerResult {
    log::info!("Received mint badge settings request: {:?}", req);

    handle_tx_result(PROTOCOL.pay_badges_to_tx(req).await, false).await
}
