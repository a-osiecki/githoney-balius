# Balius Daemon (baliusd) - Correct Usage Guide

## Overview

After investigation, we've clarified the correct way to run Balius workers in production. This document explains the differences between development tools and the production daemon, and how to properly use `baliusd`.

## Key Discovery: Two Different Runtime Environments

### 1. `cargo balius test` - Development/Testing Tool

**Location**: `/home/benja/Work/balius/balius/balius/src/bin/command/test.rs`

**Purpose**: Quick testing and development

**Characteristics**:
- Limited module support by default
- Requires manual modification of source code to enable modules
- Located in the `balius` CLI tool codebase
- NOT intended for production use
- Must rebuild entire `balius` binary to add modules

**Default Modules** (as of current version):
```rust
let runtime = Runtime::builder(store)
    .with_ledger(ledger.into())
    .with_kv(Kv::Mock)
    // HTTP, Submit, Sign NOT enabled by default
    .build()?;
```

**Why It Exists**: Provides a quick way to test workers during development without setting up full daemon configuration.

---

### 2. `baliusd` - Production Daemon

**Location**: `/home/benja/Work/balius/balius/baliusd/`

**Purpose**: Production-ready runtime for Balius workers

**Characteristics**:
- Full module support configured via TOML files
- Can run multiple workers simultaneously
- Proper lifecycle management
- Configuration-driven (no code changes needed)
- Production-grade logging, metrics, and monitoring

**Enabled Modules** (in baliusd):
```rust
let runtime = Runtime::builder(store)
    .with_ledger(ledger.into())
    .with_kv(kv)                    // ✅ Configurable (Mock, Memory, Redb)
    .with_logger(logger)            // ✅ Configurable (Silent, Tracing, File)
    .with_signer(signer)            // ✅ Configurable (InMemory with keys)
    .with_http(http)                // ✅ Enabled (Reqwest client)
    // .with_submit(submit)         // ❌ NOT YET INTEGRATED (exists in runtime, not in baliusd)
    .build()?;
```

---

## Current State Analysis

### What Works in baliusd

Based on the source code at `/home/benja/Work/balius/balius/baliusd/src/main.rs` (lines 95-103):

| Module   | Status | Configuration |
|----------|--------|---------------|
| Ledger   | ✅ Enabled | Required - `[ledger]` section in `baliusd.toml` |
| KV       | ✅ Enabled | Optional - `[kv]` section (defaults to Mock) |
| Logging  | ✅ Enabled | Optional - `[logger]` section (defaults to Silent) |
| Sign     | ✅ Enabled | Optional - `[signing]` section (defaults to empty InMemory) |
| HTTP     | ✅ Enabled | Optional - `[http]` section (defaults to 10s timeout) |
| Submit   | ❌ Not Yet | Module exists in `balius-runtime/src/submit/` but not integrated into baliusd |

### Why Submit Is Missing

The submit module implementation exists at `/home/benja/Work/balius/balius/balius-runtime/src/submit/mod.rs`:

```rust
pub enum Submit {
    Mock,  // Currently only prints tx hex
}

impl wit::Host for Submit {
    async fn submit_tx(&mut self, tx: wit::Cbor) -> Result<(), wit::SubmitError> {
        println!("{}", hex::encode(tx));
        Ok(())
    }
}
```

**But**: It's not wired up in baliusd's configuration system. To add it would require:
1. Adding `SubmitConfig` enum to `/home/benja/Work/balius/balius/baliusd/src/config.rs`
2. Implementing `From<&Config> for balius_runtime::submit::Submit`
3. Adding `.with_submit(submit)` to runtime builder in `main.rs`

**Alternative Approach**: Handle transaction submission outside of Balius using external tools or APIs (Blockfrost, cardano-submit-api, etc.)

---

## Installation

### Install baliusd

```bash
cd /home/benja/Work/balius/balius
cargo install --path ./baliusd
```

This installs the `baliusd` binary to `~/.cargo/bin/baliusd`.

---

## Configuration

### Directory Structure

```
your-project/
├── baliusd.toml          # Daemon configuration
├── config.json           # Worker-specific config
├── your-worker.wasm      # Compiled WASM worker
└── baliusd.db            # Runtime database (created automatically)
```

### Complete baliusd.toml Example

