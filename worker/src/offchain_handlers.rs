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
        minting_policy_id: &config.minting_policy_id,
    })?);

    do_tx_building_request(protocol_url, body)
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DeployParams {
    pub creation_fee: String,
    pub reward_fee: String,
    pub script: String,
    pub script_version: String,
    pub settings_minting_policy: String,
    pub settings_minting_version: String,
    pub settings_policy_id: String,
    pub settings_token_name: String,
    pub utxo_ref: String,
}

#[derive(Serialize)]
struct DeployParamsExt<'a> {
    #[serde(flatten)]
    _base: &'a DeployParams,
    githoney_payment_credential: &'a String,
    githoney_script: &'a String,
    githoney_staking_credential: &'a String,
    githoneyaddr: &'a String,
}
pub fn publish_settings(
    config: Config<WorkerConfig>,
    params: Params<DeployParams>,
) -> WorkerResult<Json<TxEnvelope>> {
    let protocol_url =
        url::Url::parse(&format!("{}/deploy-settings", &config.tx_builder_base_url)).unwrap();

    let body = Some(serde_json::to_vec(&DeployParamsExt {
        _base: &params.0,
        githoney_payment_credential: &config.githoney_payment_cred,
        githoney_script: &config.githoney_script_bytes,
        githoney_staking_credential: &config.githoney_staking_cred,
        githoneyaddr: &config.githoney_addr,
    })?);

    do_tx_building_request(protocol_url, body)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddParams {
    pub bountyref: String,
    pub rewardamount: String,
    pub settingsref: String,
    pub since: String,
    pub sponsor: String,
    pub until: String,
}

#[derive(Serialize)]
pub struct AddParamsExt<'a> {
    #[serde(flatten)]
    _base: &'a AddParams,
    script: &'a String,
    rewardassetname: &'a String,
    rewardpolicyid: &'a String,
    settingsref: &'a String,
}

pub fn add_funds(
    config: Config<WorkerConfig>,
    params: Params<AddParams>,
) -> WorkerResult<Json<TxEnvelope>> {
    let protocol_url =
        url::Url::parse(&format!("{}/add-funds", &config.tx_builder_base_url)).unwrap();

    let body = Some(serde_json::to_vec(&AddParamsExt {
        _base: &params.0,
        script: &config.githoney_script_address,
        rewardassetname: &"".to_string(),
        rewardpolicyid: &"".to_string(),
        settingsref: &config.validator_ref,
    })?);

    do_tx_building_request(protocol_url, body)
}

