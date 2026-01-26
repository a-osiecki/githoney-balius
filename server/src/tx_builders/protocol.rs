pub use githoney::{
    AddParams, AssignParams, Client, ClientOptions, CloseAssignedParams, CloseAssignedSponsoredParams,
    CloseUnassignedParams, CloseUnassignedSponsoredParams, CreateWithLovelaceParams, DeployParams,
    MintBadgeParams,MergeParams, ClaimParams, UpdateBadgeParams
};
use once_cell::sync::Lazy;
use std::collections::HashMap;

/////////////////////////////////////
/////// TRP Client Setup ////////////
/////////////////////////////////////
fn build_client() -> Client {
    let trp_endpoint = std::env::var("TRP_ENDPOINT").unwrap();
    let dmtr_api_key: String = std::env::var("DMTR_API_KEY").unwrap();
    let headers: &[(&str, &str)] = &[("dmtr-api-key", dmtr_api_key.as_str())];
    // Build the TRP client with custom endpoint and headers
    let mut headers_hm: HashMap<String, String> = HashMap::new();
    for (key, value) in headers {
        headers_hm.insert(key.to_string(), value.to_string());
    }
    let client_options = ClientOptions {
        endpoint: trp_endpoint,
        headers: Some(headers_hm),
    };
    Client::new(client_options)
}

pub static PROTOCOL: Lazy<Client> = Lazy::new(|| build_client());
