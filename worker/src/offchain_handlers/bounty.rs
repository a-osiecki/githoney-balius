use balius_sdk::{Config, Json, Params, WorkerResult};

use crate::{
    types::{TxEnvelope, WorkerConfig},
    utils::do_tx_building_request,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateWithLovelaceParams {
    pub bounty_creation_fee: String,
    pub bounty_id: String,
    pub bounty_rewards_fee: String,
    pub maintainer: String,
    pub maintainer_payment_key: String,
    pub maintainer_stake_key: String,
    pub min_ada: String,
    pub reward_amount: String,
    pub since: String,
    pub time_limit: String,
    pub until: String,
}

#[derive(Serialize)]
struct CreateWithLovelaceParamsExt<'a> {
    #[serde(flatten)]
    _base: &'a CreateWithLovelaceParams,
    githoneyaddr: &'a String,
    script: &'a String,
    admin_payment_key: &'a String,
    settings_ref: &'a String,
    minting_policy_id: &'a String,
}

pub fn create_bounty_with_lovelace(
    config: Config<WorkerConfig>,
    params: Params<CreateWithLovelaceParams>,
) -> WorkerResult<Json<TxEnvelope>> {
    let protocol_url =
        url::Url::parse(&format!("{}/bounty/create", &config.tx_builder_base_url)).unwrap();

    let body = Some(serde_json::to_vec(&CreateWithLovelaceParamsExt {
        _base: &params.0,
        githoneyaddr: &config.githoney_addr,
        script: &config.githoney_script_address,
        admin_payment_key: &config.admin_payment_cred,
        settings_ref: &config.validator_ref,
        minting_policy_id: &config.githoney_script_hash,
    })?);

    do_tx_building_request(protocol_url, body)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWithTokenParams {
    pub bounty_creation_fee: String,
    pub bounty_id: String,
    pub bounty_rewards_fee: String,
    pub maintainer: String,
    pub maintainer_payment_key: String,
    pub maintainer_stake_key: String,
    pub min_ada: String,
    pub reward_amount: String,
    pub reward_asset_name: String,
    pub reward_policy_id: String,
    pub since: String,
    pub time_limit: String,
    pub until: String,
}

#[derive(Serialize)]
pub struct CreateWithTokenParamsExt<'a> {
    #[serde(flatten)]
    _base: &'a CreateWithTokenParams,
    admin_payment_key: &'a String,
    githoneyaddr: &'a String,
    minting_policy_id: &'a String,
    script: &'a String,
    settings_ref: &'a String,
}

pub fn create_bounty_with_token(
    config: Config<WorkerConfig>,
    params: Params<CreateWithTokenParams>
) -> WorkerResult<Json<TxEnvelope>>{
    let protocol_url =
        url::Url::parse(&format!("{}/bounty/create", &config.tx_builder_base_url)).unwrap();

    let body = Some(serde_json::to_vec(&CreateWithTokenParamsExt {
        _base: &params.0,
        githoneyaddr: &config.githoney_addr,
        script: &config.githoney_script_address,
        admin_payment_key: &config.admin_payment_cred,
        settings_ref: &config.validator_ref,
        minting_policy_id: &config.githoney_script_hash,
    })?);

    do_tx_building_request(protocol_url, body)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AddParams {
    pub bounty_ref: String,
    pub initial_rewards: String,
    pub reward_amount: String,
    pub since: String,
    pub sponsor: String,
    pub until: String,
}

#[derive(Serialize, Debug)]
struct AddParamsExt<'a> {
    #[serde(flatten)]
    _base: &'a AddParams,
    script: &'a String,
    reward_asset_name: &'a String,
    reward_policy_id: &'a String,
    settings_ref: &'a String,
}

pub fn add_funds(
    config: Config<WorkerConfig>,
    params: Params<AddParams>,
) -> WorkerResult<Json<TxEnvelope>> {
    let protocol_url =
        url::Url::parse(&format!("{}/bounty/add-funds", &config.tx_builder_base_url)).unwrap();
    let add_params = AddParamsExt {
        _base: &params.0,
        script: &config.githoney_script_address,
        reward_asset_name: &"".to_string(),
        reward_policy_id: &"".to_string(),
        settings_ref: &config.validator_ref,
    };

    let body = Some(serde_json::to_vec(&add_params)?);

    do_tx_building_request(protocol_url, body)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AssignParams {
    pub bounty_ref: String,
    pub contributor: String,
    pub contributor_payment_credential: String,
    pub contributor_stake_credential: String,
    pub min_ada: String,
    pub initial_funds: String,
    pub since: String,
    pub until: String,
}

#[derive(Serialize, Debug)]
struct AssignParamsExt<'a> {
    #[serde(flatten)]
    _base: &'a AssignParams,
    script: &'a String,
    settings_ref: &'a String,
}

