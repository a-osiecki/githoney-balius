use axum::{http::StatusCode, Json};
use serde::Deserialize;

use crate::tx_builders::{
    protocol::{
        PROTOCOL,
        {
            CloseAssignedParams, CloseAssignedSponsoredParams, CloseUnassignedParams,
            CloseUnassignedSponsoredParams,
        },
    },
    tx_result::{handle_tx_result, ErrorResponse, TxHandlerResult},
};

#[derive(Debug, Clone, Deserialize)]
pub struct CloseParams {
    pub admin: String,
    pub bounty_id: String,
    pub bounty_ref: String,
    pub maintainer: String,
    pub min_ada: String,
    pub minting_policy_id: String,
    pub reward_amount: String,
    pub reward_asset_name: String,
    pub reward_policy_id: String,
    pub settings_ref: String,
    pub since: String,
    pub until: String,
    pub contributor: Option<String>,
    pub sponsor: Option<String>,
    pub refundings_amount: Option<String>,
    pub refundings_asset_name: Option<String>,
    pub refundings_policy_id: Option<String>,
}

pub async fn close(Json(req): Json<CloseParams>) -> TxHandlerResult {
    log::info!("Received close request: {:?}", req);

    let CloseParams {
        admin,
        bounty_id,
        bounty_ref,
        maintainer,
        min_ada,
        minting_policy_id,
        reward_amount,
        reward_asset_name,
        reward_policy_id,
        settings_ref,
        since,
        until,
        contributor,
        sponsor,
        refundings_amount,
        refundings_asset_name,
        refundings_policy_id,
    } = req;

    let is_sponsored = sponsor.is_some()
        || refundings_amount.is_some()
        || refundings_asset_name.is_some()
        || refundings_policy_id.is_some();

    // Assigned close branch
    if let Some(contributor) = contributor {
        if is_sponsored {
            let sponsor = match sponsor {
                Some(sponsor) => sponsor,
                None => return bad_request("sponsored close requires sponsor"),
            };
            let refundings_amount = match refundings_amount {
                Some(refundings_amount) => refundings_amount,
                None => return bad_request("sponsored close requires refundings_amount"),
            };
            let refundings_asset_name = match refundings_asset_name {
                Some(refundings_asset_name) => refundings_asset_name,
                None => return bad_request("sponsored close requires refundings_asset_name"),
            };
            let refundings_policy_id = match refundings_policy_id {
                Some(refundings_policy_id) => refundings_policy_id,
                None => return bad_request("sponsored close requires refundings_policy_id"),
            };

            let params = CloseAssignedSponsoredParams {
                admin,
                bounty_id,
                bounty_ref,
                contributor,
                maintainer,
                min_ada,
                minting_policy_id,
                refundings_amount,
                refundings_asset_name,
                refundings_policy_id,
                reward_amount,
                reward_asset_name,
                reward_policy_id,
                settings_ref,
                since,
                sponsor,
                until,
            };

            return handle_tx_result(PROTOCOL.close_assigned_sponsored_tx(params).await).await;
        }

        let params = CloseAssignedParams {
            admin,
            bounty_id,
            bounty_ref,
            contributor,
            maintainer,
            min_ada,
            minting_policy_id,
            reward_amount,
            reward_asset_name,
            reward_policy_id,
            settings_ref,
            since,
            until,
        };

        return handle_tx_result(PROTOCOL.close_assigned_tx(params).await).await;
    }

    // Unassigned close branch
    if is_sponsored {
        let sponsor = match sponsor {
            Some(sponsor) => sponsor,
            None => return bad_request("sponsored close requires sponsor"),
        };
        let refundings_amount = match refundings_amount {
            Some(refundings_amount) => refundings_amount,
            None => return bad_request("sponsored close requires refundings_amount"),
        };
        let refundings_asset_name = match refundings_asset_name {
            Some(refundings_asset_name) => refundings_asset_name,
            None => return bad_request("sponsored close requires refundings_asset_name"),
        };
        let refundings_policy_id = match refundings_policy_id {
            Some(refundings_policy_id) => refundings_policy_id,
            None => return bad_request("sponsored close requires refundings_policy_id"),
        };

        let params = CloseUnassignedSponsoredParams {
            admin,
            bounty_id,
            bounty_ref,
            maintainer,
            min_ada,
            minting_policy_id,
            refundings_amount,
            refundings_asset_name,
            refundings_policy_id,
            reward_amount,
            reward_asset_name,
            reward_policy_id,
            settings_ref,
            since,
            sponsor,
            until,
        };

        return handle_tx_result(PROTOCOL.close_unassigned_sponsored_tx(params).await).await;
    }

    let params = CloseUnassignedParams {
        admin,
        bounty_id,
        bounty_ref,
        maintainer,
        min_ada,
        minting_policy_id,
        reward_amount,
        reward_asset_name,
        reward_policy_id,
        settings_ref,
        since,
        until,
    };

    handle_tx_result(PROTOCOL.close_unassigned_tx(params).await).await
}

fn bad_request(message: impl Into<String>) -> TxHandlerResult {
    Err((
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            error: message.into(),
        }),
    ))
}
