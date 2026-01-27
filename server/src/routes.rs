use axum::{routing::post, Router};

use crate::tx_builders::{
    add_funds, assign_contributor, claim, close_assigned, close_assigned_sponsored, close_unassigned, close_unassigned_sponsored, create_bounty, deploy_settings, merge, mint_badge, update_badge, pay_badges_to
};

pub fn router() -> Router {
    Router::new()
        .route("/deploy-settings", post(deploy_settings))
        .route("/create-bounty", post(create_bounty))
        .route("/add-funds", post(add_funds))
        .route("/assign", post(assign_contributor))
        .route("/close-unassigned", post(close_unassigned))
        .route(
            "/close-unassigned-sponsored",
            post(close_unassigned_sponsored),
        )
        .route("/close-assigned", post(close_assigned))
        .route("/close-assigned-sponsored", post(close_assigned_sponsored))
        .route("/merge", post(merge))
        .route("/claim", post(claim))
        .route("/mint-badges", post(mint_badge))
        .route("/update-badge", post(update_badge))
        .route("/pay-badges-to", post(pay_badges_to))
}

