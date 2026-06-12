# Pearl Miner CLI

A modular, extensible cryptocurrency miner manager written in Rust.

This repository provides the core foundation for a miner manager that launches and monitors external mining binaries (like `lpminer` or `SRBMiner`), providing auto-restart capabilities, clean shutdown handling, and unified configuration.

## Features

- **Configuration Management**: Loads settings from `config.toml` with strict validation.
- **Process Watchdog**: Automatically restarts the miner process if it crashes or exits unexpectedly.
- **Clean Shutdown**: Gracefully terminates the child miner process upon receiving a `Ctrl+C` signal.
- **Log Streaming**: Streams `stdout` and `stderr` from the miner binary into standard Rust tracing logs.

## Setup

1. **Clone the repository:**
   ```bash
   git clone https://github.com/your-org/pearl-miner.git
   cd pearl-miner
   ```

2. **Create a configuration file:**
   Create a `config.toml` in the root directory (or wherever you run the CLI from) using the following example:

   ```toml
   wallet = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"
   worker_name = "worker1"
   pool_url = "stratum+tcp://pearlpool.cloud:5566"
   miner_binary_path = "/usr/bin/echo" # Replace with actual miner path, e.g., "./miners/lpminer/lpminer.exe"
   args = ["--algo", "pearl", "--pool", "stratum+tcp://pearlpool.cloud:5566"]
   ```

3. **Build the project:**
   ```bash
   cargo build --release
   ```

4. **Run the Miner Manager:**
   ```bash
   cargo run --bin miner-cli
   ```

## Architecture

The workspace is organized into modular crates:

- `apps/miner-cli`: The main entrypoint.
- `core/config`: Configuration parsing and validation.
- `core/logging`: Centralized logging setup.
- `core/process`: Child process spawning and stream handling.
- `core/watchdog`: The monitor loop managing the miner's lifecycle.
- `integrations/pearlpool`: Future-proofing for PearlPool-specific integrations and defaults.
