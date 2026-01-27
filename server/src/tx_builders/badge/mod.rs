use crate::tx_builders::{
<<<<<<< HEAD:server/src/tx_builders/badge.rs
    protocol::{MintBadgeParams, PROTOCOL, UpdateBadgeParams, PayBadgesToParams},
    utils::{handle_tx_result, TxHandlerResult},
=======
    protocol::{MintBadgeParams, UpdateBadgeParams, PROTOCOL},
    tx_result::{handle_tx_result, TxHandlerResult},
>>>>>>> a0eded2 (split tx builders and utils into modules):server/src/tx_builders/badge/mod.rs
};
use axum::Json;

pub async fn mint_badge(Json(req): Json<MintBadgeParams>) -> TxHandlerResult {
    log::info!("Received mint badge settings request: {:?}", req);

    handle_tx_result(PROTOCOL.mint_badge_tx(req).await, true).await
}

pub async fn update_badge(Json(req): Json<UpdateBadgeParams>) -> TxHandlerResult {
    log::info!("Received mint badge settings request: {:?}", req);

    handle_tx_result(PROTOCOL.update_badge_tx(req).await, true).await
}
<<<<<<< HEAD:server/src/tx_builders/badge.rs

pub async fn pay_badges_to(Json(req): Json<PayBadgesToParams>) -> TxHandlerResult {
    log::info!("Received mint badge settings request: {:?}", req);

    handle_tx_result(PROTOCOL.pay_badges_to_tx(req).await, false).await
}
=======
>>>>>>> a0eded2 (split tx builders and utils into modules):server/src/tx_builders/badge/mod.rs
