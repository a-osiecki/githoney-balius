use balius_sdk::{Config, Json, Params, WorkerResult};

use crate::{
    types::{TxEnvelope, WorkerConfig},
    utils::do_tx_building_request,
};
use serde::{Deserialize, Serialize};

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
        url::Url::parse(&format!("{}/settings/deploy", &config.tx_builder_base_url)).unwrap();

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
pub struct UpdateParams {
    pub bounty_creation_fees: String,
    pub bounty_rewards_fee: String,
    pub githoney_payment_key: String,
    pub githoney_staking_key: String,
    pub githoneyaddr: String,

}

#[derive(Serialize)]
pub struct UpdateParamsExt<'a> {
    #[serde(flatten)]
    _base: &'a UpdateParams,
    githoney_script: &'a String,
    settings_ref: &'a String,
    script: &'a String,
    script_version: &'a String,
    settings_validator_script: &'a String,
    settings_validator_version: &'a String,
}

pub fn update_settings(
    config: Config<WorkerConfig>,
    params: Params<UpdateParams>
) -> WorkerResult<Json<TxEnvelope>>{
    let protocol_url =
        url::Url::parse(&format!("{}/settings/update", &config.tx_builder_base_url)).unwrap();

    let body = Some(serde_json::to_vec(&UpdateParamsExt{
        _base: &params.0,
        githoney_script: &config.githoney_script_bytes,
        settings_ref: &config.validator_ref,
        script: &config.settings_address,
        script_version: &config.githoney_script_version,
        settings_validator_script: &config.settings_policy_bytes,
        settings_validator_version: &config.settings_policy_version,
    })?);

    do_tx_building_request(protocol_url, body)
}