```toml
# JSON-RPC server configuration
[rpc]
listen_address = "0.0.0.0:3001"

# Logging configuration (for daemon itself)
[logging]
max_level = "debug"       # Options: "trace", "debug", "info", "warn", "error"
include_tokio = false     # Include tokio tracing (verbose)

# UTxORPC Ledger configuration
[ledger]
endpoint_url = "https://cardano-preview.utxorpc-m1.demeter.run"
headers."dmtr-api-key" = "your-api-key-here"

# Chain synchronization configuration
[chainsync]
endpoint_url = "https://cardano-preview.utxorpc-m1.demeter.run"
headers."dmtr-api-key" = "your-api-key-here"

# HTTP client configuration
[http]
type = "reqwest"
timeout = 30              # Timeout in seconds (default: 10)

# Key-Value store configuration
[kv]
type = "memory"           # Options: "memory", "redb"

# For persistent storage:
# [kv]
# type = "redb"
# path = "kv.db"
# cache_size = 1024

# Logger configuration (for worker logs via WIT)
[logger]
type = "tracing"          # Options: "silent", "tracing", "file"

# For file logging:
# [logger]
# type = "file"
# folder = "./logs"

# Sign module configuration
[signing]
type = "memory"

# Optional: Pre-configure signing keys
# [[signing.keys]]
# worker = "my-worker"
# name = "payment-key"
# algorithm = "ed25519"
# private_key = "your-hex-encoded-private-key"

# Persistent storage configuration
[store]
path = "baliusd.db"

# Optional: Metrics endpoint
# [metrics]
# listen_address = "0.0.0.0:9090"

# Define workers to load
[[workers]]
name = "test"                    # Worker name (used in API path)
module = "test-c.wasm"           # Path to WASM file
config = "config.json"           # Worker configuration file
# since_slot = 1000000           # Optional: Start from specific slot
# until_slot = 2000000           # Optional: Stop at specific slot

# You can define multiple workers
# [[workers]]
# name = "another-worker"
# module = "another.wasm"
# config = "another-config.json"
```

---

## Running baliusd

### Basic Usage

```bash
# baliusd automatically searches for configuration files:
# 1. /etc/baliusd/daemon.toml (system-wide, optional)
# 2. baliusd.toml (current directory, optional)

cd /home/benja/Work/balius/test
baliusd
```

### Debug Mode (Ephemeral Storage)

```bash
# Run with ephemeral storage (doesn't persist state)
baliusd --debug
```

### Verify Installation

```bash
which baliusd
baliusd --version
```

---

## API Access

When running, baliusd exposes JSON-RPC APIs for each worker:

```
http://{host}:{port}/{worker-name}
```

### Example with test worker on port 3001:

```bash
# Call the get-latest-block endpoint
curl -X POST http://localhost:3001/test \
  -H "Content-Type: application/json" \
  -d '{
    "id": "1",
    "method": "get-latest-block",
    "params": {}
  }'
```

---

## Module Configuration Details

### 1. Ledger Module

**Required** - Connects to Cardano blockchain via UTxORPC

```toml
[ledger]
endpoint_url = "https://cardano-preview.utxorpc-m1.demeter.run"
headers."dmtr-api-key" = "your-api-key"
```

**Worker Usage**:
```rust
use balius_sdk::wit::balius::app::ledger;

let utxos = ledger::read_utxos(&txo_refs)?;
let params = ledger::read_params()?;
```

---

### 2. HTTP Module

**Enabled by default** - Make HTTP requests to external APIs

```toml
[http]
type = "reqwest"
timeout = 30  # seconds
```

**Worker Usage**:
```rust
use balius_sdk::http::HttpRequest;
use url::Url;

let url = Url::parse("https://api.example.com/data")?;
let response = HttpRequest::get(url)
    .header("Authorization", "Bearer token")
    .send()?;
```

---

### 3. Sign Module

**Enabled by default** - Sign transactions and payloads

```toml
[signing]
type = "memory"

# Optional: Pre-configure keys
[[signing.keys]]
worker = "test"
name = "payment-key"
algorithm = "ed25519"
private_key = "5820..." # Hex-encoded ED25519 private key (64 bytes)
```

