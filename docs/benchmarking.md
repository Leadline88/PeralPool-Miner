# Benchmarking

Pearl Miner provides a built-in benchmarking tool to measure the performance of the native CPU miner.

## Running the Benchmark

```bash
cargo run --release --bin miner-cli -- --benchmark-native-cpu
```

## Metrics Collected

- **Job Parsing Time**: Average time to parse a Stratum job JSON.
- **Work Package Creation Time**: Average time to create a mining work package from a job.
- **Share Verification Time**: Average time to verify a single share candidate.
- **Nonce Search Throughput**: The total number of hashes per second (H/s) achieved on the current hardware.

## JSON Report Schema

The benchmark results are saved to `benchmark_report.json` with the following schema:

```json
{
  "avg_job_parse_secs": 3.3312889999999997e-6,
  "avg_share_verify_secs": 9.200132699999999e-6,
  "avg_work_package_creation_secs": 7.108325e-6,
  "backend": "native-cpu",
  "duration_secs": 5.0,
  "hashrate_single": 107272.14,
  "hashrate_multi": 412345.67,
  "threads": 4,
  "timestamp": "2026-06-12T10:44:57.657359194+00:00",
  "unit": "H/s"
}
```

- `avg_job_parse_secs`: average time in seconds for parsing a job.
- `avg_share_verify_secs`: average time in seconds for verifying a share.
- `avg_work_package_creation_secs`: average time in seconds for creating a work package.
- `backend`: the backend used (e.g., `native-cpu`).
- `duration_secs`: duration of the throughput benchmark in seconds.
- `hashrate_single`: hashes per second (single-threaded).
- `hashrate_multi`: hashes per second (multi-threaded).
- `threads`: number of threads used for multi-threaded test.
- `timestamp`: RFC3339 timestamp of the benchmark run.
- `unit`: the unit of hashrate (`H/s`).
