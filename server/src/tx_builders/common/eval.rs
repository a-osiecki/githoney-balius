use pallas_primitives::{
    conway::{self},
    Fragment, NonEmptyKeyValuePairs,
};
use std::collections::BTreeMap;
use tx3_sdk::trp::TxEnvelope;

use crate::tx_builders::{ogmios, script_data};

/// Call Ogmios to evaluate a tx and update its redeemer ex-units.
pub async fn evaluate_tx(tx: TxEnvelope) -> Result<TxEnvelope, String> {
    let response = ogmios::evaluate_transaction(&tx.tx).await?;

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

async fn update_ex_units(tx: TxEnvelope, str_json_rpc_resp: String) -> Result<TxEnvelope, String> {
    let eval_results = parse_eval_results(&str_json_rpc_resp)?;
    if eval_results.is_empty() {
        return Ok(tx);
    }

    let mut unwitnessed_tx = decode_tx(&tx.tx)?;
    apply_ex_units(&mut unwitnessed_tx, eval_results)?;
    let updated_hex = rebuild_script_data_hash_and_encode(&mut unwitnessed_tx).await?;

    Ok(TxEnvelope {
        hash: tx.hash,
        tx: updated_hex,
    })
}

fn parse_eval_results(
    str_json_rpc_resp: &str,
) -> Result<Vec<ogmios::EvalResultItem>, String> {
    let parsed: ogmios::JsonRpcEvalTxResponse =
        serde_json::from_str(str_json_rpc_resp).map_err(|e| format!("bad json-rpc resp: {e}"))?;
    Ok(parsed.result)
}

fn decode_tx(tx_hex: &str) -> Result<conway::Tx, String> {
    let tx_bytes = hex::decode(tx_hex).map_err(|e| format!("bad tx hex: {e}"))?;
    conway::Tx::decode_fragment(&tx_bytes).map_err(|e| format!("decode tx failed: {e:?}"))
}

fn apply_ex_units(
    unwitnessed_tx: &mut conway::Tx,
    eval_results: Vec<ogmios::EvalResultItem>,
) -> Result<(), String> {
    let redeemers = unwitnessed_tx
        .transaction_witness_set
        .redeemer
        .clone()
        .ok_or_else(|| "tx has no redeemers to update".to_string())?;
    let old_redeemers: Vec<(conway::RedeemersKey, conway::RedeemersValue)> = match redeemers {
        conway::Redeemers::Map(kv_pairs) => kv_pairs.into_iter().collect(),
        conway::Redeemers::List(_) => {
            return Err("legacy redeemers list format is not supported".to_string());
        }
    };
    let mut old_redeemers_map: BTreeMap<conway::RedeemersKey, conway::RedeemersValue> =
        old_redeemers.into_iter().collect();

    if old_redeemers_map.is_empty() {
        return Err("tx redeemers map is empty (unexpected)".to_string());
    }

    for ogmios::EvalResultItem {
        validator: ogmios::RedeemerPointer { purpose, index },
        budget: ogmios::ExecutionUnits { memory, cpu },
    } in eval_results
    {
        let tag = script_data::parse_redeemer_tag(&purpose)?;
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

    Ok(())
}

async fn rebuild_script_data_hash_and_encode(
    unwitnessed_tx: &mut conway::Tx,
) -> Result<String, String> {
    let datums_vec: Option<Vec<conway::PlutusData>> = unwitnessed_tx
        .transaction_witness_set
        .plutus_data
        .as_ref()
        .map(|nes| nes.iter().cloned().collect());

    let language_views = script_data::build_language_views(unwitnessed_tx).await?;

    let script_integrity_hash = script_data::compute_script_data_hash(
        &unwitnessed_tx
            .transaction_witness_set
            .redeemer
            .as_ref()
            .unwrap(),
        datums_vec.as_deref(),
        &language_views,
    )?;
    unwitnessed_tx.transaction_body.script_data_hash = Some(script_integrity_hash);

    let updated_bytes = unwitnessed_tx
        .encode_fragment()
        .map_err(|e| format!("encode tx failed: {e:?}"))?;
    Ok(hex::encode(updated_bytes))
}
