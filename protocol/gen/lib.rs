// This file is auto-generated.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

pub use tx3_sdk::trp::ClientOptions;
use tx3_sdk::core::{TirEnvelope, BytesEncoding};
use tx3_sdk::trp::{ResolveParams, TxEnvelope, SubmitParams, SubmitResponse};

pub const DEFAULT_TRP_ENDPOINT: &str = "http://localhost:8164";

pub const DEFAULT_HEADERS: &[(&str, &str)] = &[
];

pub const TRANSFER_IR: &str = "ab6466656573a1694576616c506172616d6a457870656374466565736a7265666572656e6365738066696e7075747381a3646e616d6566736f75726365657574786f73a1694576616c506172616da16b457870656374496e7075748266736f75726365a56761646472657373a1694576616c506172616da16b45787065637456616c7565826673656e64657267416464726573736a6d696e5f616d6f756e74a16b4576616c4275696c74496ea16341646482a16641737365747381a366706f6c696379644e6f6e656a61737365745f6e616d65644e6f6e6566616d6f756e74a1694576616c506172616da16b45787065637456616c756582687175616e7469747963496e74a1694576616c506172616d6a4578706563744665657363726566644e6f6e65646d616e79f46a636f6c6c61746572616cf46872656465656d6572644e6f6e65676f75747075747382a46761646472657373a1694576616c506172616da16b45787065637456616c756582687265636569766572674164647265737365646174756d644e6f6e6566616d6f756e74a16641737365747381a366706f6c696379644e6f6e656a61737365745f6e616d65644e6f6e6566616d6f756e74a1694576616c506172616da16b45787065637456616c756582687175616e7469747963496e74686f7074696f6e616cf4a46761646472657373a1694576616c506172616da16b45787065637456616c7565826673656e646572674164647265737365646174756d644e6f6e6566616d6f756e74a16b4576616c4275696c74496ea16353756282a16b4576616c4275696c74496ea16353756282a16a4576616c436f65726365a16a496e746f417373657473a1694576616c506172616da16b457870656374496e7075748266736f75726365a56761646472657373a1694576616c506172616da16b45787065637456616c7565826673656e64657267416464726573736a6d696e5f616d6f756e74a16b4576616c4275696c74496ea16341646482a16641737365747381a366706f6c696379644e6f6e656a61737365745f6e616d65644e6f6e6566616d6f756e74a1694576616c506172616da16b45787065637456616c756582687175616e7469747963496e74a1694576616c506172616d6a4578706563744665657363726566644e6f6e65646d616e79f46a636f6c6c61746572616cf4a16641737365747381a366706f6c696379644e6f6e656a61737365745f6e616d65644e6f6e6566616d6f756e74a1694576616c506172616da16b45787065637456616c756582687175616e7469747963496e74a1694576616c506172616d6a45787065637446656573686f7074696f6e616cf46876616c6964697479f6656d696e747380656275726e7380656164686f63806a636f6c6c61746572616c80677369676e657273f6686d6574616461746180";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferParams {
    pub quantity: String,
    pub receiver: String,
    pub sender: String,
}
impl TransferParams {
    fn to_map(&self) -> serde_json::Map<String, serde_json::Value> {
        let mut map = serde_json::Map::new();

        map.insert("quantity".to_string(), serde_json::json!(&self.quantity));
        map.insert("receiver".to_string(), serde_json::json!(&self.receiver));
        map.insert("sender".to_string(), serde_json::json!(&self.sender));

        map.into()
    }
}

pub struct Client {
    client: tx3_sdk::trp::Client,
}

impl Client {
    pub fn new(options: ClientOptions) -> Self {
        Self {
            client: tx3_sdk::trp::Client::new(options),
        }
    }

    pub fn with_default_options() -> Self {
        let mut headers = HashMap::new();
        for (key, value) in DEFAULT_HEADERS {
            headers.insert(key.to_string(), value.to_string());
        }

        Self::new(ClientOptions {
            endpoint: DEFAULT_TRP_ENDPOINT.to_string(),
            headers: Some(headers),
        })
    }

    pub async fn transfer_tx(&self, args: TransferParams) -> Result<TxEnvelope, tx3_sdk::trp::Error> {
        let tir_info = TirEnvelope {
            content: TRANSFER_IR.to_string(),
            encoding: BytesEncoding::Hex,
            version: "v1beta0".to_string(),
        };

        self.client.resolve(ResolveParams {
            tir: tir_info,
            args: args.to_map(),
        }).await
    }

    pub async fn submit(&self, params: SubmitParams) -> Result<SubmitResponse, tx3_sdk::trp::Error> {
        self.client.submit(params).await
    }
}

// Create a default client instance
pub static PROTOCOL: once_cell::sync::Lazy<Client> = once_cell::sync::Lazy::new(|| Client::with_default_options());
