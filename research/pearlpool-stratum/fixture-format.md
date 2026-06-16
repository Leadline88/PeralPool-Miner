# Stratum Fixture Format

This document defines the sanitized JSONL (JSON Lines) format for PearlPool Stratum fixtures.

## Format Structure

Each line is a JSON object with the following fields:

- `timestamp_utc`: ISO8601 timestamp.
- `session_id`: Unique UUID for the proxy session.
- `session_label`: (Optional) User-defined label.
- `direction`: `in` (pool to miner) or `out` (miner to pool).
- `raw_line_redacted`: The actual JSON-RPC message after applying redaction rules.
- `jsonrpc`: Boolean indicating whether the line successfully parsed as JSON.
- `method`: String method name (if present).
- `id`: Type of the ID field (`number`, `string`, `null`, `other`).
- `params_shape`: Type/size description of the `params` field.
- `result_shape`: Type/size description of the `result` field.
- `error_shape`: Type/size description of the `error` field.
- `redaction_applied`: Boolean indicating if any data was altered for privacy.
- `notes`: (Optional) Manual annotations.

Connection events have a different schema containing `metadata`:
- `event`: e.g. `connect`, `disconnect`.
- `reason`: (Optional) reason for disconnect.
