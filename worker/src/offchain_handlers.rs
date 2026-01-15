use balius_sdk::wit::balius::app as worker;
use balius_sdk::{http::HttpRequest, Config, Json, Params, WorkerResult};

use crate::types::{TxEnvelope, WorkerConfig};
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

    worker::logging::log(
        worker::logging::Level::Info,
        "create-bounty",
        &format!("create-bounty body: {:?}", params.0),
    );

    let body = Some(serde_json::to_vec(&CreateWithLovelaceParamsExt {
        _base: &params.0,
        githoneyaddr: &config.githoney_addr,
        script: &config.githoney_script_address,
        admin_payment_key: &config.admin_payment_cred,
        settings_ref: &config.validator_ref,
        minting_policy_id: &config.minting_policy_id,
    })?);

    let mut request = HttpRequest::post(protocol_url).header("Content-Type", "application/json");
    request.body = body;

    let response = request
        .send()
        .map_err(|e| balius_sdk::Error::Internal(format!("Protocol request error: {:?}", e)))?;

    let parsed: TxEnvelope = response.json().map_err(|e| {
        balius_sdk::Error::Internal(format!("Protocol response parse error: {:?}", e))
    })?;

    Ok(Json(parsed))
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

    let mut request = HttpRequest::post(protocol_url).header("Content-Type", "application/json");
    request.body = body;

    let response = request
        .send()
        .map_err(|e| balius_sdk::Error::Internal(format!("Protocol request error: {:?}", e)))?;

    let parsed: TxEnvelope = response.json().map_err(|e| {
        balius_sdk::Error::Internal(format!("Protocol response parse error: {:?}", e))
    })?;

    Ok(Json(parsed))
}

// curl -X POST http://127.0.0.1:8080/create-bounty \
//   -H "Content-Type: application/json" \
//   -d '{
//     "admin_payment_key": "42704da3a869894da8e24185fc36fdbd82f6092c85a17b5fc6e52213",
//     "bounty_creation_fee": "1000000",
//     "bounty_id": "deadbeef001",
//     "bounty_rewards_fee": "500000",
//     "maintainer": "addr_test1qpp8qndr4p5cjndgufqctlpklk7c9asf9jz6z76lcmjjyyavuam5ced7vsutn86dghwa46yz8cum5hdc42dv7fedaz6sgkx26d",
//     "maintainer_payment_key": "42704da3a869894da8e24185fc36fdbd82f6092c85a17b5fc6e52213",
//     "maintainer_stake_key": "ace7774c65be6438b99f4d45dddae8823e39ba5db8aa9acf272de8b5",
//     "min_ada": "2000000",
//     "reward_amount": "10000000",
//     "since": "1768502795",
//     "time_limit": "100000",
//     "until": "1768602795"
//   }'
