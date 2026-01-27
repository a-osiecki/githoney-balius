use serde::Deserialize;
use std::collections::HashMap;
use tokio::sync::OnceCell;

static PROTOCOL_PARAMS_CACHE: OnceCell<ProtocolParams> = OnceCell::const_new();
static OGMIOS_CLIENT: OnceCell<reqwest::Client> = OnceCell::const_new();

pub async fn evaluate_transaction(base16_cbor: &str) -> Result<String, String> {
    let url = ogmios_endpoint()?;
    let ogmios_api_key = ogmios_api_key()?;
    let client = ogmios_client().await?;

    ogmios_evaluate(client, &url, &ogmios_api_key, base16_cbor)
        .await
        .map_err(|e| format!("ogmios request failed: {e}"))
}

pub async fn protocol_params() -> Result<ProtocolParams, String> {
    let cached = PROTOCOL_PARAMS_CACHE
        .get_or_try_init(fetch_protocol_params)
        .await
        .map_err(|e| format!("failed to get protocol parameters: {e}"))?;
    Ok(cached.clone())
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
        .error_for_status()?
        .text()
        .await?;

    Ok(resp_text)
}

fn ogmios_endpoint() -> Result<String, String> {
    std::env::var("OGMIOS_ENDPOINT").map_err(|_| "Missing OGMIOS_ENDPOINT".to_string())
}

fn ogmios_api_key() -> Result<String, String> {
    std::env::var("DMTR_API_KEY_OGMIOS").map_err(|_| "Missing DMTR_API_KEY_OGMIOS".to_string())
}

async fn ogmios_client() -> Result<reqwest::Client, String> {
    let client = OGMIOS_CLIENT
        .get_or_try_init(|| async {
            reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .map_err(|e| format!("failed to build http client: {e}"))
        })
        .await?;
    Ok(client.clone())
}

#[derive(Clone, Deserialize)]
pub struct ProtocolParams {
    #[serde(rename = "plutusCostModels")]
    pub plutus_cost_models: HashMap<String, Vec<i64>>,
}

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

#[derive(Deserialize)]
struct RpcResponse<T> {
    result: T,
    id: String,
}

async fn fetch_protocol_params() -> Result<ProtocolParams, Box<dyn std::error::Error + Send + Sync>>
{
    let client = ogmios_client().await.map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, format!("ogmios client: {e}"))
    })?;

    let url = ogmios_endpoint().map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("{e}"))
    })?;

    let ogmios_api_key = ogmios_api_key().map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("{e}"))
    })?;

    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "queryLedgerState/protocolParameters",
        "params": {},
        "id": "pp"
    });

    let resp_text = client
        .post(url)
        .timeout(std::time::Duration::from_secs(10))
        .header("Content-Type", "application/json")
        .header("dmtr-api-key", ogmios_api_key)
        .body(serde_json::to_vec(&body)?)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    let parsed: RpcResponse<ProtocolParams> = serde_json::from_str(&resp_text)?;

    if parsed.id != "pp" {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("unexpected ogmios response id: {}", parsed.id),
        )));
    }

    Ok(parsed.result)
}
