mod chainsync;
mod offchain_handlers;
mod signature;
mod types;
mod utils;

use balius_sdk::wit::Config;
use balius_sdk::wit::balius::app as worker;
use balius_sdk::{Error, FnHandler, Worker};
// use balius_sdk::wit::balius::app::submit;
use balius_sdk::wit::balius::app::driver::UtxoPattern;

use crate::chainsync::handle_transaction_event;
use crate::offchain_handlers::{
    add_funds, assign_contributor, claim, close_assigned, close_assigned_sponsored,
    close_unassigned, close_unassigned_sponsored, collect_utxos, create_bounty_with_lovelace, create_bounty_with_token, merge, mint_badge, pay_badges_to,
    publish_settings, update_badge,
};
use crate::signature::{_submit_tx, sign_tx};

#[balius_sdk::main]
fn main(_config: Config<WorkerConfig>) -> Worker {
//     let exact_address = hex::decode(&"addr_test1wr5nj776ygd7d7vwcw3f9fknj4axx0wzy67y3qu8dqtscys7w9q65").map_err(|_| Error::BadParams).unwrap();

    balius_sdk::logging::init();

    worker::logging::log(
        worker::logging::Level::Info,
        "init",
        "Worker initialized - monitoring all transactions with manual filtering",
    );

    Worker::new()
        .with_signer("payment-key", "ed25519")
        .with_request_handler("sign-tx", FnHandler::from(sign_tx))
        .with_request_handler("settings/deploy", FnHandler::from(publish_settings))
        .with_request_handler(
            "bounty/create_with_lovelace",
            FnHandler::from(create_bounty_with_lovelace)
        )
        .with_request_handler(
            "bounty/create_with_token",
            FnHandler::from(create_bounty_with_token)
        )
        .with_request_handler("bounty/add-funds", FnHandler::from(add_funds))
        .with_request_handler("bounty/assign", FnHandler::from(assign_contributor))
        .with_request_handler("bounty/close-unassigned", FnHandler::from(close_unassigned))
        .with_request_handler(
            "bounty/close-unassigned-sponsored",
            FnHandler::from(close_unassigned_sponsored),
        )
        .with_request_handler("bounty/close-assigned", FnHandler::from(close_assigned))
        .with_request_handler(
            "bounty/close-assigned-sponsored",
            FnHandler::from(close_assigned_sponsored),
        )
        .with_request_handler("bounty/merge", FnHandler::from(merge))
        .with_request_handler("bounty/claim", FnHandler::from(claim))
        .with_request_handler("badge/collect", FnHandler::from(collect_utxos))
        .with_request_handler("badge/mint", FnHandler::from(mint_badge))
        .with_request_handler("badge/update", FnHandler::from(update_badge))
        .with_request_handler("badge/pay", FnHandler::from(pay_badges_to))
        .with_request_handler("submit", FnHandler::from(_submit_tx))
        .with_tx_handler(
            UtxoPattern {
                address: None, //Monitor ALL transactions, filter manually in handler
                token: None,
            },
            FnHandler::from(handle_transaction_event),
        )
}