**Worker Usage**:
```rust
use balius_sdk::wit::balius::app::sign;

#[balius_sdk::main]
fn main() -> Worker {
    Worker::new()
        .with_signer("payment-key", "ed25519")  // Register key
        .with_request_handler("sign-tx", FnHandler::from(sign_handler))
}

fn sign_handler(...) -> WorkerResult<...> {
    let signature = sign::sign_payload("payment-key", &payload)?;
    // signature is Vec<u8>
}
```

**Get Public Key** from configured key:
```bash
baliusd get-public-key <worker-name> <key-name>
# Example:
baliusd get-public-key test payment-key
```

---

### 4. KV Module

**Enabled by default** - Persistent key-value storage

```toml
# Option 1: In-memory (no persistence)
[kv]
type = "memory"

# Option 2: Persistent Redb
[kv]
type = "redb"
path = "kv.db"
cache_size = 1024
```

**Worker Usage**:
```rust
use balius_sdk::wit::balius::app::kv;

kv::set_value("user:123", b"data")?;
let value = kv::get_value("user:123")?;
let keys = kv::list_values("user:")?;
```

---

### 5. Logger Module

**Enabled by default** - Structured logging for workers

```toml
# Option 1: Tracing (outputs to daemon logs)
[logger]
type = "tracing"

# Option 2: File logging
[logger]
type = "file"
folder = "./logs"

# Option 3: Silent
[logger]
type = "silent"
```

**Worker Usage**:
```rust
use balius_sdk::wit::balius::app::logging;

#[balius_sdk::main]
fn main() -> Worker {
    balius_sdk::logging::init();  // Initialize first
    Worker::new()
        .with_request_handler("my-handler", FnHandler::from(my_handler))
}

fn my_handler(...) -> WorkerResult<...> {
    logging::log(logging::Level::Info, "my-handler", "Processing request");
    // Your logic
}
```

---

### 6. Submit Module

**❌ NOT YET AVAILABLE in baliusd**

The module exists in `balius-runtime` but is not integrated into the baliusd configuration system.

**Workaround**: Handle transaction submission outside Balius:

1. **Via Blockfrost**:
```rust
// In your worker, return the signed tx_cbor to the caller
// Then submit externally via Blockfrost API
```

```bash
curl -X POST "https://cardano-preprod.blockfrost.io/api/v0/tx/submit" \
  -H "project_id: your-project-id" \
  -H "Content-Type: application/cbor" \
  --data-binary @transaction.cbor
```

2. **Via cardano-submit-api**:
```bash
curl -X POST "http://localhost:8090/api/submit/tx" \
  -H "Content-Type: application/cbor" \
  --data-binary @transaction.cbor
```

3. **Via cardano-cli**:
```bash
cardano-cli transaction submit \
  --tx-file transaction.signed \
  --testnet-magic 1
```

---

## Running Multiple Workers

One of baliusd's key features is running multiple dApps in a single process:

```toml
[[workers]]
name = "wallet"
module = "bin/wallet.wasm"
config = "wallet.json"

[[workers]]
name = "oracle"
module = "bin/oracle.wasm"
config = "oracle.json"

[[workers]]
name = "dex"
module = "bin/dex.wasm"
config = "dex.json"
```

**API Access**:
- `http://localhost:3001/wallet` - Wallet worker
- `http://localhost:3001/oracle` - Oracle worker
- `http://localhost:3001/dex` - DEX worker

---

## Event Processing

baliusd automatically handles blockchain events via the chainsync driver:

```toml
[chainsync]
endpoint_url = "https://cardano-preview.utxorpc-m1.demeter.run"
headers."dmtr-api-key" = "your-api-key"
```

Workers can listen for events:

```rust
use balius_sdk::{Event, EventHandler};

struct MyEventHandler;

impl EventHandler for MyEventHandler {
    fn handle_utxo(&self, utxo: UtxoEvent) -> WorkerResult<()> {
        // Process UTXO events
        Ok(())
    }

    fn handle_tx(&self, tx: TxEvent) -> WorkerResult<()> {
        // Process transaction events
        Ok(())
    }
}

#[balius_sdk::main]
fn main() -> Worker {
    Worker::new()
        .with_event_handler(MyEventHandler)
}
```

---

## Comparison: cargo balius test vs baliusd

