# Foundation Status

This repository serves as a **clean, honest foundation and scaffold** for the Pearl Miner.

## What is Implemented

- **Compatibility Mode**: Full support for launching and supervising external miner binaries.
- **Watchdog/Process Supervisor**: Monitors external processes and handles restarts.
- **Log Parser**: Unified streaming and parsing of external miner logs.
- **Config System**: Robust TOML-based configuration and CLI overrides.
- **Stratum Foundation**: Asynchronous Stratum V1 client with handshake support.
- **Trait Architecture**: Clean interfaces for `MiningAlgorithm`, `MiningBackend`, and `PoolAdapter`.
- **Runtime Stats**: Basic tracking of shares, hashrate, and uptime.
- **Synthetic CPU Mode**: A reference-only CPU miner for architectural verification.

## What is NOT Implemented (Future Work)

- **Real Pearl Algorithm**: The mainnet Pearl algorithm is not yet integrated.
- **Live PearlPool Verification**: Native job parsing and share submission are unverified.
- **Active Developer Fee Switching**: Identity re-authorization for fee collection is pending.
- **GPU Backends**: CUDA, HIP, OpenCL, SYCL, and Metal are design-only.
- **Performance Optimizations**: Native mining is currently unoptimized.

## Architectural Goals

The goal of this foundation is to provide future developers with a stable, compiling, and well-documented starting point. All incomplete features fail safely with clear `NotImplemented` or `Unsupported` errors.
