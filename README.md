# Pearl Miner

A modular, extensible cryptocurrency miner written in Rust.

Pearl Miner is pivoting from a simple external miner manager into a fully native Pearl miner. It supports multiple backends and provides a transparent mining experience.

## Features

- **Multiple Operating Modes**:
  - `compatibility`: Launch and supervise external miner binaries.
  - `native-cpu`: Run the internal CPU reference miner.
  - `native-gpu`: Future GPU backend path.
- **Unified Configuration**: Load settings from `config.toml` or via CLI arguments.
- **Transparent Developer Fee**: 1.0% fee to support ongoing development.
- **Process Watchdog**: Automatically restarts external miners in compatibility mode.
- **Log Streaming**: Clean, unified logging for all modes.

## Transparent 1% Developer Fee

Pearl Miner includes a transparent 1.0% developer fee. This fee is used to fund the development and maintenance of the project.

- **Default Fee**: 1.0%
- **Visibility**: The fee is clearly displayed at startup and in status outputs.
- **Honesty**: No hidden telemetry, no stealth mining, no misleading performance claims.

Use the `--dev-fee-info` flag to view detailed fee information.

## Roadmap

- [x] Native miner architecture foundation
- [x] Transparent developer fee implementation
- [x] CLI & Configuration overhaul
- [ ] Internal Pearl CPU miner implementation
- [ ] CUDA backend for NVIDIA GPUs
- [ ] HIP/OpenCL backend for AMD GPUs
- [ ] Metal backend for Apple Silicon
- [ ] SYCL/OpenCL backend for Intel GPUs

## Capability Matrix

| OS      | CPU | CUDA | HIP/OpenCL | SYCL/OpenCL | Metal |
|---------|-----|------|------------|-------------|-------|
| Windows | ✅  | 🛠️   | 🛠️         | 🛠️          | ❌    |
| Linux   | ✅  | 🛠️   | 🛠️         | 🛠️          | ❌    |
| macOS   | ✅  | ❌   | ❌         | ❌          | 🛠️    |

Legend: ✅ Functional, 🛠️ Planned/In Progress, ❌ Not Supported

## Setup

1. **Clone the repository:**
   ```bash
   git clone https://github.com/your-org/pearl-miner.git
   cd pearl-miner
   ```

2. **Build the project:**
   ```bash
   cargo build --release
   ```

3. **Run the Miner:**
   ```bash
   # Run with CLI arguments
   cargo run --bin miner-cli -- --mode native-cpu --wallet <your-wallet> --pool <pool-url>

   # Or use a config file
   cargo run --bin miner-cli -- --config config.toml
   ```

## Configuration

Example `config.toml`:

```toml
wallet = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"
worker_name = "worker1"
pool_url = "stratum+tcp://pearlpool.cloud:5566"
mode = "native-cpu"
backend = "cpu"
algo = "pearl"
```

In `compatibility` mode, you also need:

```toml
mode = "compatibility"
miner_binary_path = "./miners/lpminer/lpminer.exe"
args = ["--algo", "pearl", "--pool", "stratum+tcp://pearlpool.cloud:5566"]
```

## Security and Transparency Promise

We believe in open-source and honest mining software.
- We will never implement stealth mining.
- We will never hide or obfuscate the developer fee.
- We will never add hidden telemetry or backdoors.
- Performance claims will always be verifiable.
