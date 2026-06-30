# Microloop

> A deterministic guardrail against runaway tool costs in AI agents.

![License](https://img.shields.io/badge/license-MIT-blue.svg)

AI agents get stuck in infinite tool loops. Each loop cycle burns API credits. Microloop detects redundant tool calls locally in **nanoseconds** and blocks them before the call ever leaves the machine.

![Demo](assets/demo.gif)

---

## The Counter-Argument

**"Max iterations" is a budget, not a fix.**

| Approach | What happens |
|---|---|
| `max_iterations=10` | Kills a valid 15-step workflow at step 11. Lets a 2-step loop burn 8 more expensive API calls. |
| Microloop | Lets the valid workflow finish. Kills the loop at step 2 — the first repeat. |

Microloop is a **redundancy detector**, not a step counter. It gives agents infinite steps as long as they're making progress.

---

## Quick Start

```bash
cargo add microloop
cargo run --example basic
```

That's it. You'll see:

```
  Call 1: write_file → ALLOW
  Call 2: write_file → ALLOW
  Call 3: write_file → BLOCK
```

---

## Benchmarks

| Metric | Value |
|---|---|
| Overhead per check | ~480 ns |
| Memory footprint | < 10 MB |
| Throughput | > 2,000,000 checks/sec |
| Block latency | ~150 ns (loop pattern) |

Comparatively, detecting a loop via LLM prompt ("you are looping, please stop") takes **1.5+ seconds** and burns tokens. Microloop intercepts locally before the API call.

[Full benchmark results →](BENCHMARKS.md)

---

## Installation

### Rust

```toml
[dependencies]
microloop = "0.1.1"
```

### Python

```bash
pip install microloop
```

```python
from microloop import Microloop
engine = Microloop(config_yaml)
result = engine.verify("tool_name", '{"arg": "value"}')
```

### C / C++ / Go

Link against `libmicroloop.so` and include `microloop.h`. See the [C API reference](#c-api-reference).

---

## Architecture

```mermaid
sequenceDiagram
    participant Agent as Autonomous Agent
    participant Microloop as Microloop Core
    participant LLM as LLM Provider

    Agent->>Microloop: Step 1: Tool Execution
    Microloop->>Microloop: Hash Trajectory State
    Microloop-->>Agent: Proceed (Unique state)
    Agent->>LLM: Generate next step

    Agent->>Microloop: Step 2: Identical Tool Execution
    Microloop->>Microloop: Hash Trajectory State
    Microloop-->>Agent: BLOCK (Loop Detected)
    Note over Agent: Agent is forced to pivot
```

Microloop sits between the agent and the LLM. Before each tool call leaves the machine, Microloop hashes the tool name and arguments against a sliding window of recent calls. Redundant trajectories are blocked instantly; unique ones pass through at full speed.

The core is `no_std` Rust with a C ABI. No Python runtime, no container, no external service — it links directly into your application.

---

## Known Limits

**Syntactic, not semantic.** Microloop compares exact tool arguments. Two distinct actions that produce the same failure — `delete_line(5)` vs `comment_out(5)` — produce different hashes and won't be detected as a loop. Semantic comparison would require embeddings, pushing latency from 480 ns to 10+ ms and adding a 100+ MB model dependency. We chose speed and determinism for v0.1.

**Volatile fields are manual.** Fields like timestamps or request IDs that change on every call must be declared in config. Auto-inference is planned for v0.2.

**Proxy mode adds one moving part.** The reverse proxy is the simplest integration path for OpenAI-compatible agents. For direct integration, use the Rust crate or Python bindings instead.

---

## Roadmap

| Version | Focus |
|---|---|
| v0.1 | Syntactic loop detection, C ABI, proxy, PyO3 bindings |
| v0.2 | Volatile field auto-inference, adaptive thresholding |
| v0.3 | WASM target, opt-in semantic comparison (out-of-process) |

[Full roadmap →](ROADMAP.md)

---

## FAQ

**Does Microloop block valid repetitive tasks?**

No. Deliberate repetition (processing an array row-by-row) generates distinct state for each call. Microloop only blocks trajectories where the tool and arguments are identical within a configurable window. If your agent is doing the same thing and getting the same result, it's looping — and Microloop catches it.

**How is this different from `max_iterations`?**

A counter is blind. It kills the 15-step refactor at step 11 and lets the 2-step loop burn 8 more calls. Microloop detects *actual redundancy*, not step count. If the agent is making progress, Microloop never fires.

**Can I use this with any agent?**

Yes. Microloop is framework-agnostic — use it via the Rust crate, Python bindings, or the proxy. Works with LangChain, AutoGen, CrewAI, OpenAI, Anthropic, and any tool-calling agent.

**What about streaming?**

The proxy passes non-tool calls and streaming responses through transparently with zero parsing overhead.

---

## C API Reference

```c
// Initialize with YAML config
void* microloop_init(const char* yaml_str, size_t yaml_len);

// Verify a tool call. Returns 0 (allow) or non-zero (block).
uint8_t microloop_verify(void* state, const char* tool, size_t tool_len,
                         const char* args, size_t args_len);

// Get the last error message
const char* microloop_get_last_error(void* state);

// Free state
void microloop_free(void* state);
```

---

## Security

If you discover a security vulnerability, please do NOT file a public issue. Refer to our [Security Policy](SECURITY.md) and email the maintainers directly.

---

## License

MIT
