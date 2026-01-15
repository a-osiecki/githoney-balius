use reqwest::Client;

pub async fn call_evaluate_transaction(
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
    println!("Sending evaluateTransaction request to {} with api key {}...", url, ogmios_api_key);

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

