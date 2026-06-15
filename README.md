# Pearl Miner

A modular, extensible cryptocurrency miner written in Rust.

Pearl Miner provides a foundation/scaffold for a future native miner, while supporting compatibility mode for existing external miners.

## Features

- **Multiple Operating Modes**:
  - `compatibility`: Launch and supervise external miner binaries (e.g., lpminer, SRBMiner).
  - `native-cpu`: Run the internal CPU reference miner (Synthetic/Offline Reference-only).
- **Unified Configuration**: Load settings from `config.toml` or via CLI arguments.
- **Transparent Developer Fee**: 1.0% fee policy defined for future use.
- **Process Watchdog**: Automatically restarts external miners in compatibility mode.
- **Log Streaming**: Clean, unified logging for all modes.

## Current Status: Foundation Freeze

The repository is currently in a "Foundation Freeze" state. It provides a clean, compiling foundation and architecture, but is not yet production-ready for native mining.

- **Compatibility mode**: Available ✅
- **Config/profile system**: Available ✅
- **Watchdog/process supervisor**: Available ✅
- **Log parser**: Available ✅
- **Native CPU**: Offline synthetic/reference-only ✅
- **Real Pearl algorithm**: Not implemented ❌
- **Live PearlPool job format**: Not verified ❌
- **Live PearlPool share submission**: Unsupported ❌
- **Stratum**: Foundation/mock/experimental only; live PearlPool mining is not verified. 🛠️
- **Developer fee**: Policy defined, active collection not implemented in native mode 🛠️
- **GPU backends**: Planned only 🛠️

## Mode Overview

- **Compatibility Mode**: Recommended for live PearlPool mining. Uses proven external miners.
- **Native CPU Mode**: Offline synthetic/reference-only implementation based on SHA256. Does not connect to pools by default.
- **Experimental Stratum**: Optional raw Stratum test path, unverified for PearlPool.

## Native Mode Truthfulness

- **Native CPU**: Synthetic implementation. Offline by default.
- **Pearl Algorithm**: The real Pearl mainnet algorithm is **NOT** implemented.
- **Live Stratum**: Foundation support exists, but live PearlPool mining is **UNVERIFIED**.
- **Share Submission**: Live PearlPool share submission is **UNSUPPORTED** in native mode.
- **Developer Fee**: The 1% fee policy is defined, but **NOT** actively collected in native mode.

## Benchmarking

Pearl Miner includes a built-in benchmark for the native CPU synthetic miner.

```bash
cargo run --release --bin miner-cli -- --benchmark-native-cpu
```

## Setup

1. **Clone the repository:**
   ```bash
   git clone https://github.com/Leadline88/PeralPool-Miner.git
   cd PeralPool-Miner
   ```

2. **Build the project:**
   ```bash
   cargo build --release
   ```

3. **Running the Miner:**

   ### Compatibility Mode (Recommended for Live Mining)
   Use this mode to run verified external miners (like lpminer or SRBMiner) while using Pearl Miner's management features.
   ```bash
   ./target/release/miner-cli --mode compatibility --miner-binary /path/to/lpminer --pool stratum+tcp://pearlpool.cloud:5566 --wallet <YOUR_WALLET>
   ```

   ### Native CPU Mode (Offline Synthetic/Reference Only)
   Use this mode to test the internal multi-threaded architecture. No pool connection is made.
   ```bash
   ./target/release/miner-cli --mode native-cpu
   ```

   ### Experimental Stratum Testing (Unverified/Advanced)
   **Warning:** This mode is for protocol development only. Live share submission is unsupported.
   ```bash
   ./target/release/miner-cli --mode native-cpu --allow-experimental-live-stratum --pool stratum+tcp://pearlpool.cloud:5566 --wallet <YOUR_WALLET>
   ```

## Security and Transparency Promise

- No stealth mining.
- No hidden or obfuscated developer fee.
- No hidden telemetry or backdoors.
- Honest reporting of implementation status.
