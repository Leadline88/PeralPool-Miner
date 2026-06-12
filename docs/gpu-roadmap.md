# GPU Roadmap

Following the CPU reference implementation, we plan to add high-performance GPU backends.

## Planned Backends
1. **CUDA**: For NVIDIA hardware.
2. **HIP**: For AMD hardware.
3. **OpenCL/SYCL**: For cross-platform support.
4. **Metal**: For Apple Silicon.

## Architecture
The `MiningBackend` trait will be implemented for each GPU backend, allowing seamless switching.
