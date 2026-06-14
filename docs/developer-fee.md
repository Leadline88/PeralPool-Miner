# Developer Fee Policy

Pearl Miner defines a transparent 1.0% developer fee policy.

## Policy Details

- **Default Fee**: 1.0%
- **Status**: **Defined but Not Actively Collected in Native Mode.**
- **Wallet**: `1DevFeeAddressExample`

## Mechanism (Future)

The intended mechanism is periodic identity switching (re-authorization) on the Stratum connection.

- **User Period**: 3564 seconds (99.0%)
- **Developer Period**: 36 seconds (1.0%)

## Current Foundation Implementation

- **Scheduler**: The `DevFeeScheduler` is active and toggles `DevFeeState`.
- **Identity Switching**: **Not implemented.**
- **Native Mining**: In the current foundation mode, all shares are submitted under the user's wallet, even during "scheduled" developer periods. The status will show `ScheduledInactive` during these times.

This ensures that no "stealth mining" occurs and the user's hashrate is never diverted without explicit, verified implementation of the switching logic.
