use balius_sdk::{http::HttpRequest, Error, Json, WorkerResult};
use url::Url;

use crate::types::{ErrorResponse, TxEnvelope};

pub fn do_tx_building_request(url: Url, body: Option<Vec<u8>>) -> WorkerResult<Json<TxEnvelope>> {
    println!("{:?}", body);
    let mut request = HttpRequest::post(url).header("Content-Type", "application/json");
    request.body = body;

    let response = request
        .send()
        .map_err(|e| balius_sdk::Error::Internal(format!("Protocol request error: {:?}", e)))?;

    if !response.is_ok() {
        let err_msg = parse_error_body(&response);
        return Err(Error::Internal(format!(
            "tx-builder error (status {}): {}",
            response.status, err_msg
        )));
    }

    let parsed = parse_tx_envelope(&response).map_err(Error::Internal)?;

    Ok(Json(parsed))
}

fn parse_tx_envelope(response: &balius_sdk::http::HttpResponse) -> Result<TxEnvelope, String> {
    let body = &response.body;

    if let Ok(tx) = serde_json::from_slice::<TxEnvelope>(body) {
        return Ok(tx);
    }

    if let Ok(result) = serde_json::from_slice::<Result<TxEnvelope, String>>(body) {
        return result.map_err(|e| format!("tx-builder error: {e}"));
    }

    Err(String::from_utf8_lossy(body).to_string())
}

fn parse_error_body(response: &balius_sdk::http::HttpResponse) -> String {
    let body = &response.body;

    if let Ok(err) = serde_json::from_slice::<ErrorResponse>(body) {
        return err.error;
    }

    if let Ok(result) = serde_json::from_slice::<Result<TxEnvelope, String>>(body) {
        if let Err(err) = result {
            return err;
        }
    }

    String::from_utf8_lossy(body).to_string()
}
