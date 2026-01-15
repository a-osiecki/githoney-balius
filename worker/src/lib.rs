mod chainsync;
mod offchain_handlers;
mod signature;
mod types;

use balius_sdk::{FnHandler, Worker};
use balius_sdk::wit::balius::app as worker;
// use balius_sdk::wit::balius::app::submit;
// use balius_sdk::wit::balius::app::driver::UtxoPattern;

use crate::chainsync::get_latest_block;
use crate::offchain_handlers::{create_bounty, publish_settings};
use crate::signature::sign_payload;

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
        .with_request_handler("create-bounty", FnHandler::from(create_bounty))
        .with_request_handler("publish-settings", FnHandler::from(publish_settings))
    // .with_tx_handler(
    //     UtxoPattern {
    //         address: None,  // Monitor ALL transactions, filter manually in handler
    //         token: None,
    //     },
    //     FnHandler::from(handle_transaction_event),
    // )
}
