use axum::{http::StatusCode, Json};
use serde::Serialize;
use tx3_sdk::trp::{Error, TxEnvelope};

use crate::tx_builders::eval::evaluate_tx;

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub type TxHandlerResult = Result<Json<TxEnvelope>, (StatusCode, Json<ErrorResponse>)>;

/// Handle the result of a tx building operation, evaluate it via Ogmios, and return the final tx or an error.
pub async fn handle_tx_result(tx_result: Result<TxEnvelope, Error>) -> TxHandlerResult {
    match tx_result {
        Ok(tx) => {
            log::info!("Generated tx: hash={}", tx.hash);
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
