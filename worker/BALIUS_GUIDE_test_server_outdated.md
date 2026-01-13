# Balius SDK Guide

## Table of Contents
1. [Architecture Overview](#architecture-overview)
2. [WIT Interface System](#wit-interface-system)
3. [Runtime Modules](#runtime-modules)
4. [Enabling Optional Features](#enabling-optional-features)
5. [Available Modules](#available-modules)
6. [Example: Adding Submit Feature](#example-adding-submit-feature)

---

## Architecture Overview

Balius uses a modular architecture with three main components:

```
┌─────────────────────────────────────────────┐
│           Your WASM Worker                   │
│  (Built with balius-sdk in Rust)            │
│                                              │
│  - Handlers for events (UTXOs, Txs, etc.)   │
│  - Request handlers (JSON-RPC endpoints)    │
│  - Imports features from WIT interface      │
└──────────────────┬──────────────────────────┘
                   │ WIT Interface
                   │ (balius.wit)
┌──────────────────▼──────────────────────────┐
│         Balius Runtime                       │
│  (Provides implementations of WIT modules)   │
│                                              │
│  - HTTP, KV, Ledger, Logging, etc.          │
│  - Event routing and dispatch               │
│  - Worker lifecycle management              │
└──────────────────┬──────────────────────────┘
                   │
┌──────────────────▼──────────────────────────┐
│         Host Environment                     │
│  - Blockchain connection (UTxORPC)          │
│  - Database (reDB)                          │
│  - External APIs                            │
└─────────────────────────────────────────────┘
```

---

## WIT Interface System

**WIT (WebAssembly Interface Types)** defines the contract between your worker and the runtime.

### Location
The WIT file is located at: `/home/benja/Work/balius/balius/wit/balius.wit`

### Key Concept: `world worker`

At the bottom of `balius.wit`, you'll find:

```wit
world worker {
    import kv;          // Key-value storage
    import broadcast;   // Message broadcasting
    import logging;     // Logging functionality
    import sign;        // Transaction signing
    import submit;      // Transaction submission
    import http;        // HTTP requests
    import ledger;      // Blockchain queries

    export init: func(config: config);
    export handle: func(channel: u32, evt: event) -> result<response, handle-error>;
}
```

**Important**: When your worker declares `import http` (or any module), it **expects** the runtime to provide that implementation.

---

## Runtime Modules

The runtime must explicitly provide implementations for the modules your worker imports.

### Default vs Optional Modules

When you run `cargo balius test`, the runtime is built with:

```rust
let runtime = Runtime::builder(store)
    .with_ledger(ledger.into())       // ✅ Included by default
    .with_kv(Kv::Mock)                // ✅ Included by default
    .build()?;
```

**Missing modules**: HTTP, Submit, Sign, Broadcast, Logging (beyond basic)

If your worker uses a module that wasn't added to the runtime, you'll get an error during worker registration.

---

## Enabling Optional Features

To enable a feature like HTTP or Submit, you need to:

### Step 1: Modify the Test Command

Edit: `/home/benja/Work/balius/balius/balius/src/bin/command/test.rs`

Find the runtime builder section (around line 109) and add the module:

```rust
let runtime = Runtime::builder(store)
    .with_ledger(ledger.into())
    .with_kv(balius_runtime::kv::Kv::Mock)
    .with_http(balius_runtime::http::Http::Reqwest(reqwest::Client::new()))
    // Add other modules here...
    .build()?;
```

### Step 2: Add Required Dependencies

Some modules require external crates. Edit `/home/benja/Work/balius/balius/balius/Cargo.toml`:

```toml
[dependencies]
# ... existing dependencies ...
reqwest = "0.12"  # Required for HTTP
```

### Step 3: Rebuild and Reinstall Balius

```bash
cd /home/benja/Work/balius/balius
cargo install --path ./balius
```

---

## Available Modules

### 1. HTTP Module

**Purpose**: Make HTTP requests to external APIs

**Runtime Implementation**:
```rust
use balius_runtime::http::Http;

// Option 1: Mock (returns empty responses)
.with_http(Http::Mock)

// Option 2: Real HTTP with reqwest
let http_client = reqwest::Client::new();
.with_http(Http::Reqwest(http_client))
```

**Worker Usage**:
```rust
use balius_sdk::http::HttpRequest;
use url::Url;

let url = Url::parse("https://api.example.com/data")?;
let response = HttpRequest::get(url)
    .header("Authorization", "Bearer token")
    .send()?;

let data = response.json::<MyStruct>()?;
```

**Dependencies**: Add `reqwest = "0.12"` to balius Cargo.toml

---

### 2. Submit Module

**Purpose**: Submit transactions to the blockchain

**Runtime Implementation**:
```rust
use balius_runtime::submit::Submit;

// Configure transaction submission endpoint
let submit_config = balius_runtime::submit::Config {
    endpoint_url: "https://submit.demeter.run".to_string(),
    api_key: Some("your-api-key".to_string()),
};

let submit = Submit::new(&submit_config).await?;
.with_submit(submit)
```

**Worker Usage**:
```rust
use balius_sdk::wit::balius::app::submit;

// Submit a transaction (CBOR bytes)
let tx_cbor: Vec<u8> = build_transaction();
submit::submit_tx(&tx_cbor)?;
```

**WIT Interface** (in balius.wit):
```wit
interface submit {
    type cbor = list<u8>;
    type submit-error = u32;

    submit-tx: func(tx: cbor) -> result<_, submit-error>;
}
```

---

### 3. Sign Module

**Purpose**: Sign transactions and payloads

**Runtime Implementation**:
```rust
use balius_runtime::sign::Signer;

// In-memory signer (for testing)
.with_signer(Signer::InMemory)
```

**Worker Usage**:
```rust
use balius_sdk::wit::balius::app::sign;

// Register a signing key in your main function
#[balius_sdk::main]
fn main() -> Worker {
    Worker::new()
        .with_signer("my-key", "ed25519")
        .with_request_handler("sign-data", FnHandler::from(handle_signing))
}

// Sign a payload
fn handle_signing(config: Config<MyConfig>, params: Params<SignParams>)
    -> WorkerResult<Json<SignResponse>>
{
    let payload: Vec<u8> = params.data.clone();
    let signature = sign::sign_payload("my-key", &payload)?;

    Ok(Json(SignResponse { signature }))
}
```

---

### 4. KV (Key-Value Store) Module

**Purpose**: Persistent data storage

**Runtime Implementation**:
```rust
use balius_runtime::kv::Kv;

// Option 1: Mock (in-memory, not persisted)
.with_kv(Kv::Mock)

// Option 2: ReDB (persistent database)
.with_kv(Kv::ReDb(db_instance))
```

**Worker Usage**:
```rust
use balius_sdk::wit::balius::app::kv;

// Store a value
let key = "user:123";
let value = b"some data";
kv::set_value(key, value)?;

// Retrieve a value
let value = kv::get_value(key)?;

// List keys with prefix
let keys = kv::list_values("user:")?;
```

---

### 5. Logging Module

**Purpose**: Structured logging

**Runtime Implementation**:
```rust
use balius_runtime::logging::Logger;

// Option 1: Silent (no logs)
.with_logger(Logger::Silent)

// Option 2: File logging
.with_logger(Logger::File(file_path))
```

**Worker Usage**:
```rust
use balius_sdk::wit::balius::app::logging;

#[balius_sdk::main]
fn main() -> Worker {
    // Initialize logging first
    balius_sdk::logging::init();

    Worker::new()
        .with_request_handler("my-handler", FnHandler::from(my_handler))
}

fn my_handler(config: Config<MyConfig>, params: Params<MyParams>)
    -> WorkerResult<Json<MyResponse>>
{
    logging::log(
        logging::Level::Info,
        "my-handler",
        "Processing request"
    );

    // Your logic here...
}
```

---

### 6. Ledger Module

**Purpose**: Query blockchain data (UTXOs, parameters)

**Runtime Implementation**:
```rust
use balius_runtime::ledgers;

let ledger_config = ledgers::u5c::Config {
    endpoint_url: "cardano-preview.utxorpc-m1.demeter.run".to_string(),
    headers: Some(HashMap::from([(
        "dmtr-api-key".to_string(),
        "your-api-key".to_string(),
    )])),
};

let ledger = ledgers::u5c::Ledger::new(&ledger_config).await?;
.with_ledger(ledger.into())
```

**Worker Usage**:
```rust
use balius_sdk::wit::balius::app::ledger;

// Read specific UTXOs
let txo_refs = vec![ledger::TxoRef {
    tx_hash: tx_hash_bytes,
    tx_index: 0,
}];
let utxos = ledger::read_utxos(&txo_refs)?;

// Search UTXOs by pattern
let pattern = ledger::UtxoPattern {
    address: Some(ledger::AddressPattern {
        exact_address: address_bytes,
    }),
    asset: None,
};
let page = ledger::search_utxos(&pattern, None, 10)?;

// Read protocol parameters
let params_json = ledger::read_params()?;
```

---

## Example: Adding Submit Feature

Let's add transaction submission capability to your worker.

### 1. Update the Runtime

Edit `/home/benja/Work/balius/balius/balius/src/bin/command/test.rs`:

```rust
async fn run_project_with_config(
    project_name: &str,
    config_path: Option<PathBuf>,
    port: u16,
    utxo_url: String,
    utxo_api_key: String,
) -> miette::Result<()> {
    setup_tracing()?;

    info!("Running Balius project on daemon...");
    let store: Store = Store::open("baliusd.db", None)
        .into_diagnostic()
        .context("opening store")?;

    // Ledger setup
    let ledger_config = ledgers::u5c::Config {
        endpoint_url: utxo_url.clone(),
        headers: Some(HashMap::from([(
            "dmtr-api-key".to_string(),
            utxo_api_key.to_string(),
        )])),
    };
    let ledger = ledgers::u5c::Ledger::new(&ledger_config)
        .await
        .into_diagnostic()
        .context("setting up ledger")?;

    // HTTP setup
    let http_client = reqwest::Client::new();

    // Submit setup (NEW)
    let submit_config = balius_runtime::submit::Config {
        endpoint_url: "https://submit.demeter.run".to_string(),
        api_key: Some("your-submit-api-key".to_string()),
    };
    let submit = balius_runtime::submit::Submit::new(&submit_config)
        .await
        .into_diagnostic()
        .context("setting up submit")?;

    // Build runtime with all modules
    let runtime = Runtime::builder(store)
        .with_ledger(ledger.into())
        .with_kv(balius_runtime::kv::Kv::Mock)
        .with_http(balius_runtime::http::Http::Reqwest(http_client))
        .with_submit(submit)  // Add submit module
        .build()
        .into_diagnostic()
        .context("setting up runtime")?;

    // ... rest of the function
}
```

### 2. Check Dependencies

Verify `/home/benja/Work/balius/balius/balius/Cargo.toml` has:

```toml
[dependencies]
balius-runtime = { version = "0.5.2", path = "../balius-runtime" }
reqwest = "0.12"
# Other dependencies...
```

### 3. Rebuild Balius

```bash
cd /home/benja/Work/balius/balius
cargo install --path ./balius
```

### 4. Use Submit in Your Worker

Edit `/home/benja/Work/balius/test/src/lib.rs`:

```rust
use balius_sdk::{Config, FnHandler, Params, Json, Worker, WorkerResult};
use balius_sdk::wit::balius::app::submit;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct SubmitTxParams {
    tx_cbor: String, // Hex-encoded transaction
}

#[derive(Serialize, Deserialize)]
struct SubmitResponse {
    success: bool,
}

fn submit_transaction(
    _config: Config<BlockfrostConfig>,
    params: Params<SubmitTxParams>
) -> WorkerResult<Json<SubmitResponse>> {
    // Decode hex transaction
    let tx_bytes = hex::decode(&params.tx_cbor)
        .map_err(|e| balius_sdk::Error::Internal(format!("Invalid hex: {}", e)))?;

    // Submit to blockchain
    submit::submit_tx(&tx_bytes)?;

    Ok(Json(SubmitResponse { success: true }))
}

#[balius_sdk::main]
fn main() -> Worker {
    balius_sdk::logging::init();
    Worker::new()
        .with_request_handler("get-latest-block", FnHandler::from(get_latest_block))
        .with_request_handler("submit-tx", FnHandler::from(submit_transaction))
}
```

### 5. Test It

```bash
# Start the server
cargo balius test --port 3001 --config ./config.json \
  --utxo-url "cardano-preview.utxorpc-m1.demeter.run" \
  --utxo-api-key "your-api-key"

# Submit a transaction
curl -X POST http://localhost:3001/test \
  -H "Content-Type: application/json" \
  -d '{
    "id": "1",
    "method": "submit-tx",
    "params": {
      "tx_cbor": "84a300818258..."
    }
  }'
```

---

## Module Dependency Matrix

| Module    | Required Runtime Dependency | Required Worker Dependency |
|-----------|----------------------------|----------------------------|
| HTTP      | `reqwest`                  | `url`                      |
| Submit    | None (built-in)            | None                       |
| Sign      | None (built-in)            | None                       |
| KV        | None (built-in)            | None                       |
| Ledger    | None (built-in)            | None                       |
| Logging   | None (built-in)            | None                       |

---

## Troubleshooting

### Worker Registration Fails

**Error**: `Error running project: registering worker test-c.wasm`

**Cause**: Your worker imports a WIT module that the runtime doesn't provide.

**Solution**:
1. Check which modules your code uses
2. Add the corresponding `.with_MODULE()` to the runtime builder
3. Rebuild and reinstall balius

### HTTP Module Not Working

**Symptoms**: Worker registers but HTTP requests fail silently

**Checklist**:
- ✅ Added `.with_http(Http::Reqwest(...))` to runtime
- ✅ Added `reqwest = "0.12"` to balius Cargo.toml
- ✅ Rebuilt balius: `cargo install --path ./balius`
- ✅ Restarted the test server

---

## Current Project Status

Your current setup has:

✅ **HTTP Module** - Enabled and working
✅ **KV Module** - Mock implementation
✅ **Ledger Module** - Connected to UTxORPC
✅ **Logging Module** - Basic logging initialized

❌ **Submit Module** - Not yet enabled
❌ **Sign Module** - Not yet enabled

---

## Next Steps

To add Submit functionality:
1. Follow the [Example: Adding Submit Feature](#example-adding-submit-feature) section above
2. Configure the submit endpoint URL and API key
3. Rebuild balius
4. Add a submit handler to your worker
5. Test transaction submission

---

## References

- **WIT Specification**: `/home/benja/Work/balius/balius/wit/balius.wit`
- **Runtime Source**: `/home/benja/Work/balius/balius/balius-runtime/src/lib.rs`
- **Test Command**: `/home/benja/Work/balius/balius/balius/src/bin/command/test.rs`
- **SDK Documentation**: https://docs.rs/balius-sdk
- **Balius Repository**: https://github.com/txpipe/balius
