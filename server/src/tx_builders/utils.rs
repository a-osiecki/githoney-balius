use axum::Json;
use tx3_sdk::trp::{Error, TxEnvelope};

////////////////////////////////////
///// TX evaluate and handlers /////
////////////////////////////////////
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
            Ok(tx)
        }
        Err(e) => {
            log::error!("Error evaluating transaction: {:?}", e);
            Err(format!("Error evaluating transaction: {:?}", e))
        }
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
