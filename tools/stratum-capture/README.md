# Stratum Capture Tool

A standalone utility designed to securely capture and redact Stratum JSON-RPC traffic.

## Purpose
This tool acts as a TCP proxy between an external miner and a remote Stratum pool. It records the session traffic and automatically redacts sensitive data (wallets, passwords, workers) to generate sanitized JSONL fixtures.

## Warning
**NEVER COMMIT RAW UNREDACTED CAPTURES TO GIT.**

Always review the generated `.jsonl` files before committing them. Look for leaked IP addresses or unredacted passwords.

## Build

```bash
cd tools/stratum-capture
cargo build --release
```

## Running

1. **Start the proxy:**
   Point the capture tool to the PearlPool endpoint and specify redaction fields.

   ```bash
   ./target/release/stratum-capture \
     --listen 127.0.0.1:3333 \
     --pool pearlpool.cloud:5566 \
     --output ./capture.jsonl \
     --redact-wallet 1MySecretWallet... \
     --redact-worker myworker1
   ```

2. **Start your miner:**
   Configure your external miner (e.g. lpminer, SRBMiner) to point to the local capture tool.

   ```bash
   ./miners/lpminer/lpminer.exe --pool stratum+tcp://127.0.0.1:3333 --wallet 1MySecretWallet... --worker myworker1
   ```

## What is Captured
- Bidirectional TCP data, line by line.
- Extracted JSON-RPC shapes (method, id type, params length/keys, result keys).

## What is Redacted
- Explicit matching strings given via CLI (`--redact-wallet`, `--redact-worker`, `--redact-password`).
- JSON values matching keys like `user`, `pass`, `password`, `worker`, `wallet`, `login`, `address`.

## Fixture Review Checklist
- [ ] No occurrences of your real wallet address.
- [ ] No occurrences of your real worker name.
- [ ] No occurrences of your pool password.
- [ ] `redaction_applied` boolean is checked if appropriate.
