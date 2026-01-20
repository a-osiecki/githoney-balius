use balius_sdk::wit::balius::app as worker;
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
pub struct CreateWithLovelaceParamsExt<'a> {
    #[serde(flatten)]
    _base: &'a CreateWithLovelaceParams,
    githoneyaddr: &'a String,
    script: &'a String,
    admin_payment_key: &'a String,
    settings_ref: &'a String,
    minting_policy_id: &'a String,
}
pub fn create_bounty(
    config: Config<WorkerConfig>,
    params: Params<CreateWithLovelaceParams>,
) -> WorkerResult<Json<TxEnvelope>> {
    let protocol_url =
        url::Url::parse(&format!("{}/create-bounty", &config.tx_builder_base_url)).unwrap();

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

#[derive(Serialize, Deserialize, Clone)]
pub struct DeployParams {
    pub creation_fee: String,
    pub reward_fee: String,
    pub utxo_ref: String,
}

#[derive(Serialize)]
struct DeployParamsExt<'a> {
    #[serde(flatten)]
    _base: &'a DeployParams,
    githoneyaddr: &'a String,
    githoney_payment_credential: &'a String,
    githoney_staking_credential: &'a String,
    githoney_script: &'a String,
    script: &'a String,
    script_version: &'a String,
    settings_minting_policy: &'a String,
    settings_minting_version: &'a String,
    settings_policy_id: &'a String,
    settings_token_name: &'a String,
}
pub fn publish_settings(
    config: Config<WorkerConfig>,
    params: Params<DeployParams>,
) -> WorkerResult<Json<TxEnvelope>> {
    let protocol_url =
        url::Url::parse(&format!("{}/deploy-settings", &config.tx_builder_base_url)).unwrap();

    let body = Some(serde_json::to_vec(&DeployParamsExt {
        _base: &params.0,
        githoneyaddr: &config.githoney_addr,
        githoney_payment_credential: &config.githoney_payment_cred,
        githoney_staking_credential: &config.githoney_staking_cred,
        githoney_script: &config.githoney_script_bytes,
        script_version: &config.githoney_script_version,
        script: &config.settings_address,
        settings_minting_policy: &config.settings_policy_bytes,
        settings_minting_version: &config.settings_policy_version,
        settings_policy_id: &config.settings_policy_hash,
        settings_token_name: &config.settings_token_name,
    })?);

    do_tx_building_request(protocol_url, body)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddParams {
    pub bounty_ref: String,
    pub initial_rewards: String,
    pub reward_amount: String,
    pub since: String,
    pub sponsor: String,
    pub until: String,
}

#[derive(Debug, Serialize)]
pub struct AddParamsExt<'a> {
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
        url::Url::parse(&format!("{}/add-funds", &config.tx_builder_base_url)).unwrap();
    let add_params = AddParamsExt {
        _base: &params.0,
        script: &config.githoney_script_address,
        reward_asset_name: &"".to_string(),
        reward_policy_id: &"".to_string(),
        settings_ref: &config.validator_ref,
    };

    let body = Some(serde_json::to_vec(&add_params)?);

    worker::logging::log(
        worker::logging::Level::Info,
        "info",
        &format!("{:?}", add_params),
    );
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
pub struct CloseUnassignedParamsExt<'a> {
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
    let protocol_url = url::Url::parse(&format!(
        "{}/close-before-contributor",
        &config.tx_builder_base_url
    ))
    .unwrap();

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
pub struct CloseUnassignedSponsoredParamsExt<'a> {
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
    let protocol_url = url::Url::parse(&format!(
        "{}/close-before-contributor-with-reward",
        &config.tx_builder_base_url
    ))
    .unwrap();

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
pub struct CloseAssignedParamsExt<'a> {
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
    let protocol_url = url::Url::parse(&format!(
        "{}/close-after-contributor",
        &config.tx_builder_base_url
    ))
    .unwrap();

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
