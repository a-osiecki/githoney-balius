use axum::Json;
use tx3_sdk::trp::TxEnvelope;

use crate::tx_builders::{
    protocol::{AssignParams, PROTOCOL},
    utils::handle_tx_result,
};

pub async fn assign_contributor(Json(req): Json<AssignParams>) -> Json<Result<TxEnvelope, String>> {
    println!("Received assign contributor request: {:?}", req);

    handle_tx_result(PROTOCOL.assign_tx(req).await).await
}
