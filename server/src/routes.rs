use axum::{routing::post, Router};

use crate::tx_builders::{
    add_funds, assign_contributor, claim, close_assigned, close_assigned_sponsored, close_unassigned, close_unassigned_sponsored, create_bounty, deploy_settings, merge, mint_badge, update_badge, pay_badges_to
};

pub fn router() -> Router {
    Router::new()
    .route("/bounty/create", post(create_bounty))
    .route("/bounty/add-funds", post(add_funds))
    .route("/bounty/assign", post(assign_contributor))
        .route("/bounty/close", post(close))
    .route("/bounty/merge", post(merge))
    .route("/bounty/claim", post(claim))
    .route("/badge/mint", post(mint_badge))
    .route("/badge/update", post(update_badge))
    .route("/badge/pay", post(pay_badges_to))
    .route("/settings/deploy", post(deploy_settings))
}
