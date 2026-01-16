use reqwest::Client;
use tx3_sdk::trp::TxEnvelope;

async fn ogmios_evaluate(
    client: Client,
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

pub async fn evaluate_tx(tx: TxEnvelope) -> Result<TxEnvelope, String> {
    let evaluate_url = std::env::var("OGMIOS_ENDPOINT").unwrap();
    let ogmios_api_key = std::env::var("DMTR_API_KEY_OGMIOS").unwrap();
    let client = reqwest::Client::new();

    match ogmios_evaluate(client, &evaluate_url, &ogmios_api_key, &tx.tx).await {
        Ok(response) => {
            println!("Transaction evaluated successfully: {}", response);
            if response.contains("Some of the scripts failed") {
                return Err(response);
            }
            Ok(tx)
        }
        Err(e) => {
            println!("Error evaluating transaction: {:?}", e);
            Err(format!("Error evaluating transaction: {:?}", e))
        }
    }
}
