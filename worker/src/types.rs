use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct WorkerConfig {
    // Infra config
    pub project_id: String, // Blockfrost
    pub payment_key_public: String,
    pub webhook_url: String,
    pub monitoring_address: String,
    pub tx_builder_base_url: String,
    // Githoney specific config
    pub admin_address: String,
    pub admin_payment_cred: String,
    pub githoney_script_address: String,
    pub githoney_script_bytes: String,
    pub githoney_script_hash: String,
    pub githoney_script_version: String,
    // Settings script config
    pub settings_address: String,
    pub settings_policy_bytes: String,
    pub settings_policy_hash: String,
    pub settings_policy_version: String,
    pub settings_token_name: String,
    pub validator_ref: String,
    // Maintainer addresses
    pub githoney_addr: String,
    pub githoney_payment_cred: String,
    pub githoney_staking_cred: String,
    //Badges Config
    pub script_badge: String,
}

///// OFFCHAIN PROTOCOL TYPES /////
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxEnvelope {
    #[serde(rename = "hash")]
    pub hash: String,
    #[serde(rename = "tx")]
    pub tx: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}
