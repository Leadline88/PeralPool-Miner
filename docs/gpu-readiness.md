# GPU Readiness

Pearl Miner is designed with future GPU support in mind, but **no GPU code is currently implemented.**

## Design Foundation

The following traits in `core/mining` are ready for GPU backend implementation:

- `MiningBackend`: Interface for starting/stopping and job management.
- `MiningAlgorithm`: Interface for job parsing and share verification.

## Planned Backends

- **CUDA**: `backends/cuda`
- **HIP**: `backends/hip`
- **OpenCL**: `backends/opencl`
- **SYCL/Metal**: `backends/sycl`, `backends/metal`

## Constraints

- No GPU kernels (CUDA, HIP, OpenCL, SYCL, Metal) are currently in the codebase.
- Future implementations must ensure zero overhead for users not using GPU backends.
- All GPU work must be transparently reported in the runtime stats.
