use axum::Json;
use crate::tx_builders::{
    protocol::{MintBadgeParams, PROTOCOL, UpdateBadgeParams, PayBadgesToParams, CollectUtxosParams},
    utils::{handle_tx_result, TxHandlerResult},
};

pub async fn mint_badge(Json(req): Json<MintBadgeParams>) -> TxHandlerResult {
    log::info!("Received mint badge request: {:?}", req);

    handle_tx_result(PROTOCOL.mint_badge_tx(req).await, true).await
}

pub async fn update_badge(Json(req): Json<UpdateBadgeParams>) -> TxHandlerResult {
    log::info!("Received update badge request: {:?}", req);

    handle_tx_result(PROTOCOL.update_badge_tx(req).await, true).await
}

pub async fn pay_badges_to(Json(req): Json<PayBadgesToParams>) -> TxHandlerResult {
    log::info!("Received pay badges to request: {:?}", req);

    handle_tx_result(PROTOCOL.pay_badges_to_tx(req).await, false).await
}

pub async fn collect_utxos(Json(req): Json<CollectUtxosParams>) -> TxHandlerResult {
    log::info!("Received collect utxos request: {:?}", req);

    handle_tx_result(PROTOCOL.collect_utxos_tx(req).await, true).await
}