pub fn assign_contributor(
    config: Config<WorkerConfig>,
    params: Params<AssignParams>,
) -> WorkerResult<Json<TxEnvelope>> {
    let protocol_url =
        url::Url::parse(&format!("{}/bounty/assign", &config.tx_builder_base_url)).unwrap();

    let body = Some(serde_json::to_vec(&AssignParamsExt {
        _base: &params.0,
        script: &config.githoney_script_address,
        settings_ref: &config.validator_ref,
    })?);

    do_tx_building_request(protocol_url, body)
}

#[derive(Serialize, Deserialize)]
pub struct CloseUnassignedParams {
    pub bounty_id: String,
    pub bounty_ref: String,
    pub maintainer: String,
    pub min_ada: String,
    pub reward_amount: String,
    pub since: String,
    pub until: String,
}

#[derive(Serialize)]
struct CloseUnassignedParamsExt<'a> {
    #[serde(flatten)]
    _base: &'a CloseUnassignedParams,
    script: &'a String,
    admin: &'a String,
    settings_ref: &'a String,
    reward_asset_name: &'a String,
    reward_policy_id: &'a String,
    minting_policy_id: &'a String,
}

pub fn close_unassigned(
    config: Config<WorkerConfig>,
    params: Params<CloseUnassignedParams>,
) -> WorkerResult<Json<TxEnvelope>> {
    let protocol_url =
        url::Url::parse(&format!("{}/bounty/close", &config.tx_builder_base_url)).unwrap();

    let body = Some(serde_json::to_vec(&CloseUnassignedParamsExt {
        _base: &params.0,
        script: &config.githoney_script_address,
        admin: &config.admin_address,
        settings_ref: &config.validator_ref,
        reward_asset_name: &"".to_string(),
        reward_policy_id: &"".to_string(),
        minting_policy_id: &config.githoney_script_hash,
    })?);

    do_tx_building_request(protocol_url, body)
}

#[derive(Serialize, Deserialize)]
pub struct CloseUnassignedSponsoredParams {
    pub bounty_id: String,
    pub bounty_ref: String,
    pub maintainer: String,
    pub min_ada: String,
    pub reward_amount: String,
    pub sponsor: String,
    pub refundings_amount: String,
    pub since: String,
    pub until: String,
}

#[derive(Serialize)]
struct CloseUnassignedSponsoredParamsExt<'a> {
    #[serde(flatten)]
    _base: &'a CloseUnassignedSponsoredParams,
    admin: &'a String,
    minting_policy_id: &'a String,
    settings_ref: &'a String,
    reward_asset_name: &'a String,
    reward_policy_id: &'a String,
    refundings_asset_name: &'a String,
    refundings_policy_id: &'a String,
}

pub fn close_unassigned_sponsored(
    config: Config<WorkerConfig>,
    params: Params<CloseUnassignedSponsoredParams>,
) -> WorkerResult<Json<TxEnvelope>> {
    let protocol_url =
        url::Url::parse(&format!("{}/bounty/close", &config.tx_builder_base_url)).unwrap();

    let body = Some(serde_json::to_vec(&CloseUnassignedSponsoredParamsExt {
        _base: &params.0,
        admin: &config.admin_address,
        settings_ref: &config.validator_ref,
        reward_asset_name: &"".to_string(),
        reward_policy_id: &"".to_string(),
        refundings_asset_name: &"".to_string(),
        refundings_policy_id: &"".to_string(),
        minting_policy_id: &config.githoney_script_hash,
    })?);

    do_tx_building_request(protocol_url, body)
}

#[derive(Serialize, Deserialize)]
pub struct CloseAssignedParams {
    pub bounty_id: String,
    pub bounty_ref: String,
    pub contributor: String,
    pub maintainer: String,
    pub min_ada: String,
    pub reward_amount: String,
    pub since: String,
    pub until: String,
}

#[derive(Serialize)]
struct CloseAssignedParamsExt<'a> {
    #[serde(flatten)]
    _base: &'a CloseAssignedParams,
    admin: &'a String,
    minting_policy_id: &'a String,
    settings_ref: &'a String,
    reward_asset_name: &'a String,
    reward_policy_id: &'a String,
}

