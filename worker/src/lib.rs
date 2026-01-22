mod chainsync;
mod offchain_handlers;
mod signature;
mod types;
mod utils;

use balius_sdk::wit::balius::app as worker;
use balius_sdk::{FnHandler, Worker};
// use balius_sdk::wit::balius::app::submit;
// use balius_sdk::wit::balius::app::driver::UtxoPattern;

use crate::chainsync::get_latest_block;
use crate::offchain_handlers::{
    add_funds, assign_contributor, close_assigned, close_assigned_sponsored, close_unassigned,
    close_unassigned_sponsored, create_bounty, publish_settings, merge, claim, deploy_badge
};
use crate::signature::sign_tx;

#[balius_sdk::main]
fn main() -> Worker {
    balius_sdk::logging::init();

    worker::logging::log(
        worker::logging::Level::Info,
        "init",
        "Worker initialized - monitoring all transactions with manual filtering",
    );

    Worker::new()
        .with_signer("payment-key", "ed25519")
        .with_request_handler("sign-tx", FnHandler::from(sign_tx))
        .with_request_handler("get-latest-block", FnHandler::from(get_latest_block))
        .with_request_handler("publish-settings", FnHandler::from(publish_settings))
        .with_request_handler("create-bounty", FnHandler::from(create_bounty))
        .with_request_handler("add-funds", FnHandler::from(add_funds))
        .with_request_handler("assign", FnHandler::from(assign_contributor))
        .with_request_handler("close-unassigned", FnHandler::from(close_unassigned))
        .with_request_handler(
            "close-unassigned-sponsored",
            FnHandler::from(close_unassigned_sponsored),
        )
        .with_request_handler("close-assigned", FnHandler::from(close_assigned))
        .with_request_handler(
            "close-assigned-sponsored",
            FnHandler::from(close_assigned_sponsored)
        )
        .with_request_handler("merge", FnHandler::from(merge))
        .with_request_handler("claim", FnHandler::from(claim))
        .with_request_handler("deploy-badges", FnHandler::from(deploy_badge))
    // .with_tx_handler(
    //     UtxoPattern {
    //         address: None,  // Monitor ALL transactions, filter manually in handler
    //         token: None,
    //     },
    //     FnHandler::from(handle_transaction_event),
    // )
}
