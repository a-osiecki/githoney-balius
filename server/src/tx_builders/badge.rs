use axum::Json;
use tx3_sdk::trp::TxEnvelope;

use crate::tx_builders::{
    protocol::{DeployBadgeParams, PROTOCOL},
    utils::handle_tx_result,
};

pub async fn deploy_badge(Json(req): Json<DeployBadgeParams>) -> Json<Result<TxEnvelope, String>> {
    log::info!("Received deploy badge settings request: {:?}", req);

    handle_tx_result(PROTOCOL.deploy_badge_tx(req).await).await
}