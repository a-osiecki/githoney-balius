# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository Purpose

This is a **test repository for Balius SDK functionality**, specifically focused on creating and testing submit and sign endpoints for Cardano blockchain interactions. It serves as a development environment for experimenting with Balius runtime modules before production deployment.

## Project Structure

This project follows the Balius headless dApp architecture:

```
test/
├── src/lib.rs           # WASM worker implementation (handlers)
├── Cargo.toml           # Worker dependencies (balius-sdk, serde, url)
├── config.json          # Runtime configuration (API keys, settings)
├── .env                 # Environment variables (BLOCKFROST_PROJECT_ID)
├── BALIUS_GUIDE.md      # Comprehensive Balius architecture documentation
└── test-c.wasm          # Compiled WASM component (generated)
```

**Key Architecture Concept**: This worker compiles to WebAssembly and runs inside the Balius runtime. The runtime provides module implementations (HTTP, Submit, Sign, KV, etc.) that the worker imports via WIT interfaces.

## Development Commands

### Build the Worker
```bash
cargo balius build
```
Compiles the Rust worker to WASM and creates `test-c.wasm` component.

### Run Test Server
```bash
cargo balius test --port 3001 --config ./config.json \
  --utxo-url "cardano-preview.utxorpc-m1.demeter.run" \
  --utxo-api-key "utxorpc120sj2h7w5h5kjy0cey9"
```
Starts local JSON-RPC server on port 3001 with:
- Worker loaded from `test-c.wasm`
- Configuration from `config.json`
- Connected to Cardano preview network via UTxORPC

### Test Endpoints
```bash
# Test the get-latest-block handler
curl -X POST http://localhost:3001/test \
  -H "Content-Type: application/json" \
  -d '{"id": "1", "method": "get-latest-block", "params": {}}'

# Test future submit-tx handler (once implemented)
curl -X POST http://localhost:3001/test \
  -H "Content-Type: application/json" \
  -d '{"id": "1", "method": "submit-tx", "params": {"tx_cbor": "..."}}'
```

### Clean State
```bash
rm -f baliusd.db test-c.wasm
```
Remove the runtime database and compiled WASM to start fresh.

## Critical Architecture: WIT Modules

**The most important concept**: Your worker imports modules from the WIT interface (`/home/benja/Work/balius/balius/wit/balius.wit`). The Balius runtime MUST provide implementations for all imported modules.

### Module System Flow

1. **Worker declares imports** (in `balius.wit`):
   ```wit
   world worker {
       import http;      // Your worker needs HTTP
       import submit;    // Your worker needs Submit
       import sign;      // Your worker needs Sign
       // ...
   }
   ```

2. **Runtime must provide implementations** (in `/home/benja/Work/balius/balius/balius/src/bin/command/test.rs`):
   ```rust
   let runtime = Runtime::builder(store)
       .with_http(Http::Reqwest(reqwest::Client::new()))
       .with_submit(submit)
       .with_signer(signer)
       // Must match all imports!
       .build()?;
   ```

3. **Worker uses the modules** (in `src/lib.rs`):
   ```rust
   use balius_sdk::http::HttpRequest;
   use balius_sdk::wit::balius::app::submit;
   use balius_sdk::wit::balius::app::sign;
   ```

**If the runtime doesn't provide a module that your worker imports, you'll get**: `Error running project: registering worker test-c.wasm`

## Enabling New Modules (Submit/Sign)

### Current Status
- ✅ HTTP module - Enabled and working
- ✅ KV module - Mock implementation
- ✅ Ledger module - UTxORPC connection
- ❌ Submit module - **NOT YET ENABLED** (your next goal)
- ❌ Sign module - **NOT YET ENABLED** (your next goal)

### To Enable Submit Module

**1. Modify the runtime** (`/home/benja/Work/balius/balius/balius/src/bin/command/test.rs`):

Find the runtime builder section (~line 110) and add:
```rust
// Add submit configuration
let submit_config = balius_runtime::submit::Config {
    endpoint_url: "https://submit.demeter.run".to_string(),
    api_key: Some("your-api-key".to_string()),
};
let submit = balius_runtime::submit::Submit::new(&submit_config)
    .await
    .into_diagnostic()
    .context("setting up submit")?;

let runtime = Runtime::builder(store)
    .with_ledger(ledger.into())
    .with_kv(balius_runtime::kv::Kv::Mock)
    .with_http(balius_runtime::http::Http::Reqwest(http_client))
    .with_submit(submit)  // ADD THIS LINE
    .build()
    .into_diagnostic()
    .context("setting up runtime")?;
```

**2. Rebuild Balius CLI**:
```bash
cd /home/benja/Work/balius/balius
cargo install --path ./balius
cd /home/benja/Work/balius/test
```

**3. Add handler to worker** (`src/lib.rs`):
```rust
use balius_sdk::wit::balius::app::submit;

fn submit_transaction(
    _config: Config<BlockfrostConfig>,
    params: Params<SubmitTxParams>
) -> WorkerResult<Json<SubmitResponse>> {
    let tx_bytes = hex::decode(&params.tx_cbor)
        .map_err(|e| balius_sdk::Error::Internal(format!("Invalid hex: {}", e)))?;

    submit::submit_tx(&tx_bytes)?;

    Ok(Json(SubmitResponse { success: true }))
}

#[balius_sdk::main]
fn main() -> Worker {
    Worker::new()
        .with_request_handler("submit-tx", FnHandler::from(submit_transaction))
}
```