| Feature | `cargo balius test` | `baliusd` |
|---------|-------------------|-----------|
| Purpose | Quick development testing | Production runtime |
| Configuration | Code changes required | TOML files |
| Module Support | Limited, manual | Full, configurable |
| Multiple Workers | No | Yes |
| Event Processing | Basic | Full chainsync integration |
| Metrics | No | Optional Prometheus |
| Logging | Basic | Configurable (Silent/Tracing/File) |
| State Persistence | Mock/ephemeral | Configurable (Mock/Memory/Redb) |
| Deployment Ready | ❌ No | ✅ Yes |

---

## Migration Path: From Test to Production

### Development (Quick Testing)
```bash
# Quick iteration during development
cargo balius build
cargo balius test --port 3001 --config ./config.json \
  --utxo-url "cardano-preview.utxorpc-m1.demeter.run" \
  --utxo-api-key "your-key"
```

### Production (baliusd)
```bash
# Build worker
cargo balius build

# Create baliusd.toml with all settings
# Start daemon
baliusd

# Or with debug mode (ephemeral)
baliusd --debug
```

---

## Example: Complete Setup for test Worker

### 1. Build Worker
```bash
cd /home/benja/Work/balius/test
cargo balius build
```

### 2. Create baliusd.toml
```toml
[rpc]
listen_address = "0.0.0.0:3001"

[logging]
max_level = "debug"
include_tokio = false

[ledger]
endpoint_url = "https://cardano-preview.utxorpc-m1.demeter.run"
headers."dmtr-api-key" = "utxorpc120sj2h7w5h5kjy0cey9"

[chainsync]
endpoint_url = "https://cardano-preview.utxorpc-m1.demeter.run"
headers."dmtr-api-key" = "utxorpc120sj2h7w5h5kjy0cey9"

[http]
type = "reqwest"
timeout = 30

[kv]
type = "memory"

[logger]
type = "tracing"

[signing]
type = "memory"

[store]
path = "baliusd.db"

[[workers]]
name = "test"
module = "test-c.wasm"
config = "config.json"
```

### 3. Ensure config.json exists
```json
{
  "project_id": "preprod6Zlqd54IUdWzdIG85rBE4BCnK0M78nsI"
}
```

### 4. Run baliusd
```bash
baliusd
```

### 5. Test API
```bash
curl -X POST http://localhost:3001/test \
  -H "Content-Type: application/json" \
  -d '{"id": "1", "method": "get-latest-block", "params": {}}'
```

---

## Troubleshooting

### Error: "component imports instance `balius:app/submit@0.1.0`"

**Cause**: Worker imports a module (like submit) that baliusd doesn't provide.

**Solution**:
1. Remove/comment out the import in worker code
2. Or wait for module integration into baliusd
3. Or handle functionality externally

### Error: "Database already open. Cannot acquire lock."

**Cause**: Previous baliusd instance didn't shut down cleanly.

**Solution**:
```bash
rm -f baliusd.db
baliusd
```

### Worker Registration Fails

**Check**:
1. WASM file exists and is recent: `ls -lh test-c.wasm`
2. Worker config file exists: `ls -lh config.json`
3. baliusd.toml paths are correct (relative to where you run baliusd)

---

## Key Files Reference

- **baliusd source**: `/home/benja/Work/balius/balius/baliusd/src/main.rs`
- **Config types**: `/home/benja/Work/balius/balius/baliusd/src/config.rs`
- **Example configs**: `/home/benja/Work/balius/balius/baliusd/example-preview/`
- **Runtime modules**: `/home/benja/Work/balius/balius/balius-runtime/src/`
- **WIT interface**: `/home/benja/Work/balius/balius/wit/balius.wit`

---

## Next Steps

1. **Implement Sign Endpoint**: Add transaction signing functionality to your worker
2. **External Submit**: Set up external transaction submission (Blockfrost/submit-api)
3. **Production Config**: Create production baliusd.toml with proper keys and persistence
4. **Monitoring**: Add metrics endpoint for production monitoring
5. **Multi-Worker**: Explore running multiple workers in single daemon

---

## Conclusion

**Use `baliusd` for production workloads**. It provides:
- Full module support out of the box
- Configuration-driven architecture
- Proper lifecycle management
- Production-grade features (metrics, logging, persistence)

**Use `cargo balius test` only for**:
- Quick development iteration
- Testing small changes
- When you don't need full daemon features

The `baliusd` daemon is the proper, supported way to run Balius workers in production environments.
