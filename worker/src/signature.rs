use balius_sdk::wit::balius::app::{sign, submit};
use balius_sdk::wit::balius::app as worker;
use balius_sdk::wit::balius::app::kv::{self, set_value};
use balius_sdk::{Config, Json, Params, WorkerResult};
use pallas_addresses::Address;
use pallas_codec::minicbor;
use pallas_primitives::{conway, Fragment, NonEmptySet, alonzo::VKeyWitness};
use pallas_traverse::MultiEraTx;
use serde::{Deserialize, Serialize};

use crate::types::{TxEnvelope, WorkerConfig};

use crate::utils::TX_STATUS_PENDING;
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
    let mut vkey_witnesses: Vec<VKeyWitness>= unwitnessed_tx
        .transaction_witness_set
        .vkeywitness
        .as_ref()
        .map(|x| x.clone().to_vec())
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

#[derive(Serialize, Deserialize, Clone)]
pub struct SubmitTxParams {
    pub tx_cbor: String,
}

#[derive(Serialize, Deserialize)]
pub struct SubmitResponse {
    pub success: bool,
    pub tx_hash: Option<String>,
}
pub fn _submit_tx(config: Config<WorkerConfig>, params: Params<SubmitTxParams>) -> WorkerResult<Json<SubmitResponse>> {

    let bytes = hex::decode(params.tx_cbor.clone()).unwrap();

    // worker::logging::log(
    //         worker::logging::Level::Debug,
    //         "tx_handler",
    //         &format!("parsed transaction: {:?}", &bytes),
    //     );
    // Adding the transaction to kv database
    let monitoring_addr_bytes =
        pallas_addresses::Address::from_bech32(&config.githoney_script_address)
            .expect("Invalid bech32 monitoring address in config")
            .to_vec();

    let mtx : conway::Tx  = minicbor::decode::<conway::Tx>(&bytes).unwrap();

    let metx = MultiEraTx::from_conway(&mtx);

    let has_monitored_address = metx
        .outputs()
        .iter()
        .any(|output| output.address().unwrap_or_else(|_e| panic!("Failed to obtain output address")) == Address::from_bytes(&monitoring_addr_bytes).unwrap_or_else(|_e| panic!("Failed to parse monitoring address")));

    if has_monitored_address{
        worker::logging::log(
            worker::logging::Level::Debug,
            "tx_handler",
            &format!("Transaction tracked: {}", &metx.hash().to_string()),
        );
        worker::logging::log(
                worker::logging::Level::Debug,
                "tx_handler",
                &format!("Transaction is in DB: {:?}", kv::get_value(&metx.hash().to_string())),
            );

        if kv::get_value(&metx.hash().to_string()).unwrap_or_default() == TX_STATUS_PENDING.as_bytes(){
            worker::logging::log(
                worker::logging::Level::Debug,
                "tx_handler",
                &format!("Transaction is in DB: {:?}", kv::get_value(&metx.hash().to_string())),
            );
        }

        if let Err(e) = kv::set_value(&metx.hash().to_string(), TX_STATUS_PENDING.as_bytes()) {
            worker::logging::log(
                worker::logging::Level::Error,
                "tx_handler",
                &format!("Failed to insert tx status: {:?}", e),
            );
        }
    }

    if let Err(e) = submit::submit_tx(&bytes){
        worker::logging::log(
            worker::logging::Level::Error,
            "tx-handler",
            &format!("Failet to submit transaction {:?}", e),
        );

        return Ok(Json(SubmitResponse { success: false, tx_hash: None }))
    }

    Ok(Json(SubmitResponse{
        success: true,
        tx_hash: Some(metx.hash().to_string())
    }))
}

