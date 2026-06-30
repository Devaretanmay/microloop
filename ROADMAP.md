# Roadmap

## v0.1 (Current)

- Syntactic loop detection (sliding window hash)
- C ABI (`microloop_init`, `microloop_verify`, `microloop_free`)
- OpenAI / Anthropic reverse proxy
- PyO3 Python bindings
- `no_std` core, zero dependencies

## v0.2

**Volatile field auto-inference.** Microloop currently requires users to declare fields like timestamps or request IDs as volatile. In v0.2, the engine will detect fields that change on every call across consecutive identical invocations and automatically exclude them from the hash.

**Adaptive thresholding.** Currently `max_repeats` is a static config value. Adaptive thresholding adjusts the limit based on the agent's historical repeat rate — more permissive for complex multi-step tasks, tighter for simple operations.

**Loop feedback.** When a loop is detected but the hashes differ by a single high-entropy field, Microloop will surface a warning: "Loop detected, but hashes differ by `session_id`. Consider adding `session_id` to volatile fields."

## v0.3

**WebAssembly target.** Compile Microloop to WASM for browser-based agents and edge runtimes.

**Opt-in semantic comparison.** An optional out-of-process plugin (not in the core) that uses lightweight embeddings to detect semantically equivalent tool calls. This addresses the `delete_line(5)` vs `comment_out(5)` gap without compromising the core's latency or dependency profile.

**Universal gateway.** First-class support for OpenAI, Anthropic, Google, and local LLM providers through a unified proxy interface.

---

*Roadmap is subject to change based on community feedback. Open an issue if you'd like to see something prioritized.*
