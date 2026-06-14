# Handoff Documentation

This document provides technical details for developers who will continue the development of the Pearl Miner.

## Current Architecture

The project is structured as a Rust workspace with the following key crates:

- `apps/miner-cli`: The main CLI entry point.
- `core/config`: Configuration loading and validation.
- `core/stratum`: Stratum V1 client and protocol handling.
- `core/mining`: Core traits for algorithms and backends.
- `core/watchdog`: Supervisor for external mining processes.
- `algos/pearl`: Synthetic implementation of the Pearl algorithm.
- `backends/cpu`: Multi-threaded CPU mining backend.
- `integrations/pearlpool`: PearlPool specific protocol adapters and command builders.

## What Works

- **Compatibility Mode**: Reliable execution and monitoring of external miners.
- **Config System**: Handles `config.toml` and CLI overrides effectively.
- **Watchdog**: Automatic restarts and log streaming for external miners.
- **Native CPU Foundation**: A working multi-threaded mining loop (synthetic).
- **Stratum Foundation**: Asynchronous handshake and job subscription.

## What is Intentionally Not Implemented

- **Real Pearl Algorithm**: Native mining uses a SHA256 placeholder.
- **Live PearlPool Verification**: `mining.notify` parsing and `mining.submit` format for PearlPool are unverified.
- **Active Dev-Fee Switching**: The infrastructure for switching wallets on the fly is not complete.
- **GPU Mining**: Backends for CUDA, HIP, etc., are only defined as traits.

## How to Run

### Compatibility Mode (Recommended for Live Mining)
```bash
cargo run -- --mode compatibility --miner-binary path/to/miner --pool stratum+tcp://... --wallet <address>
```

### Native CPU Synthetic Mode (Offline)
```bash
cargo run -- --mode native-cpu
```

### Native CPU Experimental Mode (Live Stratum)
```bash
cargo run -- --mode native-cpu --allow-experimental-live-stratum --pool stratum+tcp://... --wallet <address>
```

## Implementation Guide for Future Developers

### 1. Real Pearl Algorithm
Implement the `MiningAlgorithm` trait in `algos/pearl/src/lib.rs`. Replace the synthetic SHA256 implementation with the actual Pearl algorithm.

### 2. Live PearlPool Job/Submit Parsing
Update `integrations/pearlpool/src/lib.rs`.
- Refine `parse_job` to handle real PearlPool `mining.notify` payloads.
- Implement `build_share_submit` to create valid `mining.submit` JSON-RPC requests.

### 3. GPU Backends
Create new crates in `backends/` (e.g., `backends/cuda`, `backends/hip`) and implement the `MiningBackend` trait.

### 4. Dev-Fee Identity Switching
Update `core/stratum/src/miner_loop.rs` and `core/scheduler/src/lib.rs` to handle re-authorization on the Stratum connection when the developer fee cycle starts.

## CI and Testing
- Run tests: `cargo test --workspace`
- Run clippy: `cargo clippy --workspace -- -D warnings`
- Run fmt: `cargo fmt --all -- --check`
