# Stratum Redaction Policy

To ensure privacy and security, all captured Stratum fixtures must be redacted before being stored in the repository.

## Redaction Requirements

The following information **MUST** be redacted:

| Field | Redaction Method | Example |
|-------|------------------|---------|
| Wallet Addresses | Replace with placeholder | `1UserWalletAddress_REDACTED` |
| Worker Names | Replace with placeholder or generic name | `worker_1` |
| IP Addresses | Remove or mask | `0.0.0.0` or `redacted-pool-ip` |
| Passwords/Tokens | Replace with `x` or generic placeholder | `x` |
| Machine Identifiers | Remove entirely | N/A |
| Timestamps | Remove if present in metadata | N/A |
| Extranonce (partially) | Keep length but randomize content if sensitive | `0123abcd` |

## Allowed Data

The following information **MAY** remain in the fixtures:

- **Method Names:** Essential for protocol verification (e.g., `mining.subscribe`).
- **JSON-RPC Structure:** `id`, `method`, `params`, `result`, `error` keys.
- **Parameter Shapes:** The number and types of parameters in a method call.
- **Difficulty Values:** Numeric values for `mining.set_difficulty`.
- **Job Field Names:** Names of fields in `mining.notify`.
- **Sanitized IDs:** JSON-RPC request IDs (usually small integers).
- **Protocol Constants:** Version strings that are not unique to a user.

## Verification Procedure

Before committing any fixture:
1. Run a grep search for the original wallet address.
2. Run a grep search for the original pool IP.
3. Manually inspect the JSONL file to ensure no sensitive metadata was leaked.
