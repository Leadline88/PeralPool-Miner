# Pearl Miner

A modular, extensible cryptocurrency miner written in Rust.

Pearl Miner provides a foundation/scaffold for a future native miner, while supporting compatibility mode for existing external miners.

## Features

- **Multiple Operating Modes**:
  - `compatibility`: Launch and supervise external miner binaries (e.g., lpminer, SRBMiner).
  - `native-cpu`: Run the internal CPU reference miner (Synthetic/Reference-only).
- **Unified Configuration**: Load settings from `config.toml` or via CLI arguments.
- **Transparent Developer Fee**: 1.0% fee policy defined for future use.
- **Process Watchdog**: Automatically restarts external miners in compatibility mode.
- **Log Streaming**: Clean, unified logging for all modes.

## Current Status: Foundation Freeze

The repository is currently in a "Foundation Freeze" state. It provides a clean, compiling foundation and architecture, but is not yet production-ready for native mining.

- **Compatibility mode**: Available and functional ✅
- **Native CPU reference miner**: Synthetic/Reference-only implementation ✅
- **Real Pearl Algorithm**: Not implemented ❌
- **Live PearlPool mining**: Not verified/experimental 🛠️
- **PearlPool share submission**: Unsupported (Stubbed) 🛠️
- **Developer Fee**: Policy defined, active wallet switching not implemented in native mode 🛠️
- **GPU Backends**: Planned only (Traits/interfaces defined) 🛠️

## Native Mode Truthfulness

- **Native CPU**: Synthetic/reference-only implementation based on SHA256.
- **Pearl Algorithm**: The real Pearl mainnet algorithm is **NOT** implemented.
- **Live PearlPool**: Job parsing is unverified and experimental.
- **Share Submission**: Live PearlPool share submission is **UNSUPPORTED** until the format is verified.
- **Developer Fee**: The 1% fee is scheduled by the internal timer, but **NOT** actively collected in native mode (identity switching is pending).

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

## Security and Transparency Promise

- No stealth mining.
- No hidden or obfuscated developer fee.
- No hidden telemetry or backdoors.
- Honest reporting of implementation status.
