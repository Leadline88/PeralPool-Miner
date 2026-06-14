# Future Work Roadmap

This roadmap outlines the key tasks remaining to bring Pearl Miner to production readiness for native mining.

## 1. Real Pearl Algorithm Integration

- [ ] Replace SHA256 synthetic implementation with the official Pearl algorithm in `algos/pearl`.
- [ ] Verify algorithm correctness against mainnet fixtures and test vectors.
- [ ] Optimize algorithm performance using SIMD (AVX2, AVX-512, NEON).

## 2. PearlPool Protocol Verification

- [ ] **PearlPool fixture capture**: Capture and document real `mining.notify` and `mining.set_difficulty` messages from PearlPool.
- [ ] **Real mining.notify parsing**: Implement robust parsing for all PearlPool-specific job fields.
- [ ] **Real mining.submit verification**: Verify the exact format for `mining.submit` (params order, hex vs decimal, etc.) and enable it in `PearlPoolAdapter`.

## 3. Developer-Fee Completion

- [ ] **Active developer-fee identity switching**: Implement active wallet switching (re-authorization) on the Stratum connection.
- [ ] Ensure seamless transition between user and developer mining cycles without connection drops.
- [ ] Add verification tests for fee collection transparency and accuracy.

## 4. GPU Backend Implementation

- [ ] **CUDA**: Implement high-performance kernel for NVIDIA GPUs.
- [ ] **HIP**: Support for AMD GPUs via ROCm/HIP.
- [ ] **OpenCL/SYCL**: Broad cross-platform GPU support.
- [ ] **Metal**: Optimized backend for Apple Silicon.

## 5. User Interface and Experience

- [ ] Develop a graphical user interface (GUI) or a web-based dashboard.
- [ ] Create easy-to-use installers and release packaging for Windows, Linux, and macOS.

## 6. Performance and Stability

- [ ] Conduct long-term stability tests on various hardware configurations.
- [ ] Implement auto-tuning for thread counts and GPU parameters.
- [ ] Enhance error recovery for network and hardware failures.
