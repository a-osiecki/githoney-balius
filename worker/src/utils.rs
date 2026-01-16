use balius_sdk::{http::HttpRequest, Json, WorkerResult};
use url::Url;

use crate::types::TxEnvelope;

pub fn do_tx_building_request(url: Url, body: Option<Vec<u8>>) -> WorkerResult<Json<TxEnvelope>> {
    let mut request = HttpRequest::post(url).header("Content-Type", "application/json");
    request.body = body;

    let response = request
        .send()
        .map_err(|e| balius_sdk::Error::Internal(format!("Protocol request error: {:?}", e)))?;

    let parsed: TxEnvelope = response.json().map_err(|e| {
        balius_sdk::Error::Internal(format!("Protocol response parse error: {:?}", e))
    })?;

    Ok(Json(parsed))
}
