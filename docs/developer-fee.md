# Transparent Developer Fee

Pearl Miner implements a transparent 1.0% developer fee to support the ongoing development and maintenance of the project.

## Fee Structure

- **Percentage**: 1.0%
- **Mechanism**: The miner periodically switches from mining for the user's wallet to mining for the developer's wallet.
- **Cycle**: A total cycle is 10 minutes (600 seconds).
    - **User Mining**: 594 seconds (99.0%)
    - **Developer Mining**: 6 seconds (1.0%)

## Transparency

We believe in full transparency regarding developer fees.

1. **Startup Disclosure**: The fee percentage is clearly displayed in the startup banner.
2. **CLI Flag**: The `--dev-fee-info` flag provides detailed information about the fee and the developer wallet.
3. **Runtime Status**: The runtime status output clearly indicates whether the miner is currently working for the user or the developer.
4. **No Obfuscation**: The scheduling logic is open-source and easy to verify in `core/scheduler/src/lib.rs`.

## Developer Wallet

The current developer fee wallet is: `1DevFeeAddressExample`

## Promise

- We will never implement "stealth" mining.
- The fee will always be clearly communicated.
- Performance metrics (hashrate) shown to the user will always be honest and not inflated to hide the fee impact.
