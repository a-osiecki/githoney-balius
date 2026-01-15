use balius_sdk::{Config, Json, Params, WorkerResult};
use balius_sdk::wit::balius::app::sign;
use serde::{Deserialize, Serialize};

use crate::types::{WorkerConfig};

#[derive(Serialize, Deserialize)]
pub struct SubmitTxParams {
    pub tx_cbor: String,
}

#[derive(Serialize, Deserialize)]
pub struct SubmitResponse {
    pub success: bool,
    pub tx_hash: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SignPayloadParams {
    pub key_name: String,
    pub payload: String, // Hex-encoded payload to sign
}

#[derive(Serialize, Deserialize)]
pub struct SignPayloadResponse {
    pub signature: String,  // Hex-encoded signature
    pub public_key: String, // Hex-encoded public key (for verification)
}


pub fn sign_payload(
    config: Config<WorkerConfig>,
    params: Params<SignPayloadParams>,
) -> WorkerResult<Json<SignPayloadResponse>> {
    // Decode hex payload to bytes
    let payload_bytes = hex::decode(&params.payload)
        .map_err(|e| balius_sdk::Error::Internal(format!("Invalid hex payload: {}", e)))?;

    // Sign the payload using the WIT sign interface
    let signature = sign::sign_payload(&params.key_name, &payload_bytes)
        .map_err(|e| balius_sdk::Error::Internal(format!("Sign error: {:?}", e)))?;

    // Return signature with public key from config
    Ok(Json(SignPayloadResponse {
        signature: hex::encode(&signature),
        public_key: config.payment_key_public.clone(),
    }))
}
