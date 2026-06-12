# Stratum Client

The `core/stratum` crate provides a native Stratum V1 client implementation for the Pearl miner.

## Features

- **Async TCP Connection**: Uses `tokio` for non-blocking I/O.
- **JSON-RPC 2.0**: Handles request-response matching and notifications.
- **Automatic Reconnection**: Exponential backoff logic to maintain pool connectivity.
- **Graceful Shutdown**: Integrated with `CancellationToken` for clean exits.
- **URL Prefix Handling**: Automatically strips `stratum+tcp://` from pool URLs.
- **Event-Driven**: Broadcasts events like `Connected`, `Disconnected`, `Job`, and `Difficulty`.

## Handshake Flow

The client follows the standard Stratum V1 handshake:
1. `mining.subscribe`: Negotiates protocol features and receives extranonces.
2. `mining.authorize`: Authenticates the worker using wallet and name.

## Components

- `StratumClient`: The main connection manager.
- `PoolAdapter`: A trait for implementing pool-specific protocol variations.
- `MinerLoop`: Orchestrates the connection, handshake, and share submission.

## Usage

```rust
let adapter = Arc::new(PearlPoolAdapter);
let client = StratumClient::new("stratum+tcp://pearlpool.cloud:5566", adapter.clone());
let cancel_token = CancellationToken::new();

// Run the client in a background task
tokio::spawn(client.clone().run(cancel_token.clone()));

// Use MinerLoop to handle logic
let miner = MinerLoop::new(client, adapter, wallet, worker);
miner.run(share_rx, cancel_token).await;
```
