use balius_sdk::{Ack, Config, FnHandler, Json, Params, Tx, Worker, WorkerResult};
// use balius_sdk::wit::balius::app::submit;
// use balius_sdk::wit::balius::app::sign;
use balius_sdk::http::{HttpError, HttpRequest, HttpResponse};
use balius_sdk::wit::balius::app as worker;
use balius_sdk::wit::balius::app::driver::UtxoPattern;
use balius_sdk::wit::balius::app::kv;
use balius_sdk::wit::balius::app::sign;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
struct BlockfrostConfig {
    project_id: String,
    payment_key_public: String,
    webhook_url: String,
    monitoring_address: String,
}

#[derive(Serialize, Deserialize)]
struct EmptyParams {}

#[derive(Serialize, Deserialize)]
struct BlockfrostResponse {
    #[serde(flatten)]
    data: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
struct SubmitTxParams {
    tx_cbor: String,
}

#[derive(Serialize, Deserialize)]
struct SubmitResponse {
    success: bool,
    tx_hash: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct SignPayloadParams {
    key_name: String,
    payload: String, // Hex-encoded payload to sign
}

#[derive(Serialize, Deserialize)]
struct SignPayloadResponse {
    signature: String,  // Hex-encoded signature
    public_key: String, // Hex-encoded public key (for verification)
}

#[derive(Serialize, Deserialize)]
struct SignTransactionParams {
    key_name: String,
    tx_cbor: String, // Hex-encoded CBOR transaction
}

#[derive(Serialize, Deserialize)]
struct SignTransactionResponse {
    signed_tx_cbor: String, // Hex-encoded signed CBOR transaction
    tx_hash: String,
}

#[derive(Serialize, Deserialize)]
struct SignAndSubmitParams {
    key_name: String,
    tx_cbor: String, // Hex-encoded CBOR transaction
}

#[derive(Serialize, Deserialize)]
struct SignAndSubmitResponse {
    success: bool,
    signed_tx_cbor: String,
    tx_hash: String,
}

// Webhook payload sent to the confirmation server
#[derive(Serialize, Deserialize)]
struct WebhookPayload {
    tx_hash: String,
    block_hash: String,
    block_height: u64,
    block_slot: u64,
}

// Transaction tracking status
const TX_STATUS_PENDING: &str = "pending";
const TX_STATUS_CONFIRMED: &str = "confirmed";

// Helper function to send webhook notification
fn send_confirmation_webhook(
    webhook_url: &str,
    tx_hash: &str,
    block_hash: &str,
    block_height: u64,
    block_slot: u64,
) -> WorkerResult<()> {
    let payload = WebhookPayload {
        tx_hash: tx_hash.to_string(),
        block_hash: block_hash.to_string(),
        block_height,
        block_slot,
    };

    worker::logging::log(
        worker::logging::Level::Info,
        "webhook",
        &format!("Sending confirmation webhook for tx: {}", tx_hash),
    );

    let url = url::Url::parse(webhook_url)
        .map_err(|e| balius_sdk::Error::Internal(format!("Invalid webhook URL: {}", e)))?;

    let response = balius_sdk::http::HttpRequest::post(url)
        .header("Content-Type", "application/json")
        .json(&payload)?
        .send()?;

    if !response.is_ok() {
        worker::logging::log(
            worker::logging::Level::Error,
            "webhook",
            &format!("Webhook failed with status: {}", response.status),
        );
        return Err(balius_sdk::Error::Internal(format!(
            "Webhook request failed with status {}",
            response.status
        )));
    }

    worker::logging::log(
        worker::logging::Level::Info,
        "webhook",
        &format!("Successfully sent webhook for tx: {}", tx_hash),
    );

    Ok(())
}

// Transaction event handler - processes transactions involving the monitored address
fn handle_transaction_event(config: Config<BlockfrostConfig>, tx_event: Tx) -> WorkerResult<Ack> {
    let tx_hash = hex::encode(&tx_event.hash);

    // Decode monitoring address from config
    let monitoring_addr_bytes = pallas_addresses::Address::from_bech32(&config.monitoring_address)
        .expect("Invalid bech32 monitoring address in config")
        .to_vec();

    worker::logging::log(
        worker::logging::Level::Info,
        "tx_handler",
        &format!(
            "=== TX EVENT RECEIVED: {} (block: {}, slot: {}) ===",
            tx_hash, tx_event.block_height, tx_event.block_slot
        ),
    );

    // Manual filtering: Check if any input matches the monitoring address
    let has_monitored_address = tx_event
        .tx
        .inputs
        .iter()
        .filter_map(|input| input.as_output.as_ref())
        .any(|output| output.address.to_vec() == monitoring_addr_bytes);

    if !has_monitored_address {
        worker::logging::log(
            worker::logging::Level::Debug,
            "tx_handler",
            &format!(
                "Transaction does not involve monitoring address, skipping: {}",
                tx_hash
            ),
        );
        return Ok(Ack);
    }

    worker::logging::log(
        worker::logging::Level::Info,
        "tx_handler",
        &format!("Transaction involves monitoring address: {}", tx_hash),
    );

    // Check if this transaction is being tracked
    match kv::get_value(&tx_hash) {
        Ok(status_bytes) => {
            let status = String::from_utf8_lossy(&status_bytes);

            if status == TX_STATUS_PENDING {
                worker::logging::log(
                    worker::logging::Level::Info,
                    "tx_handler",
                    &format!("Found pending transaction confirmed: {}", tx_hash),
                );

                // Extract block information
                let block_hash = hex::encode(&tx_event.block_hash);
                let block_height = tx_event.block_height;
                let block_slot = tx_event.block_slot;

                // Send webhook notification
                if let Err(e) = send_confirmation_webhook(
                    &config.webhook_url,
                    &tx_hash,
                    &block_hash,
                    block_height,
                    block_slot,
                ) {
                    worker::logging::log(
                        worker::logging::Level::Error,
                        "tx_handler",
                        &format!("Failed to send webhook: {:?}", e),
                    );
                    // Continue processing even if webhook fails
                }

                // Update status to confirmed
                if let Err(e) = kv::set_value(&tx_hash, TX_STATUS_CONFIRMED.as_bytes()) {
                    worker::logging::log(
                        worker::logging::Level::Error,
                        "tx_handler",
                        &format!("Failed to update tx status: {:?}", e),
                    );
                }

                worker::logging::log(
                    worker::logging::Level::Info,
                    "tx_handler",
                    &format!("Transaction confirmation processed: {}", tx_hash),
                );
            } else {
                worker::logging::log(
                    worker::logging::Level::Debug,
                    "tx_handler",
                    &format!("Transaction already {}: {}", status, tx_hash),
                );
            }
        }
        Err(_) => {
            // Transaction not being tracked, ignore it
            worker::logging::log(
                worker::logging::Level::Debug,
                "tx_handler",
                &format!("Transaction not tracked: {}", tx_hash),
            );
        }
    }

    Ok(Ack)
}

fn get_latest_block(
    config: Config<BlockfrostConfig>,
    _params: Params<EmptyParams>,
) -> WorkerResult<Json<BlockfrostResponse>> {
    let project_id =
        std::env::var("BLOCKFROST_PROJECT_ID").unwrap_or_else(|_| config.project_id.clone());

    let url = url::Url::parse("https://cardano-preprod.blockfrost.io/api/v0/blocks/latest")
        .map_err(|e| balius_sdk::Error::Internal(format!("Invalid URL: {}", e)))?;

    let response = balius_sdk::http::HttpRequest::get(url)
        .header("project_id", project_id.as_str())
        .send()?;

    if !response.is_ok() {
        return Err(balius_sdk::Error::Internal(format!(
            "HTTP error: status {}",
            response.status
        )));
    }

    let data: serde_json::Value = serde_json::from_slice(&response.body)
        .map_err(|e| balius_sdk::Error::Internal(format!("Failed to parse JSON: {}", e)))?;

    Ok(Json(BlockfrostResponse { data }))
}

fn sign_payload(
    config: Config<BlockfrostConfig>,
    params: Params<SignPayloadParams>,
) -> WorkerResult<Json<SignPayloadResponse>> {
    // Decode hex payload to bytes
    let payload_bytes = hex::decode(&params.payload)
        .map_err(|e| balius_sdk::Error::Internal(format!("Invalid hex payload: {}", e)))?;

    // Sign the payload using the WIT sign interface
    let signature = sign::sign_payload(&params.key_name, &payload_bytes)
        .map_err(|e| balius_sdk::Error::Internal(format!("Sign error: {:?}", e)))?;

    // Return signature with public key from config
    Ok(Json(SignPayloadResponse {
        signature: hex::encode(&signature),
        public_key: config.payment_key_public.clone(),
    }))
}

#[derive(Serialize, Deserialize)]
struct TransferParams {
    sender: String,
    receiver: String,
    quantity: String,
}
#[derive(Serialize, Deserialize)]
struct TransferResponse {
    cbor_hex: String,
}

fn use_protocol(
    _config: Config<BlockfrostConfig>,
    params: Params<TransferParams>,
) -> WorkerResult<Json<TransferResponse>> {
    let protocol_url = url::Url::parse("http://127.0.0.1:8080/transfer").unwrap();

    let body = Some(serde_json::to_vec(&params.0)?);

    let mut request = HttpRequest::post(protocol_url).header("Content-Type", "application/json");
    request.body = body;

    let response = request
        .send()
        .map_err(|e| balius_sdk::Error::Internal(format!("Protocol request error: {:?}", e)))?;

    let parsed: TransferResponse = response.json().map_err(|e| {
        balius_sdk::Error::Internal(format!("Protocol response parse error: {:?}", e))
    })?;

    Ok(Json(parsed))
}

#[balius_sdk::main]
fn main() -> Worker {
    balius_sdk::logging::init();

    worker::logging::log(
        worker::logging::Level::Info,
        "init",
        "Worker initialized - monitoring all transactions with manual filtering",
    );

    Worker::new()
        .with_signer("payment-key", "ed25519") // Register signing key (loaded via baliusd config)
        .with_request_handler("get-latest-block", FnHandler::from(get_latest_block))
        .with_request_handler("sign-payload", FnHandler::from(sign_payload))
        .with_request_handler("use-protocol", FnHandler::from(use_protocol))
    // .with_tx_handler(
    //     UtxoPattern {
    //         address: None,  // Monitor ALL transactions, filter manually in handler
    //         token: None,
    //     },
    //     FnHandler::from(handle_transaction_event),
    // )
}
