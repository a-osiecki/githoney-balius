use axum::Json;

use crate::tx_builders::{
    protocol::{DeployParams, PROTOCOL},
    utils::{handle_tx_result, TxHandlerResult},
};

pub async fn deploy_settings(Json(req): Json<DeployParams>) -> TxHandlerResult {
    log::info!("Received deploy settings request: {:?}", req);

    handle_tx_result(PROTOCOL.deploy_tx(req).await, true).await
}
