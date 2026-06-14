# Stratum Capture Tool Design

This document outlines the design for a standalone Stratum capture tool. The tool is intended to be used as a proxy or a passive sniffer to record communication between a miner and a Stratum pool.

## Objective

Create a lightweight, standalone utility that can:
1. Act as a TCP proxy between a miner and a Stratum pool.
2. Log all bidirectional JSON-RPC traffic.
3. Automatically apply redaction rules defined in `research/pearlpool-stratum/redaction-policy.md`.
4. Output traffic in the format defined in `research/pearlpool-stratum/fixture-format.md`.

## Proposed Architecture

The tool should be implemented as a standalone Rust crate outside the main workspace to avoid dependency bloat and ensure it remains a pure research utility.

### Components

- **Proxy Listener:** Listens for incoming TCP connections from the miner.
- **Pool Client:** Establishes a TCP connection to the real Stratum pool.
- **Traffic Interceptor:** Asynchronously pipes data between the miner and the pool, parsing each line as a potential JSON-RPC message.
- **Redactor:** A module that identifies and masks sensitive fields based on regex patterns or JSON path selectors.
- **Logger:** Writes the sanitized messages to a `.jsonl` file.

### CLI Interface (Conceptual)

```bash
stratum-capture \
  --listen 127.0.0.1:3333 \
  --pool pearlpool.example.com:4444 \
  --output ./fixtures/capture.jsonl \
  --redact-wallet 1UserWalletAddress... \
  --redact-worker worker1
```

## Implementation Notes

- **No Dependency on `miner-cli`:** The tool must not depend on any miner logic. It is a protocol-agnostic proxy.
- **No Changes to `core/stratum`:** The capture tool uses its own minimal Stratum parsing logic to avoid introducing circular dependencies or modifying production code.
- **Async Runtime:** Likely `tokio` for efficient handling of concurrent TCP streams.

## Security

The capture tool handles sensitive information (wallets, passwords) before redaction. It must be used in a secure environment, and the unredacted logs must never be committed to version control.
