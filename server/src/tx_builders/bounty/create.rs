use axum::{http::StatusCode, Json};
use serde::Deserialize;

use crate::tx_builders::{
    protocol::{CreateWithLovelaceParams, CreateWithTokenParams, PROTOCOL},
    tx_result::{handle_tx_result, ErrorResponse, TxHandlerResult},
};

#[derive(Debug, Clone, Deserialize)]
pub struct CreateParams {
    pub admin_payment_key: String,
    pub bounty_creation_fee: String,
    pub bounty_id: String,
    pub bounty_rewards_fee: String,
    pub githoneyaddr: String,
    pub maintainer: String,
    pub maintainer_payment_key: String,
    pub maintainer_stake_key: String,
    pub min_ada: String,
    pub minting_policy_id: String,
    pub reward_amount: String,
    pub reward_asset_name: Option<String>,
    pub reward_policy_id: Option<String>,
    pub script: String,
    pub settings_ref: String,
    pub since: String,
    pub time_limit: String,
    pub until: String,
}
pub async fn create_bounty(Json(req): Json<CreateParams>) -> TxHandlerResult {
    log::info!("Received create bounty request: {:?}", req);

    let CreateParams {
        admin_payment_key,
        bounty_creation_fee,
        bounty_id,
        bounty_rewards_fee,
        githoneyaddr,
        maintainer,
        maintainer_payment_key,
        maintainer_stake_key,
        min_ada,
        minting_policy_id,
        reward_amount,
        reward_asset_name,
        reward_policy_id,
        script,
        settings_ref,
        since,
        time_limit,
        until
    } = req;

    let is_created_with_tokens = reward_policy_id.is_some()
        || reward_asset_name.is_some();

    if is_created_with_tokens {
        let reward_policy_id = match reward_policy_id {
            Some(reward_policy_id) => reward_policy_id,
            None => return bad_request("Create with tokens requires reward_policy_id"),
        };

        let reward_asset_name = match reward_asset_name {
            Some(reward_asset_name) => reward_asset_name,
            None => return bad_request("Create with tokens requires reward_asset_name"),
        };

        let params = CreateWithTokenParams{
            admin_payment_key,
            bounty_creation_fee,
            bounty_id,
            bounty_rewards_fee,
            githoneyaddr,
            maintainer,
            maintainer_payment_key,
            maintainer_stake_key,
            min_ada,
            minting_policy_id,
            reward_amount,
            reward_asset_name,
            reward_policy_id,
            script,
            settings_ref,
            since,
            time_limit,
            until,
        };

        return handle_tx_result(PROTOCOL.create_with_token_tx(params).await, true)
            .await;
    }
    else {
        let params = CreateWithLovelaceParams{
            admin_payment_key,
            bounty_creation_fee,
            bounty_id,
            bounty_rewards_fee,
            githoneyaddr,
            maintainer,
            maintainer_payment_key,
            maintainer_stake_key,
            min_ada,
            minting_policy_id,
            reward_amount,
            script,
            settings_ref,
            since,
            time_limit,
            until,
        };
        return handle_tx_result(PROTOCOL.create_with_lovelace_tx(params).await, true).await;
    }
//    handle_tx_result(PROTOCOL.create_with_lovelace_tx(req).await, true).await
}

fn bad_request(message: impl Into<String>) -> TxHandlerResult {
    Err((
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            error: message.into(),
        }),
    ))
}