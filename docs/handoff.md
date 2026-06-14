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

## What Works Now

- **Compatibility Mode**: Reliable execution and monitoring of external miners.
- **Config System**: Handles `config.toml`, profiles, and CLI overrides effectively.
- **Watchdog**: Automatic restarts and log streaming for external miners.
- **Native CPU Foundation**: A working multi-threaded mining loop (synthetic).
- **Stratum Foundation**: Asynchronous handshake and job subscription.

## What is Intentionally Unsupported

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

## Where Future Developers Should Start

1.  **Pearl Algorithm Integration**: Implement the real Pearl algorithm in `algos/pearl`.
2.  **PearlPool Protocol Capture**: Use a proxy to capture real PearlPool Stratum traffic to verify `mining.notify` and `mining.submit` formats.
3.  **Share Submission**: Update `integrations/pearlpool` to build valid `mining.submit` requests once the format is known.
4.  **GPU Backends**: Implement the `MiningBackend` trait for CUDA, HIP, etc.
5.  **Active Dev-Fee Collection**: Implement re-authorization logic in the `MinerLoop` to support active wallet switching for the developer fee.
