# Native Miner Loop

The native miner loop is the central orchestration point of the Pearl miner. It connects the network layer (Stratum) with the processing layer (GPU kernels).

## Loop Responsibilities

1. **Connection Management**: Listens for `Connected` events from `StratumClient` and performs `mining.authorize`.
2. **Job Distribution**: Receives `mining.notify` calls and prepares them for the mining backend.
3. **Share Submission**: Receives solved `ShareCandidate`s from the backend and submits them to the pool via `mining.submit`.
4. **Statistics Tracking**: Updates the `ShareTracker` with accepted/rejected shares and current difficulty.

## State Machine

The loop operates as an async task, selecting over:
- Stratum events (Network -> Miner)
- Share candidates (Worker -> Miner)
- Shutdown signals