pub fn close_assigned(
    config: Config<WorkerConfig>,
    params: Params<CloseAssignedParams>,
) -> WorkerResult<Json<TxEnvelope>> {
    let protocol_url =
        url::Url::parse(&format!("{}/bounty/close", &config.tx_builder_base_url)).unwrap();

    let body = Some(serde_json::to_vec(&CloseAssignedParamsExt {
        _base: &params.0,
        admin: &config.admin_address,
        settings_ref: &config.validator_ref,
        reward_asset_name: &"".to_string(),
        reward_policy_id: &"".to_string(),
        minting_policy_id: &config.githoney_script_hash,
    })?);

    do_tx_building_request(protocol_url, body)
}

#[derive(Serialize, Deserialize)]
pub struct CloseAssignedSponsoredParams {
    pub bounty_id: String,
    pub bounty_ref: String,
    pub contributor: String,
    pub maintainer: String,
    pub min_ada: String,
    pub refundings_amount: String,
    pub reward_amount: String,
    pub since: String,
    pub sponsor: String,
    pub until: String,
}

#[derive(Serialize)]
struct CloseAssignedSponsoredParamsExt<'a> {
    #[serde(flatten)]
    _base: &'a CloseAssignedSponsoredParams,
    admin: &'a String,
    minting_policy_id: &'a String,
    refundings_asset_name: &'a String,
    refundings_policy_id: &'a String,
    reward_asset_name: &'a String,
    reward_policy_id: &'a String,
    settings_ref: &'a String,
}

pub fn close_assigned_sponsored(
    config: Config<WorkerConfig>,
    params: Params<CloseAssignedSponsoredParams>,
) -> WorkerResult<Json<TxEnvelope>> {
    let protocol_url =
        url::Url::parse(&format!("{}/bounty/close", &config.tx_builder_base_url)).unwrap();

    let body = Some(serde_json::to_vec(&CloseAssignedSponsoredParamsExt {
        _base: &params.0,
        admin: &config.admin_address,
        settings_ref: &config.validator_ref,
        reward_asset_name: &"".to_string(),
        reward_policy_id: &"".to_string(),
        refundings_asset_name: &"".to_string(),
        refundings_policy_id: &"".to_string(),
        minting_policy_id: &config.githoney_script_hash,
    })?);

    do_tx_building_request(protocol_url, body)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MergeParams {
    pub bounty_ref: String,
    pub githoney_fee: String,
    pub initial_funds: String,
    pub maintainer: String,
    pub min_ada: String,
    pub since: String,
    pub until: String,
}

#[derive(Serialize, Debug)]
struct MergeParamsExt<'a> {
    #[serde(flatten)]
    _base: &'a MergeParams,
    admin: &'a String,
    githoneyaddr: &'a String,
    reward_asset_name: &'a String,
    reward_policy_id: &'a String,
    script: &'a String,
    settings_ref: &'a String,
}

pub fn merge(
    config: Config<WorkerConfig>,
    params: Params<MergeParams>,
) -> WorkerResult<Json<TxEnvelope>> {
    let protocol_url =
        url::Url::parse(&format!("{}/bounty/merge", &config.tx_builder_base_url)).unwrap();
    let merge_params = MergeParamsExt {
        _base: &params.0,
        admin: &config.admin_address,
        githoneyaddr: &config.githoney_addr,
        reward_asset_name: &"".to_string(),
        reward_policy_id: &"".to_string(),
        script: &config.githoney_script_address,
        settings_ref: &config.validator_ref,
    };

    let body = Some(serde_json::to_vec(&merge_params)?);

    do_tx_building_request(protocol_url, body)
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ClaimParams {
    pub bounty_id: String,
    pub bounty_ref: String,
    pub contributor: String,
    pub since: String,
    pub until: String,
}

#[derive(Serialize)]
struct ClaimParamsExt<'a> {
    #[serde(flatten)]
    _base: &'a ClaimParams,
    minting_policy_id: &'a String,
    settings_ref: &'a String,
}

pub fn claim(
    config: Config<WorkerConfig>,
    params: Params<ClaimParams>,
) -> WorkerResult<Json<TxEnvelope>> {
    let protocol_url =
        url::Url::parse(&format!("{}/bounty/claim", &config.tx_builder_base_url)).unwrap();

    let body = Some(serde_json::to_vec(&ClaimParamsExt {
        _base: &params.0,
        minting_policy_id: &config.githoney_script_hash,
        settings_ref: &config.validator_ref,
    })?);

    do_tx_building_request(protocol_url, body)
}
