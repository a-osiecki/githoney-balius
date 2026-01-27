use balius_sdk::{Config, Json, Params, WorkerResult};

use crate::{
    types::{TxEnvelope, WorkerConfig},
    utils::do_tx_building_request,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MintBadgeParams {
    pub badge_policy_script: String,
    pub badge_policy_script_version: String,
    pub description: String,
    pub description_value: String,
    pub ft_badge_amount: String,
    pub ft_badge_name: String,
    pub ftaddress: String,
    pub logo: String,
    pub logo_value: String,
    pub m_version: String,
    pub name: String,
    pub name_value: String,
    pub utxo_ref: String,
    pub minting_policy_id: String,
    pub ref_nft_asset_name: String,
}

#[derive(Serialize)]
struct MintBadgeParamsExt<'a> {
    #[serde(flatten)]
    _base: &'a MintBadgeParams,
    githoneyaddr: &'a String,
    scriptbadge: &'a String,
}

pub fn mint_badge(
    config: Config<WorkerConfig>,
    params: Params<MintBadgeParams>,
) -> WorkerResult<Json<TxEnvelope>> {
    let protocol_url =
        url::Url::parse(&format!("{}/badge/mint", &config.tx_builder_base_url)).unwrap();

    let body = Some(serde_json::to_vec(&MintBadgeParamsExt {
        _base: &params.0,
        githoneyaddr: &config.githoney_addr,
        scriptbadge: &config.script_badge,
    })?);

    do_tx_building_request(protocol_url, body)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateBadgeParams {
    pub description: String,
    pub description_value: String,
    pub logo: String,
    pub logo_value: String,
    pub m_version: String,
    pub name: String,
    pub name_value: String,
    pub badge_utxo_ref: String,
}

#[derive(Serialize)]
struct UpdateBadgeParamsExt<'a> {
    #[serde(flatten)]
    _base: &'a UpdateBadgeParams,
    scriptbadge: &'a String,
    settings_ref: &'a String,
    badges_script: &'a String,
    badges_script_version: &'a String,
    githoneyaddr: &'a String,
}

pub fn update_badge(
    config: Config<WorkerConfig>,
    params: Params<UpdateBadgeParams>,
) -> WorkerResult<Json<TxEnvelope>> {
    let protocol_url =
        url::Url::parse(&format!("{}/badge/update", &config.tx_builder_base_url)).unwrap();

    let body = Some(serde_json::to_vec(&UpdateBadgeParamsExt {
        _base: &params.0,
        githoneyaddr: &config.githoney_addr,
        scriptbadge: &config.script_badge,
        badges_script: &config.badges_script,
        badges_script_version: &config.badges_script_version,
        settings_ref: &config.validator_ref,
    })?);

    do_tx_building_request(protocol_url, body)
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayBadgesToParams {
    pub badge_name: String,
    pub badge_policy: String,
    pub ftaddress: String,
    pub payaddress: String,
}

pub fn pay_badges_to(
    config: Config<WorkerConfig>,
    params: Params<PayBadgesToParams>
) -> WorkerResult<Json<TxEnvelope>>{
    let protocol_url = url::Url::parse(&format!(
        "{}/pay-badges-to",
        &config.tx_builder_base_url
    ))
    .unwrap();

    let body = Some(serde_json::to_vec(&params.0)?);

    do_tx_building_request(protocol_url, body)
}
