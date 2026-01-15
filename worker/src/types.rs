use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct WorkerConfig {
    // Blockfrost config
    pub project_id: String,
    pub payment_key_public: String,
    pub webhook_url: String,
    pub monitoring_address: String,
    // Githoney specific config
    pub admin_payment_cred: String,
    pub githoney_script_address: String,
    pub githoney_script_bytes: String,
    pub githoney_addr: String,
    pub githoney_payment_cred: String,
    pub githoney_staking_cred: String,
    pub tx_builder_base_url: String,
    pub validator_ref: String,
    pub minting_policy_id: String,
}

///// OFFCHAIN PROTOCOL TYPES /////
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxEnvelope {
    #[serde(rename = "hash")]
    pub hash: String,
    #[serde(rename = "tx")]
    pub tx: String,
}
