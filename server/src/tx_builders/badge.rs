use axum::Json;
use tx3_sdk::trp::TxEnvelope;

use crate::tx_builders::{
    protocol::{MintBadgeParams, PROTOCOL},
    utils::handle_tx_result,
};

pub async fn mint_badge(Json(req): Json<MintBadgeParams>) -> Json<Result<TxEnvelope, String>> {
    log::info!("Received mint badge settings request: {:?}", req);

    handle_tx_result(PROTOCOL.mint_badge_tx(req).await).await
}