# Why your AI agent is stuck in a loop (and how to fix it in 480 nanoseconds)

Autonomous AI agents are expensive. Not because of the hardware, but because of the loops.

An agent tasked with fixing a bug will try `write_file`, get an error, try the exact same `write_file` again, get the same error, and repeat 50 times before someone manually kills the process. Each iteration burns ~$0.01 in API costs and takes 1-3 seconds. A 50-iteration loop costs $0.50 and takes over a minute. An unsupervised overnight run can cost hundreds.

The industry answer so far has been `max_iterations`.

## Why max_iterations is a budget, not a fix

Every agent framework ships with a step counter. LangChain has `max_iterations`. AutoGen has `max_consecutive_auto_reply`. OpenAI has `max_tokens`.

These are all the same thing: an arbitrary limit on how many steps the agent can take.

The problem is that step counters are blind. A valid 15-step refactoring task gets killed at step 11. A 2-step loop (`read_file` -> error -> `read_file` again) burns 8 more expensive API calls before the counter finally trips. The counter doesn't distinguish between progress and failure — it just counts.

What agents need is not a budget counter. They need a redundancy detector.

## Redundancy detection in 480 nanoseconds

Microloop is a small Rust library that sits between the agent and the LLM. Before each tool call leaves the machine, Microloop hashes the tool name and arguments against a sliding window of recent calls.

Three identical `write_file` calls produce three identical hashes. The fourth is blocked instantly — not after a 1.5-second LLM roundtrip, but in under a microsecond. The agent receives the block response and is forced to pivot to a different approach.

The core design principles:

- **Deterministic, not probabilistic.** An LLM might realize it's looping, or it might not. Microloop uses exact hash comparison — if the trajectory is identical, it's blocked, 100% of the time.
- **Local, not remote.** No API call, no external service. The check happens in the same process, in nanoseconds.
- **Framework-agnostic.** The core is `no_std` Rust with a C ABI. It links into Python, Go, or Node.js just as easily as Rust. There's also a proxy that works with any OpenAI-compatible client without code changes.

## The tradeoff: syntactic vs semantic

Microloop compares exact tool arguments. This means it catches syntactic loops — the same tool, same arguments, same result. It does not catch semantic loops — two different actions that produce the same failure, like `delete_line(5)` and `comment_out(5)` returning the same error.

This is a deliberate tradeoff. Semantic comparison requires embeddings, which pushes latency from 480 nanoseconds to 10+ milliseconds and adds a 100+ MB model dependency. Microloop is designed to be a zero-dependency, sub-microsecond guardrail. Semantic detection is a future feature as an opt-in, out-of-process plugin — not something that compromises the core.

## The benchmarks

| Metric | Microloop | LLM prompt |
|---|---|---|
| Latency per check | 480 ns | 1.5 s |
| Cost per check | $0 | ~200 tokens |
| Deterministic | Yes | No |
| Memory | < 10 MB | N/A |

At 2 million checks per second per thread, Microloop adds no measurable overhead.

## How it works in 30 lines

```rust
let mut state = microloop::state::MicroloopState::new(yaml).unwrap();

for i in 1..=4 {
    let verdict = microloop::verify(&mut state,
        b"write_file",
        b"{\"path\": \"/tmp/test.txt\", \"content\": \"hello\"}");
    println!("Call {}: {}", i,
        if verdict == 0 { "ALLOW" } else { "BLOCK" });
}
```

Output:

```
Call 1: ALLOW
Call 2: ALLOW
Call 3: BLOCK
Call 4: BLOCK
```

The third identical call is blocked. No API roundtrip. No wasted tokens. The agent gets an instant signal to try something different.

## The config surface

Microloop exposes four knobs:

- `max_repeats` — How many identical calls before blocking (default: 3).
- `ignore_args` — Match on tool name only, ignoring argument differences.
- `history_window` — How far back to look for repeats.
- `volatile_fields` — JSON fields to exclude from the hash (timestamps, request IDs).

That's it. No Prometheus metrics, no Redis-backed state, no Kubernetes sidecar YAML. It does one thing.

## What's next

The current version (v0.1) covers the core loop detection use case. The roadmap includes volatile field auto-inference (the engine detects fields that change on every call and excludes them automatically), adaptive thresholding, and an opt-in semantic comparison plugin for v0.3.

## Try it

```bash
cargo add microloop
cargo run --example basic
```

10 seconds, and you've seen exactly what it does.

---

*Microloop is MIT-licensed. Contributions, issues, and feedback are welcome.*
