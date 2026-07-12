# Microloop for Python

**Rust-backed loop detection for AI agents. Blocks repetitive tool calls before they burn API credits.**

[![License: Apache 2.0](https://img.shields.io/badge/license-Apache%202.0-red.svg)](LICENSE)
[![PyPI](https://img.shields.io/pypi/v/microloop)](https://pypi.org/project/microloop/)
[![PyPI - Python Version](https://img.shields.io/pypi/pyversions/microloop)](https://pypi.org/project/microloop/)

<p align="center">
  <img src="../../assets/demo.gif" alt="Microloop Demo" width="720" style="max-width: 100%">
  <br>
  <em>See the full demo: <code>python3 scripts/demo_loop.py</code></em>
</p>

---

## The $500 Loop of Death

Every AI agent developer knows this pain:
You build an autonomous agent, run it overnight, and wake up to a **$500 OpenAI bill**.

What happened? The agent tried to write a file. The tool returned an error. The agent retried with the exact same arguments. It failed again. It spent the next 6 hours in a tight loop, burning millions of tokens, accomplishing nothing.

**Microloop stops this. It detects and blocks agent loops in under 200 nanoseconds.**

---

## Blind Counters vs. Microloop

Traditional frameworks rely on step counters (`max_iterations = 10`). Step counters are blind.

| The Blind Counter | The Microloop Shield |
| :--- | :--- |
| **Blocks valid workflows.** If a complex 15-step plan is working perfectly, a limit of 10 kills it. | **Unlocks infinite steps.** Your agent can run 100 steps if it's making progress. |
| **Allows expensive waste.** A 2-step loop will run 5 more times before hitting a `max_iterations = 10` limit. | **Fails fast.** Blocks the loop at step 2—the very first redundant repeat. |

*Counters measure quantity. Microloop measures redundancy.*

---

## Core Pillars

### 1. Zero-Latency Fast Path
Runs locally in-process. Hashes tool calls and checks for repetitive trajectories in **under 200 nanoseconds** using the native Rust core.

### 2. Smart Volatile Field Masking
Automatically detects high-entropy fields (like `req_id: "9831"` changing to `req_id: "9832"`) and masks them, catching loops even when arguments aren't 100% identical.

### 3. Adaptive Thresholding
If a tool call returns an error, the agent is in a high-risk state. Microloop automatically tightens its repetition thresholds on failure.

### 4. Horizontal Redis Sync
Plug in a Redis backend to share blocklists and loop states across a cluster of agent workers.

---

## Quick Start

### Installation

```bash
pip install microloop
```

### Basic SDK Usage

```python
from microloop.microloop_core import Microloop

engine = Microloop("max_repeats: 3")

result = engine.verify("write_file", '{"path": "/tmp/x.txt"}')
if result > 0:
    print("Action blocked! Forcing agent to pivot.")
```

### LiteLLM Integration

```python
import litellm
from microloop.integrations import MicroloopLiteLLMGuardrail

guardrail = MicroloopLiteLLMGuardrail(max_repeats=3)
litellm.callbacks = [guardrail]
```

---

## Performance Benchmarks

| Benchmark | Microloop | Prompt-Based Guardrails |
| :--- | :--- | :--- |
| **Check Latency** | **197 nanoseconds** | ~1.5 seconds (API call) |
| **Operational Cost** | **$0.00** (Local execution) | ~$0.01 per step (Token burn) |
| **Execution Safety** | **100% Deterministic** | Probabilistic |
| **Memory Footprint** | **< 10 MB** | N/A |
| **Throughput** | **5,000,000+ checks/sec** | ~50 checks/sec |

| Benchmark | Iterations | Per Op | Ops/s |
| :--- | ---: | ---: | ---: |
| `verify_e2e` | 100,000 | **197 ns** | **5,071,573** |
| `cold_start_verify` | 10,000 | **42,460 ns** | **23,551** |
| `compress_short_fastpath` (<512 bytes) | 200,000 | **50 ns** | **19,905,780** |
| `oscillation_detect` | 500,000 | 3,049 ns | 327,887 |
| `compress_git_diff` | 50,000 | 6,811 ns | 146,821 |
| `compress_build_output` | 50,000 | 15,393 ns | 64,961 |
| `compress_json_array` | 50,000 | 33,993 ns | 29,417 |
| `compress_source_code` | 50,000 | 164,475 ns | 6,080 |
| `mixed_workload` (verify + compress, 8 tools) | 10,000 | 1,094,099 ns | 914 |

---

## Configuration

```yaml
max_repeats: 3          # Max identical calls before blocking
history_window: 8       # Rolling window size
strictness: Balanced    # Lenient, Balanced, or Strict
tools:
  - name: execute_command
    trajectory_gate:
      volatile_fields: ["timestamp", "session_id"]
```

---

## Security

- No tool data or code ever leaves your environment.
- Zero external API calls.
- Apache 2.0 Licensed.
