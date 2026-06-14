# Stratum Status

The `core/stratum` crate provides a native Stratum V1 client foundation.

## Implementation Status

- **Connection Management**: Stable async TCP connection with reconnection logic.
- **Protocol Support**: JSON-RPC 2.0 requests, responses, and notifications.
- **Handshake**: Standard `mining.subscribe` and `mining.authorize` flow.
- **Mock Support**: `MockPoolAdapter` and `MockStratumServer` for testing.

## Unverified/Unsupported Features

- **Live PearlPool Jobs**: Receiving jobs from a live pool is experimental and unverified.
- **Live PearlPool Shares**: Building `mining.submit` payloads for live PearlPool is **UNSUPPORTED**.
- **PearlPoolAdapter**: Strictly returns `UnsupportedRealPearlShareSubmitFormat` for all live submission attempts to prevent sending invalid data to pools.

Future developers should verify the exact PearlPool Stratum dialect before enabling live share submissions.
