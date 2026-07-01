# Roadmap

## v0.1 (Current)

- Syntactic loop detection (sliding window hash)
- C ABI (`microloop_init`, `microloop_verify`, `microloop_free`)
- OpenAI / Anthropic reverse proxy
- PyO3 Python bindings
- `no_std` core, zero dependencies

## v0.2 (Complete ✅)

**Volatile field auto-inference.** The engine now detects fields that change on every call across consecutive identical invocations and automatically excludes them from the hash. Includes validation across multiple prior calls to prevent false positives.

**Adaptive thresholding.** `max_repeats` is now dynamic based on error detection — reduces tolerance when errors are present to force faster pivoting.

**Pluggable blocklist backend.** Added Redis support for multi-instance deployments with in-memory fallback for single-instance usage.

## v0.3 (In Progress 🚧)

**WebAssembly target.** Compile Microloop to WASM for browser-based agents and edge runtimes. (Pre-existing in crates/microloop-wasm)

**Opt-in semantic comparison.** An optional out-of-process sidecar that uses lightweight embeddings to detect semantically equivalent tool calls. This addresses the `delete_line(5)` vs `comment_out(5)` gap without compromising the core's latency or dependency profile. Now uses synchronous communication to eliminate race conditions.

**Improved proxy architecture.** Enhanced reverse proxy with synchronous sidecar communication, better error handling, and production-ready Redis backend support.

---

*Roadmap is subject to change based on community feedback. Open an issue if you'd like to see something prioritized.*
