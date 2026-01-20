use axum::Json;
use tx3_sdk::trp::TxEnvelope;

use crate::tx_builders::{
    protocol::{
        PROTOCOL,
        {
            CloseAssignedParams, CloseAssignedSponsoredParams, CloseUnassignedParams,
            CloseUnassignedSponsoredParams,
        },
    },
    utils::handle_tx_result,
};

pub async fn close_unassigned(
    Json(req): Json<CloseUnassignedParams>,
) -> Json<Result<TxEnvelope, String>> {
    log::info!("Received close before contributor request: {:?}", req);

    handle_tx_result(PROTOCOL.close_unassigned_tx(req).await).await
}

pub async fn close_unassigned_sponsored(
    Json(req): Json<CloseUnassignedSponsoredParams>,
) -> Json<Result<TxEnvelope, String>> {
    log::info!(
        "Received close before contributor with reward request: {:?}",
        req
    );

    handle_tx_result(PROTOCOL.close_unassigned_sponsored_tx(req).await).await
}

pub async fn close_assigned(
    Json(req): Json<CloseAssignedParams>,
) -> Json<Result<TxEnvelope, String>> {
    log::info!("Received close after contributor request: {:?}", req);

    handle_tx_result(PROTOCOL.close_assigned_tx(req).await).await
}

pub async fn close_assigned_sponsored(
    Json(req): Json<CloseAssignedSponsoredParams>,
) -> Json<Result<TxEnvelope, String>> {
    log::info!(
        "Received close after contributor with reward request: {:?}",
        req
    );

    handle_tx_result(PROTOCOL.close_assigned_sponsored_tx(req).await).await
}