**4. Rebuild worker and test**:
```bash
cargo balius build
cargo balius test --port 3001 --config ./config.json \
  --utxo-url "cardano-preview.utxorpc-m1.demeter.run" \
  --utxo-api-key "utxorpc120sj2h7w5h5kjy0cey9"
```

### To Enable Sign Module

**1. Modify the runtime** (same file as above):
```rust
let runtime = Runtime::builder(store)
    .with_ledger(ledger.into())
    .with_kv(balius_runtime::kv::Kv::Mock)
    .with_http(balius_runtime::http::Http::Reqwest(http_client))
    .with_submit(submit)
    .with_signer(balius_runtime::sign::Signer::InMemory)  // ADD THIS LINE
    .build()?;
```

**2. Use in worker**:
```rust
use balius_sdk::wit::balius::app::sign;

#[balius_sdk::main]
fn main() -> Worker {
    Worker::new()
        .with_signer("payment-key", "ed25519")  // Register key
        .with_request_handler("sign-data", FnHandler::from(sign_handler))
}

fn sign_handler(...) -> WorkerResult<...> {
    let signature = sign::sign_payload("payment-key", &payload)?;
    // ...
}
```

## Worker Development Pattern

All handlers follow this pattern:

```rust
use balius_sdk::{Config, FnHandler, Params, Json, Worker, WorkerResult};

#[derive(Serialize, Deserialize, Clone)]
struct MyConfig {
    // Fields from config.json
}

#[derive(Serialize, Deserialize)]
struct MyParams {
    // Request parameters
}

#[derive(Serialize, Deserialize)]
struct MyResponse {
    // Response data
}

fn my_handler(
    config: Config<MyConfig>,
    params: Params<MyParams>
) -> WorkerResult<Json<MyResponse>> {
    // Handler logic
    Ok(Json(MyResponse { /* ... */ }))
}

#[balius_sdk::main]
fn main() -> Worker {
    balius_sdk::logging::init();
    Worker::new()
        .with_request_handler("my-handler", FnHandler::from(my_handler))
}
```

## Configuration Files

### config.json
Runtime configuration passed to worker on initialization:
```json
{
  "project_id": "preprod6Zlqd54IUdWzdIG85rBE4BCnK0M78nsI"
}
```

### .env
Environment variables (read via `std::env::var`):
```
BLOCKFROST_PROJECT_ID=preprod6Zlqd54IUdWzdIG85rBE4BCnK0M78nsI
```

## Modified Balius Runtime

This project has **locally modified** the Balius CLI to enable HTTP support. The modifications are in:

**File**: `/home/benja/Work/balius/balius/balius/src/bin/command/test.rs` (line 109-113)
**Change**: Added `.with_http(balius_runtime::http::Http::Reqwest(http_client))`

**File**: `/home/benja/Work/balius/balius/balius/Cargo.toml` (line 26)
**Change**: Added `reqwest = "0.12"`

To apply similar changes for Submit/Sign, follow the same pattern: modify the test command and rebuild the Balius CLI.

## Troubleshooting

### Worker fails to register
**Error**: `Error running project: registering worker test-c.wasm`

**Cause**: Worker imports a WIT module that runtime doesn't provide.

**Solution**:
1. Check which modules your `src/lib.rs` uses
2. Verify runtime has corresponding `.with_MODULE()` in test command
3. Rebuild balius: `cd /home/benja/Work/balius/balius && cargo install --path ./balius`

### Port already in use
```bash
lsof -i :3001
kill -9 <PID>
rm -f baliusd.db
```

### HTTP requests fail with JSON parsing errors
Check that you're using `serde_json::from_slice(&response.body)` instead of `response.json::<T>()` to avoid Result type confusion.

## Key Files Reference

- **WIT Interface**: `/home/benja/Work/balius/balius/wit/balius.wit` - Defines all available modules
- **Runtime Source**: `/home/benja/Work/balius/balius/balius-runtime/src/lib.rs` - Runtime builder
- **Test Command**: `/home/benja/Work/balius/balius/balius/src/bin/command/test.rs` - Where modules are enabled
- **Module Implementations**: `/home/benja/Work/balius/balius/balius-runtime/src/{http,submit,sign}/` - Module code

## Current Implementation

The worker currently has one working endpoint:

**get-latest-block**: Calls Blockfrost API to fetch latest Cardano preprod block
- Uses HTTP module to make external API requests
- Demonstrates how to integrate external APIs in Balius workers
- Returns JSON response with block data (hash, height, slot, etc.)

## Next Steps

To complete the repository goals:

1. **Enable Submit module** in runtime (modify test command)
2. **Add submit-tx endpoint** to worker (src/lib.rs)
3. **Enable Sign module** in runtime
4. **Add sign-data endpoint** to worker
5. Test both endpoints with real transaction data

See BALIUS_GUIDE.md for comprehensive module documentation and examples.
