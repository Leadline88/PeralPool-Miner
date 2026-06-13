# Pearl CPU Reference Miner

The `native-cpu` mode in Pearl Miner provides a correctness-first reference implementation of the Pearl mining algorithm.

## Overview

The CPU miner is designed for:
- Correctness verification against test vectors.
- Educational reference for the Pearl algorithm.
- Initial bootstrapping of the miner architecture.
- Baseline for performance comparisons with GPU backends.

## Implementation Details

- **Algorithm**: Correctness-first reference implementation. Currently uses a synthetic SHA256 as a placeholder for the final Pearl hash.
- **Backend**: Multi-threaded using Tokio tasks.
- **Nonce Partitioning**: Nonce ranges are partitioned across threads without overlap.
- **Cancellation**: Workers are immediately cancelled when a new job arrives.
- **Resource Management**: Uses `tokio::task::yield_now()` every 10,000 nonces to ensure the async executor is not starved.

## Usage

To run the native CPU miner:

```bash
cargo run --bin miner-cli -- --mode native-cpu --wallet <your-wallet> --pool <pool-url>
```

### Options

- `--threads <n>`: Specify the number of mining threads. Defaults to the number of logical CPU cores.
- `--deterministic`: Enables deterministic mode for testing purposes.
- `--verify-pearl-fixture <path>`: Verifies a Pearl fixture JSON file.

## Performance Warning

> [!WARNING]
> This implementation is optimized for **correctness and transparency**, not for raw speed. It is not intended to be performance-competitive with specialized GPU mining software.
