use axum::{routing::post, Router};

use crate::tx_builders::{
    add_funds, close_assigned, close_assigned_sponsored, close_unassigned,
    close_unassigned_sponsored, create_bounty, deploy_settings,
};

pub fn router() -> Router {
    Router::new()
        .route("/deploy-settings", post(deploy_settings))
        .route("/create-bounty", post(create_bounty))
        .route("/add-funds", post(add_funds))
        .route("/close-unassigned", post(close_unassigned))
        .route(
            "/close-unassigned-sponsored",
            post(close_unassigned_sponsored),
        )
        .route("/close-assigned", post(close_assigned))
        .route("/close-assigned-sponsored", post(close_assigned_sponsored))
}
