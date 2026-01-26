use axum::{http::StatusCode, Json};
use pallas_primitives::{
    conway::{self},
    Fragment, NonEmptyKeyValuePairs,
};

use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use tx3_sdk::trp::{Error, TxEnvelope};

////////////////////////////////////
///// TX evaluate and handlers /////
////////////////////////////////////
use serde::Deserialize;

#[derive(Deserialize)]
pub struct JsonRpcEvalTxResponse {
    pub result: Vec<EvalResultItem>,
}

#[derive(Debug, Deserialize)]
pub struct EvalResultItem {
    pub validator: RedeemerPointer,

    pub budget: ExecutionUnits,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RedeemerPointer {
    pub purpose: String,
    pub index: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionUnits {
    pub memory: u64,
    pub cpu: u64,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub type TxHandlerResult = Result<Json<TxEnvelope>, (StatusCode, Json<ErrorResponse>)>;

/// Handle the result of a tx building operation, evaluate it via Ogmios, and return the final tx or an error.
pub async fn handle_tx_result(tx_result: Result<TxEnvelope, Error>) -> TxHandlerResult {
    match tx_result {
        Ok(tx) => {
            log::info!("Generated tx: hash={}", tx.hash,);
            match evaluate_tx(tx).await {
                Ok(evaluated_tx) => Ok(Json(evaluated_tx)),
                Err(e) => {
                    log::error!("Error evaluating transaction: {}", e);
                    Err((
                        StatusCode::BAD_GATEWAY,
                        Json(ErrorResponse {
                            error: format!("Error evaluating transaction: {e}"),
                        }),
                    ))
                }
            }
        }
        Err(e) => {
            log::error!("Error building transaction: {:?}", e);
            Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Error building transaction: {e:?}"),
                }),
            ))
        }
    }
}

/// Call Ogmios to evaluate a tx and update its redeemer ex-units.
async fn evaluate_tx(tx: TxEnvelope) -> Result<TxEnvelope, String> {
    let evaluate_url =
        std::env::var("OGMIOS_ENDPOINT").map_err(|_| "Missing OGMIOS_ENDPOINT".to_string())?;
    let ogmios_api_key = std::env::var("DMTR_API_KEY_OGMIOS")
        .map_err(|_| "Missing DMTR_API_KEY_OGMIOS".to_string())?;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("failed to build http client: {e}"))?;

    let response = ogmios_evaluate(client, &evaluate_url, &ogmios_api_key, &tx.tx)
        .await
        .map_err(|e| format!("ogmios request failed: {e}"))?;

    if response.is_empty() {
        return Err("ogmios returned empty response".to_string());
    }

    if response.contains("Some of the scripts failed") || response.contains("Unauthorized") {
        return Err(format!("ogmios evaluate error: {response}"));
    }

    log::info!("Transaction evaluated successfully: {}", response);

    update_ex_units(tx, response).map_err(|e| format!("update_ex_units failed: {e}"))
}

/// Send an evaluateTransaction request to Ogmios and return the raw response body.
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

    let resp = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("dmtr-api-key", ogmios_api_key)
        .body(serde_json::to_vec(&body)?)
        .send()
        .await?;

    let status = resp.status();
    let resp_text = resp.text().await?;
    log::info!("Ogmios evaluate response: {}", resp_text);
    if !status.is_success() {
        return Err(format!(
            "ogmios http error: status={} info={}",
            status.as_u16(),
            resp_text
        )
        .into());
    }

    Ok(resp_text)
}

fn update_ex_units(tx: TxEnvelope, str_json_rpc_resp: String) -> Result<TxEnvelope, String> {
    // Parse evaluateTransaction response
    let parsed: JsonRpcEvalTxResponse =
        serde_json::from_str(&str_json_rpc_resp).map_err(|e| format!("bad json-rpc resp: {e}"))?;

    // Decode tx bytes
    let tx_bytes = hex::decode(&tx.tx).map_err(|e| format!("bad tx hex: {e}"))?;
    let mut unwitnessed_tx =
        conway::Tx::decode_fragment(&tx_bytes).map_err(|e| format!("decode tx failed: {e:?}"))?;

    // Extract existing redeemers
    let redeemers = unwitnessed_tx
        .transaction_witness_set
        .redeemer
        .clone()
        .ok_or_else(|| "tx has no redeemers to update".to_string())?;
    let old_redeemers: Vec<(conway::RedeemersKey, conway::RedeemersValue)> = match redeemers {
        conway::Redeemers::Map(kv_pairs) => kv_pairs.into_iter().collect(),
        conway::Redeemers::List(_) => {
            return Err("Found legacy redeemers, expected Conway format".to_string());
        }
    };
    let mut old_redeemers_map: BTreeMap<conway::RedeemersKey, conway::RedeemersValue> =
        old_redeemers.into_iter().collect();

    if old_redeemers_map.is_empty() {
        return Err("tx redeemers map is empty (unexpected)".to_string());
    }

    let mut updated_keys: BTreeSet<conway::RedeemersKey> = BTreeSet::new();
    for EvalResultItem {
        validator: RedeemerPointer { purpose, index },
        budget: ExecutionUnits { memory, cpu },
    } in parsed.result
    {
        let tag = match purpose.as_str() {
            "spend" => conway::RedeemerTag::Spend,
            "mint" => conway::RedeemerTag::Mint,
            other => return Err(format!("unknown redeemer purpose: {other}")),
        };

        let key = conway::RedeemersKey { tag, index };

        let val = old_redeemers_map
            .get_mut(&key)
            .ok_or_else(|| format!("Evaluation returned redeemer not present in tx: {:?}", key))?;

        val.ex_units = conway::ExUnits {
            mem: memory,
            steps: cpu,
        };

        updated_keys.insert(key);
    }

    let new_redeemers: Vec<_> = old_redeemers_map.into_iter().collect();
    let inner = NonEmptyKeyValuePairs::from_vec(new_redeemers)
        .ok_or_else(|| "redeemers became empty unexpectedly".to_string())?;
    unwitnessed_tx.transaction_witness_set.redeemer = Some(conway::Redeemers::Map(inner));

    // Encode updated tx
    let updated_bytes = unwitnessed_tx
        .encode_fragment()
        .map_err(|e| format!("encode tx failed: {e:?}"))?;
    let updated_hex = hex::encode(updated_bytes);

    Ok(TxEnvelope {
        hash: tx.hash,
        tx: updated_hex,
    })
}
