# Share Lifecycle

This document describes how a share moves through the Pearl miner system.

## 1. Candidate Generation
When a GPU kernel finds a hash meeting the current pool difficulty, it generates a `ShareCandidate`.
- `job_id`: The ID of the job being worked on.
- `nonce`: The nonce that produced the valid hash.
- `timestamp`: When the share was found.

## 2. Submission
The `ShareCandidate` is sent via a channel to the `MinerLoop`.
The `MinerLoop` uses the `PoolAdapter` to format a `mining.submit` request.

## 3. Verification (Pool Side)
The pool receives the submission and responds with a JSON-RPC result.
- `result: true`: Share accepted.
- `error: [...]`: Share rejected (e.g., stale, low difficulty, duplicate).

## 4. Result Recording
The `MinerLoop` parses the response into a `ShareResult` and updates the `ShareTracker`.
- Latency is calculated from submission to response.
- Counters for `Accepted`, `Rejected`, `Stale`, and `Invalid` are updated.
