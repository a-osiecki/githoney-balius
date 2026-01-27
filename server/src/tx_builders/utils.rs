use axum::{http::StatusCode, Json};
use pallas_codec::minicbor::{self, Encode};
use pallas_crypto::hash::Hasher;
use pallas_primitives::{
    conway::{self, Redeemers},
    CostModel, Fragment, NonEmptyKeyValuePairs, PlutusData,
};

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use tokio_tungstenite::tungstenite::client;
use tx3_sdk::trp::{Error, TxEnvelope};

////////////////////////////////////
///// TX evaluate and handlers /////
////////////////////////////////////

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
pub async fn handle_tx_result(tx_result: Result<TxEnvelope, Error>, eval: bool) -> TxHandlerResult {
    match tx_result {
        Ok(tx) => {
            log::info!("Generated tx: hash={}", tx.hash,);
            if !eval {
                return Ok(Json(tx));
            }

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

    update_ex_units(tx, response)
        .await
        .map_err(|e| format!("update_ex_units failed: {e}"))
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

async fn update_ex_units(tx: TxEnvelope, str_json_rpc_resp: String) -> Result<TxEnvelope, String> {
    // Parse evaluateTransaction response
    let parsed: JsonRpcEvalTxResponse =
        serde_json::from_str(&str_json_rpc_resp).map_err(|e| format!("bad json-rpc resp: {e}"))?;

    // No scripts evaluated, return original tx
    if parsed.result.len() == 0 {
        return Ok(tx);
    }

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
    }

    let new_redeemers: Vec<_> = old_redeemers_map.into_iter().collect();
    let inner = NonEmptyKeyValuePairs::from_vec(new_redeemers)
        .ok_or_else(|| "redeemers became empty unexpectedly".to_string())?;
    unwitnessed_tx.transaction_witness_set.redeemer = Some(conway::Redeemers::Map(inner));

    // Recompute script integrity hash
    let datums_vec: Option<Vec<conway::PlutusData>> = unwitnessed_tx
        .transaction_witness_set
        .plutus_data
        .as_ref()
        .map(|nes| nes.iter().cloned().collect());
    let language_view = build_language_view(&unwitnessed_tx).await?;

    let script_integrity_hash = compute_script_data_hash(
        &unwitnessed_tx
            .transaction_witness_set
            .redeemer
            .as_ref()
            .unwrap(),
        datums_vec.as_deref(),
        &language_view,
    );
    unwitnessed_tx.transaction_body.script_data_hash = Some(script_integrity_hash);

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

pub type PlutusVersion = u8;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageView(pub PlutusVersion, pub CostModel);
impl<C> Encode<C> for LanguageView {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        ctx: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        match self.0 {
            // PlutusV1 special encoding (version 0) :contentReference[oaicite:4]{index=4}
            0 => {
                let mut inner = vec![];
                let mut sub = minicbor::Encoder::new(&mut inner);

                sub.begin_array()
                    .map_err(minicbor::encode::Error::message)?;
                for v in self.1.iter() {
                    sub.encode_with(v, ctx)
                        .map_err(minicbor::encode::Error::message)?;
                }
                sub.end().map_err(minicbor::encode::Error::message)?;

                e.map(1)?;
                // key is bytes(CBOR(0)) :contentReference[oaicite:5]{index=5}
                e.bytes(&minicbor::to_vec(0).map_err(minicbor::encode::Error::message)?)?;
                e.bytes(&inner)?;
                Ok(())
            }
            // PlutusV2/V3/... :contentReference[oaicite:6]{index=6}
            _ => {
                e.map(1)?;
                e.encode(self.0)?;
                e.encode(&self.1)?;
                Ok(())
            }
        }
    }
}

/// Compute the Conway script_data_hash exactly like pallas_txbuilder does:
/// CBOR(redeemers) ++ CBOR(datums?) ++ CBOR(language_view), then blake2b-256. :contentReference[oaicite:7]{index=7}
pub fn compute_script_data_hash(
    redeemers: &Redeemers,
    datums: Option<&[PlutusData]>,
    language_view: &LanguageView,
) -> pallas_crypto::hash::Hash<32> {
    let mut buf = Vec::<u8>::new();

    minicbor::encode(redeemers, &mut buf).unwrap(); // infallible in pallas :contentReference[oaicite:8]{index=8}
    if let Some(d) = datums {
        minicbor::encode(d, &mut buf).unwrap(); // infallible in pallas :contentReference[oaicite:9]{index=9}
    }
    minicbor::encode(language_view, &mut buf).unwrap(); // infallible in pallas :contentReference[oaicite:10]{index=10}

    Hasher::<256>::hash(&buf) // :contentReference[oaicite:11]{index=11}
}

async fn build_language_view(tx: &conway::Tx) -> Result<LanguageView, String> {
    let wit = &tx.transaction_witness_set;

    if wit.plutus_v2_script.is_some() {
        let cm = ogmios_get_plutus_v2_cost_model().await.unwrap();
        Ok(LanguageView(2, cm))
    } else if wit.plutus_v1_script.is_some() {
        let cm = ogmios_get_plutus_v2_cost_model().await.unwrap();
        Ok(LanguageView(1, cm))
    } else {
        Err("No Plutus V2 scripts found in transaction".to_string())
    }
}

use std::collections::HashMap;

// ---- Ogmios response types (minimal) ----

#[derive(Deserialize)]
struct RpcResponse<T> {
    result: T,
    id: String,
}

#[derive(Deserialize)]
struct ProtocolParams {
    #[serde(rename = "plutusCostModels")]
    plutus_cost_models: HashMap<String, Vec<i64>>,
}
pub async fn ogmios_get_pp_raw() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    let url =
        std::env::var("OGMIOS_ENDPOINT").map_err(|_| "Missing OGMIOS_ENDPOINT".to_string())?;
    let ogmios_api_key = std::env::var("DMTR_API_KEY_OGMIOS")
        .map_err(|_| "Missing DMTR_API_KEY_OGMIOS".to_string())?;

    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "queryLedgerState/protocolParameters",
        "params": {},
        "id": "pp"
    });

    let resp_text = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("dmtr-api-key", ogmios_api_key)
        .body(serde_json::to_vec(&body)?)
        .send()
        .await?
        .error_for_status()? // important: fail on non-2xx
        .text()
        .await?;

    Ok(resp_text)
}

#[derive(Debug, Deserialize)]
struct RpcResp<T> {
    result: T,
}

#[derive(Debug, Deserialize)]
struct ProtocolParameters {
    #[serde(rename = "plutusCostModels")]
    plutus_cost_models: HashMap<String, Vec<i64>>,
}

fn extract_plutus_v2_cost_model(pp_json: &str) -> Result<Vec<i64>, String> {
    let resp: RpcResp<ProtocolParameters> =
        serde_json::from_str(pp_json).map_err(|e| format!("bad pp json: {e}"))?;

    resp.result
        .plutus_cost_models
        .get("plutus:v2")
        .cloned()
        .ok_or_else(|| "missing plutus:v2 in plutusCostModels".to_string())
}

async fn ogmios_get_plutus_v2_cost_model(
) -> Result<Vec<i64>, Box<dyn std::error::Error + Send + Sync>> {
    let pp_raw = ogmios_get_pp_raw().await?;
    let v2 = extract_plutus_v2_cost_model(&pp_raw)
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;
    Ok(v2)
}
