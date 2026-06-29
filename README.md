# Microloop

Microloop is a lightweight, ultra-fast runtime safety layer for autonomous agents. It detects and blocks tool-call loops before they burn tokens or crash your infrastructure.

This repository contains the `no_std` Rust core library, C ABI FFI bindings, and Python adapters (LangChain, CrewAI, AutoGen, LangGraph).

## Features
- **Ultra-fast**: Executes in ~17 microseconds. Fast-rejects loops in 375 nanoseconds.
- **Dependency-free**: Pure `no_std` Rust core.
- **Universal**: Plugs into any language or harness via C FFI and adapters.

## Real Benchmarks
Microloop is built to add zero noticeable overhead to your autonomous agents.

We ran 100,000 iterations of complex, nested JSON LLM tool payloads on a single thread to capture real-world performance:

| Metric | Result |
|--------|--------|
| **Memory Footprint (Base)** | `128 bytes` |
| **Max Throughput** | `57,947 verifications/sec` |
| **Cold Start Latency** | `7.41 µs` |
| **Average Latency (Warm)** | `17.21 µs` |
| **P99 Latency** | `25.12 µs` |
| **Adversarial Loop Fast-Reject** | `375 ns` (0.37 µs) |

### Python PyO3 Adapter Benchmarks
Even when called from Python, Microloop uses PyO3 to bypass slow FFI and maintain native Rust speeds. Here is the overhead when integrated into a Python agent:

| Metric | Result |
|--------|--------|
| **Max Throughput** | `13,603 verifications/sec` |
| **Cold Start Latency** | `107 µs` |
| **Average Latency (Warm)** | `73.39 µs` |
| **P99 Latency** | `90.50 µs` |
| **Adversarial Loop Fast-Reject** | `5.5 µs` |

*Note: The ~50µs overhead difference between Native Rust and Python is due entirely to Python's `json.dumps()` serialization.*

*Benchmarks were executed using `cargo run --release --bin microloop-bench`*

## Building
```bash
cargo build --release
```
