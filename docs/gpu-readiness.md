# GPU Readiness Roadmap

This document outlines the planned extension points for supporting GPU backends in Pearl Miner. **Note: No GPU code is currently implemented.**

## Extension Points

### MiningBackend Trait
All future GPU backends (CUDA, HIP, OpenCL, etc.) will implement the `MiningBackend` trait defined in `core/mining`. This ensures a consistent interface for starting, stopping, and job management.

### Job Distribution Model
Jobs will be distributed to GPU backends via the central `MinerLoop`. Each backend will be responsible for its own memory management and kernel execution.

### Share Candidate Flow
GPU backends will use the same asynchronous share candidate channel (`mpsc::Sender<PearlShareCandidate>`) to report found candidates back to the `MinerLoop`.

### Stats Integration
Real-time hashrate and candidate counts from GPU backends will be integrated into the `StatsManager` using atomic counters to ensure low overhead.

## Planned Backend Locations

- **CUDA**: `backends/cuda`
- **HIP**: `backends/hip`
- **OpenCL**: `backends/opencl`
- **SYCL/Metal**: `backends/sycl`, `backends/metal`

## Future Work
- Implementation of GPU-specific Pearl algorithm kernels.
- Optimized memory transfer between Host and Device.
- Auto-detection of available GPU hardware.
