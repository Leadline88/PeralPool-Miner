# PearlPool Stratum Research

This directory contains research materials and preparation for capturing real PearlPool Stratum traffic fixtures. These fixtures are essential for verifying the compatibility and correctness of the Pearl miner's Stratum implementation against the live PearlPool protocol.

## Purpose

Real PearlPool fixtures are needed to:
- Verify the exact JSON-RPC message formats used by PearlPool.
- Ensure that the miner correctly handles pool-specific extensions or variations in the Stratum V1 protocol.
- Provide a ground truth for integration tests without requiring a live network connection during every test run.
- Debug protocol-level issues by comparing miner behavior against known-good pool responses.

## Capture Scope

### What to Capture
- `mining.subscribe` requests and responses.
- `mining.authorize` requests and responses.
- `mining.set_difficulty` notifications.
- `mining.notify` jobs.
- `mining.submit` requests and their corresponding responses (accepted and rejected).
- Error responses for any of the above.
- Protocol-level events such as `client.reconnect` or unexpected socket closures.

### What MUST NOT be Captured (Redaction Required)
- **Wallet Addresses:** Must be replaced with placeholders (e.g., `1UserWalletAddress_REDACTED`).
- **Worker Names:** If they contain private or identifying information.
- **IP Addresses:** Both client and server IP addresses must be removed or masked.
- **Authentication Tokens:** Any passwords or session tokens.
- **Machine Identifiers:** Any unique IDs that could identify the specific hardware or operator.
- **Timestamps:** If they could be used to correlate the capture with specific network activity.

## Future Usage

The captured and redacted fixtures will be used to:
1. Populate a `MockPool` for high-fidelity offline integration testing.
2. Verify the `PearlPoolAdapter` implementation in `integrations/pearlpool`.
3. Drive automated regression tests for the `core/stratum` client.

## Implementation Disclaimer

**Important:** No live mining support is implemented in this research branch. These materials are for documentation and preparation purposes only. The existing miner implementation remains synthetic and reference-only as per the Foundation Freeze policy.
