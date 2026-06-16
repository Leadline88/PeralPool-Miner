# Stratum Capture Tool

A standalone Stratum JSON-RPC proxy and capture tool for recording and redacting mining pool traffic.

## Purpose

This tool acts as a transparent TCP proxy between a miner and a pool. It logs every line of communication, redacting sensitive information like wallets, workers, and passwords, and produces a structured JSONL fixture file for testing and research.

## Safety Warning

**NEVER commit unredacted captures to the repository.**
Always review the output file before sharing or committing. While the tool performs automatic redaction, it is your responsibility to ensure no PII (Personally Identifiable Information) or secrets remain.

## Build

```bash
cd tools/stratum-capture
cargo build
```

## Test

```bash
cd tools/stratum-capture
cargo test
```

## Usage

### Dry Run (Validation)

Validate your arguments without opening any sockets:

```bash
cargo run -- --pool pearlpool.cloud:5566 --output test.jsonl --dry-run
```

### Local Capture Proxy

Run the proxy to listen on `127.0.0.1:3333` and forward to PearlPool:

```bash
cargo run -- \
  --listen 127.0.0.1:3333 \
  --pool pearlpool.cloud:5566 \
  --output my_capture.jsonl \
  --redact-wallet <YOUR_WALLET> \
  --redact-worker <YOUR_WORKER>
```

Then, point your external miner (e.g., `lpminer`) to `127.0.0.1:3333` instead of the pool's real address.

### Automatic Stop

You can limit the capture session by lines or duration:

```bash
# Stop after 100 lines
cargo run -- --pool ... --output ... --max-lines 100

# Stop after 60 seconds
cargo run -- --pool ... --output ... --max-seconds 60
```

## What gets captured

- Timestamps (UTC)
- Direction (`client_to_pool`, `pool_to_client`, `proxy_event`)
- Redacted raw lines
- Extracted JSON-RPC metadata (method, id, shapes of params/result/error)
- Proxy events (connect, disconnect)

## Redaction Rules

- Exact matches for `--redact-wallet`, `--redact-worker`, and `--redact-password` are replaced with `<REDACTED_WALLET>`, etc.
- JSON fields named `user`, `pass`, `password`, `worker`, `wallet`, `login`, `address` are automatically redacted recursively.
- Placeholder values:
    - Wallet/Address/Login/User: `<REDACTED_WALLET_OR_USER>`
    - Worker: `<REDACTED_WORKER>`
    - Password: `<REDACTED_PASSWORD>`

## Fixture Review Checklist

Before using a capture for tests or documentation:
1. [ ] No real IP addresses in `raw_line_redacted`.
2. [ ] No real wallet addresses or worker names.
3. [ ] No real passwords.
4. [ ] JSON-RPC structure is intact but values are safely placeholderized.
