use axum::Json;
use pallas_primitives::{
    conway::{self, RedeemersKey},
    Fragment, NonEmptyKeyValuePairs,
};
use tx3_sdk::trp::{Error, TxEnvelope};

////////////////////////////////////
///// TX evaluate and handlers /////
////////////////////////////////////
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JsonRpcEvalTxResponse {
    pub jsonrpc: JsonRpcVersion,
    pub method: EvaluateTransactionMethod,
    pub result: Vec<EvalResultItem>,
    pub id: serde_json::Value, // "any"
}

#[derive(Debug, Deserialize)]
pub enum JsonRpcVersion {
    #[serde(rename = "2.0")]
    V2,
}

#[derive(Debug, Deserialize)]
pub enum EvaluateTransactionMethod {
    #[serde(rename = "evaluateTransaction")]
    EvaluateTransaction,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvalResultItem {
    #[serde(rename = "redeemerPointer")]
    pub redeemer_pointer: RedeemerPointer,

    #[serde(rename = "executionUnits")]
    pub execution_units: ExecutionUnits,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RedeemerPointer {
    // Adjust these fields to the exact schema you have.
    // Common shapes include one or more of: purpose, index, tag, etc.
    // Example (one plausible shape):
    pub purpose: String,
    pub index: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionUnits {
    // Typical naming is "mem" and "steps" (Ogmios-style).
    pub memory: u64,
    pub cpu: u64,
}
async fn ogmios_evaluate(
    client: reqwest::Client,
    url: &str,
    ogmios_api_key: &str,
    base16_cbor: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "evaluateTransaction",
        "params": { "transaction": { "cbor": base16_cbor } }
    });

    let resp_text = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("dmtr-api-key", ogmios_api_key)
        .body(serde_json::to_vec(&body)?)
        .send()
        .await?
        .text()
        .await?;

    Ok(resp_text)
}

async fn evaluate_tx(tx: TxEnvelope) -> Result<TxEnvelope, String> {
    let evaluate_url = std::env::var("OGMIOS_ENDPOINT").unwrap();
    let ogmios_api_key = std::env::var("DMTR_API_KEY_OGMIOS").unwrap();
    let client = reqwest::Client::new();

    match ogmios_evaluate(client, &evaluate_url, &ogmios_api_key, &tx.tx).await {
        Ok(response) => {
            if response.contains("Some of the scripts failed") || response.contains("Unauthorized")
            {
                log::error!("Error evaluating transaction: {}", response);
                return Err(response);
            }
            log::info!("Transaction evaluated successfully: {}", response);
            let updated_tx = update_ex_units(tx, response);
            Ok(updated_tx)
        }
        Err(e) => {
            log::error!("Error evaluating transaction: {:?}", e);
            Err(format!("Error evaluating transaction: {:?}", e))
        }
    }
}

fn update_ex_units(tx: TxEnvelope, str_json_rpc_resp: String) -> TxEnvelope {
    // Parse evaluate response and extract execution units
    let parsed: JsonRpcEvalTxResponse = serde_json::from_str(&str_json_rpc_resp).unwrap();
    let JsonRpcEvalTxResponse { result, .. } = parsed;

    // Decode the original transaction
    let tx_bytes = hex::decode(&tx.tx)
        .map_err(|e| {
            log::error!("Failed to decode tx hex: {:?}", e);
        })
        .unwrap();
    let mut unwitnessed_tx = conway::Tx::decode_fragment(&tx_bytes)
        .map_err(|e| {
            log::error!("Failed to decode transaction fragment: {:?}", e);
        })
        .unwrap();
    let old_redeemers = unwitnessed_tx
        .transaction_witness_set
        .redeemer
        .clone()
        .unwrap();

    let mut redeemers_vec: Vec<(conway::RedeemersKey, conway::RedeemersValue)> = vec![];
    for eval_item in result {
        let EvalResultItem {
            redeemer_pointer,
            execution_units,
        } = eval_item;
        let RedeemerPointer { purpose, index } = redeemer_pointer;
        let ExecutionUnits { memory, cpu } = execution_units;

        let cur_redeemers_key = match purpose.as_str() {
            "spend" => RedeemersKey {
                tag: conway::RedeemerTag::Spend,
                index,
            },
            "mint" => RedeemersKey {
                tag: conway::RedeemerTag::Mint,
                index,
            },
            _ => {
                log::error!("Unknown redeemer purpose: {}", purpose);
                continue;
            }
        };

        let cur_plutus_data = old_redeemers.encode_fragment();
        let cur_redeemers_value = conway::RedeemersValue {
            ex_units: conway::ExUnits {
                mem: memory,
                steps: cpu,
            },
            data: ,
        };
    }

    let redeemers_inner: NonEmptyKeyValuePairs<conway::RedeemersKey, conway::RedeemersValue> =
        NonEmptyKeyValuePairs::from_vec(redeemers_vec).unwrap();
    let redeemers = conway::Redeemers::Map(redeemers_inner);
    unwitnessed_tx.transaction_witness_set.redeemer = Some(redeemers);

    let updated_tx_bytes = unwitnessed_tx.encode_fragment().unwrap();
    let updated_tx_hex = hex::encode(updated_tx_bytes);

    TxEnvelope {
        hash: tx.hash,
        tx: updated_tx_hex,
    }
}

pub async fn handle_tx_result(
    tx_result: Result<TxEnvelope, Error>,
) -> Json<Result<TxEnvelope, String>> {
    match tx_result {
        Ok(tx) => {
            log::info!("Generated CBOR: {}", tx.tx);
            let evaluated_tx_or_err = evaluate_tx(tx).await;
            Json(evaluated_tx_or_err)
        }
        Err(e) => {
            log::error!("Error building transaction: {:?}", e);
            Json(Err(format!("Error building transaction: {:?}", e)))
        }
    }
}
