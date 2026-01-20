use axum::Json;
use tx3_sdk::trp::TxEnvelope;

use crate::tx_builders::{
    protocol::{DeployParams, PROTOCOL},
    utils::handle_tx_result,
};

pub async fn deploy_settings(Json(req): Json<DeployParams>) -> Json<Result<TxEnvelope, String>> {
    log::info!("Received deploy settings request: {:?}", req);

    handle_tx_result(PROTOCOL.deploy_tx(req).await).await
}
