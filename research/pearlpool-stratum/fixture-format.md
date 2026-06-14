# Stratum Fixture Format

This document defines the sanitized JSONL (JSON Lines) format for PearlPool Stratum fixtures. Each line in a `.jsonl` file represents a single Stratum message (request, response, or notification).

## Format Structure

Each entry is a JSON object with the following fields:
- `direction`: `in` (from pool to miner) or `out` (from miner to pool).
- `payload`: The actual JSON-RPC message.
- `metadata`: (Optional) Additional context such as connection events.

## Message Types

### 1. Client Subscribe Request (`out`)
```json
{"direction": "out", "payload": {"id": 1, "method": "mining.subscribe", "params": ["pearl-miner/0.1.0"]}}
```

### 2. Server Subscribe Response (`in`)
```json
{"direction": "in", "payload": {"id": 1, "result": [["mining.set_difficulty", "deadbeef"], "01234567"], "error": null}}
```

### 3. Client Authorize Request (`out`)
```json
{"direction": "out", "payload": {"id": 2, "method": "mining.authorize", "params": ["1UserWalletAddress_REDACTED", "x"]}}
```

### 4. Server Authorize Response (`in`)
```json
{"direction": "in", "payload": {"id": 2, "result": true, "error": null}}
```

### 5. Mining Set Difficulty (`in`)
```json
{"direction": "in", "payload": {"id": null, "method": "mining.set_difficulty", "params": [0.5]}}
```

### 6. Mining Notify (`in`)
```json
{"direction": "in", "payload": {"id": null, "method": "mining.notify", "params": ["job_id_redacted", "prevhash_redacted", "coinb1_redacted", "coinb2_redacted", [], "00000001", "1d00ffff", "504c2162", true]}}
```

### 7. Mining Submit Request (`out`)
```json
{"direction": "out", "payload": {"id": 3, "method": "mining.submit", "params": ["1UserWalletAddress_REDACTED", "job_id_redacted", "extranonce2_redacted", "ntime_redacted", "nonce_redacted"]}}
```

### 8. Mining Submit Response (`in`)
```json
{"direction": "in", "payload": {"id": 3, "result": true, "error": null}}
```

### 9. Error Response (`in`)
```json
{"direction": "in", "payload": {"id": 3, "result": null, "error": [20, "Other/Unknown", null]}}
```

### 10. Connection Events
```json
{"metadata": {"event": "disconnect", "reason": "socket_closed"}}
```
