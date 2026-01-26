use axum::Json;
use tx3_sdk::trp::TxEnvelope;

use crate::tx_builders::{
    protocol::{UpdateBadgeParams, PROTOCOL},
    utils::handle_tx_result,
};

pub async fn update_badge(Json(req): Json<UpdateBadgeParams>) -> Json<Result<TxEnvelope, String>> {
    log::info!("Received mint badge settings request: {:?}", req);

    handle_tx_result(PROTOCOL.update_badge_tx(req).await).await
}