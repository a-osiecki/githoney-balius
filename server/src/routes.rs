use axum::{routing::post, Router};

use crate::tx_builders::{
    add_funds, assign_contributor, claim, close, collect_utxos, create_bounty, deploy_settings, merge, mint_badge, pay_badges_to, update_badge, update_settings
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
        .route("/badge/collect", post(collect_utxos))
        .route("/settings/deploy", post(deploy_settings))
        .route("/settings/update", post(update_settings))
}
