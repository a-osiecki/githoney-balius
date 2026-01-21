use balius_sdk::wit::balius::app::sign;
use balius_sdk::{Config, Json, Params, WorkerResult};
use pallas_primitives::{conway, Fragment, NonEmptySet};
use serde::{Deserialize, Serialize};

use crate::types::{TxEnvelope, WorkerConfig};

#[derive(Serialize, Deserialize)]
pub struct SignTransactionParams {
    pub key_name: String,
    pub payload: TxEnvelope,
}

#[derive(Serialize, Deserialize)]
pub struct SignTransactionResponse {
    pub signed_tx: String, // Hex-encoded signed transaction
}

pub fn sign_tx(
    config: Config<WorkerConfig>,
    params: Params<SignTransactionParams>,
) -> WorkerResult<Json<SignTransactionResponse>> {
    // Decode hex payload to bytes
    let tx_bytes = hex::decode(&params.payload.tx)
        .map_err(|e| balius_sdk::Error::Internal(format!("Invalid tx hex in params: {}", e)))?;
    let tx_hash_bytes = hex::decode(&params.payload.hash)
        .map_err(|e| balius_sdk::Error::Internal(format!("Invalid hex payload: {}", e)))?;

    // Sign the payload using the WIT sign interface
    let signature = sign::sign_payload(&params.key_name, &tx_hash_bytes)
        .map_err(|e| balius_sdk::Error::Internal(format!("Sign error: {:?}", e)))?;

    // Get public key bytes from config
    let pub_key_bytes = hex::decode(&config.payment_key_public.clone()).map_err(|e| {
        balius_sdk::Error::Internal(format!("Invalid public key hex in config: {}", e))
    })?;

    // Deserialize the transaction
    let mut unwitnessed_tx = conway::Tx::decode_fragment(&tx_bytes).map_err(|e| {
        balius_sdk::Error::Internal(format!("Failed to decode transaction fragment: {:?}", e))
    })?;

    // Build VKeyWitness set
    let vkey_witness = conway::VKeyWitness {
        vkey: pub_key_bytes.into(),
        signature: signature.clone().into(),
    };
    let mut vkey_witnesses = unwitnessed_tx
        .transaction_witness_set
        .vkeywitness
        .map(|x| x.to_vec())
        .unwrap_or_default();
    vkey_witnesses.push(vkey_witness);

    // Set new VKeyWitness and serialize again
    unwitnessed_tx.transaction_witness_set.vkeywitness =
        Some(NonEmptySet::from_vec(vkey_witnesses).unwrap());
    let signed_tx = unwitnessed_tx.encode_fragment().map_err(|e| {
        balius_sdk::Error::Internal(format!("Failed to encode signed transaction: {:?}", e))
    })?;

    // Return signed tx
    Ok(Json(SignTransactionResponse {
        signed_tx: hex::encode(&signed_tx),
    }))
}

#[derive(Serialize, Deserialize)]
pub struct SubmitTxParams {
    pub tx_cbor: String,
}

#[derive(Serialize, Deserialize)]
pub struct SubmitResponse {
    pub success: bool,
    pub tx_hash: Option<String>,
}
fn _submit_tx(_config: Config<WorkerConfig>, _params: Params<SubmitTxParams>) -> () {}
