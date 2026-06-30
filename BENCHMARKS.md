# Benchmarks

Run `cargo run --release --bin microloop-bench` to reproduce these results on your hardware.

---

## Overhead per Check

| Metric | Value |
|---|---|
| Cold start (first verification) | ~520 ns |
| Average latency (warm, complex payload) | ~480 ns |
| P95 latency | ~620 ns |
| P99 latency | ~890 ns |

Microloop intercepts each tool call in under a microsecond. Compare this to the 1.5+ seconds required for an LLM roundtrip to detect the same loop.

---

## Throughput

| Payload | Checks/sec |
|---|---|
| Simple (`{"query": "..."}`) | ~2,400,000 |
| Complex (nested JSON, code blocks) | ~2,100,000 |

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
| Volatile fields excluded | Yes | Must be configured |

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
| Block on repeat count | ~150 ns |
| Block on rule violation | ~200 ns |

This is because the history comparator short-circuits as soon as the repeat threshold is reached.

---

## Memory

| Metric | Value |
|---|---|
| `MicroloopState` struct | ~180 bytes |
| History buffer (per tracked call) | ~200 bytes |
| Total footprint (typical workload) | < 10 MB |
