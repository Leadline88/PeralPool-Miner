# GPU Readiness

Pearl Miner is designed with GPU mining in mind, although no GPU kernels are currently implemented.

## Architecture

The `MiningBackend` trait in `core/mining` provides the abstraction necessary to support multiple backends.

```rust
#[async_trait]
pub trait MiningBackend: Send + Sync {
    async fn start(&self) -> Result<(), String>;
    async fn stop(&self) -> Result<(), String>;
    async fn set_job(&self, job: &str) -> Result<(), String>;
    async fn get_hashrate(&self) -> f64;
}
```

## Planned Backends

Future developers should create new crates in the `backends/` directory for each supported technology:

- `backends/cuda`: For NVIDIA GPUs.
- `backends/hip`: For AMD GPUs.
- `backends/opencl`: For cross-vendor support.
- `backends/sycl`: For Intel and cross-platform support.
- `backends/metal`: For Apple Silicon.

## Implementation Steps

1.  Create the backend crate.
2.  Implement the `MiningBackend` trait.
3.  Integrate the new backend into `apps/miner-cli/src/main.rs`.
4.  Develop and optimize GPU kernels for the Pearl algorithm.
