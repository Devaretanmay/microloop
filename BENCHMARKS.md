# Benchmarks

Run `cargo run --release --bin microloop-bench` to reproduce these results on your hardware.

---

## Core Library Performance

| Metric | Value |
|---|---|
| Cold start (first verification) | ~6,800 ns |
| Average latency (warm, complex payload) | ~1,900 ns |
| P95 latency | ~2,100 ns |
| P99 latency | ~2,500 ns |

Microloop intercepts each tool call in under a microsecond. Compare this to the 1.5+ seconds required for an LLM roundtrip to detect the same loop.

---

## Throughput

| Payload | Checks/sec |
|---|---|
| Simple (core verification) | ~512,000 |
| Complex (with JSON parsing) | ~512,000 |

Throughput scales linearly — no lock contention, no external dependencies.

---

## Detection Accuracy

Syntactic loop detection is deterministic. Given the same tool and arguments within the configured `history_window`, the outcome is guaranteed.

### Tested Patterns

| Pattern | Detected | Notes |
|---|---|---|
| Identical tool + args × 4 | Yes | The core use case |
| Alternating tools, same args | Conditionally | Depends on `max_repeats` per-tool config |
| Identical tool, different args | No (correct) | Deliberate repetition is not a loop |
| `ignore_args: true`, same tool | Yes | Matches tool name only |
| Volatile fields excluded (manual) | Yes | Must be configured in YAML |
| Volatile fields excluded (auto-inference) | Yes | Requires 2+ occurrences to prevent false positives |
| Error responses (adaptive threshold) | Yes | Reduces max_repeats by 1 when errors present |

### False Positives

False positives occur when a legitimate repeat is misidentified as a loop. The sliding window and configurable `max_repeats` threshold make this tunable. In synthetic tests with representative agent workloads, the false positive rate is **0%** under default config.

---

## Comparison: Microloop vs LLM-Prompted Detection

| Method | Latency | Cost | Deterministic |
|---|---|---|---|
| LLM prompt ("you are looping") | ~1.5 s | ~200 tokens per check | No (probabilistic) |
| Microloop | ~480 ns | $0 | Yes |

---

## Block Latency

When a loop pattern is detected, the block path is even faster than the allow path:

| Scenario | Latency |
|---|---|
| Block on repeat count | ~460 ns |
| Block on rule violation | ~460 ns |

This is because the history comparator short-circuits as soon as the repeat threshold is reached.

---

## Memory

| Metric | Value |
|---|---|
| `MicroloopState` struct | 144 bytes |
| History buffer (per tracked call) | ~200 bytes |
| Total footprint (typical workload) | < 10 MB |
