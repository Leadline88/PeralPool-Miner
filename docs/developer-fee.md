# Developer Fee Policy

Pearl Miner includes a transparent 1.0% developer fee to support ongoing development.

## Current Status: Policy Only

The infrastructure for the developer fee is implemented at the foundation level, but **active collection is not implemented** in the native mining mode.

- **Policy**: 1.0% (36 seconds out of every 3600 seconds).
- **Implementation**: The `DevFeeScheduler` tracks the cycle and updates the `DevFeeState`.
- **Native Mode**: In native-cpu mode, the scheduler is either disabled (offline mode) or runs in a "ScheduledInactive" state (experimental mode).
- **No Identity Switching**: The current implementation does not perform Stratum re-authorization. All shares found during the "developer window" are still submitted under the user's wallet.
- **No Active Collection**: Current native CPU mode does not collect any fee. The `ScheduledInactive` state is for policy visibility only. `ActiveDeveloperMining` is not yet possible in native mode as it requires Stratum identity switching.

## Placeholder Wallet

The address `1DevFeeAddressExample` is used throughout the code as a **placeholder and example only**. It is not a real wallet address and does not collect any funds.

## Future Implementation

Future developers should implement the following in `core/stratum/src/miner_loop.rs` to enable active collection:

1.  Monitor the `DevFeeState` from the `StatsManager`.
2.  When the state changes to `ActiveDeveloperMining`, send a new `mining.authorize` request with the developer's wallet and worker.
3.  Ensure all subsequent `mining.submit` requests use the developer's credentials until the cycle returns to `ActiveUserMining`.
4.  Re-authorize back to the user's wallet once the developer window closes.
