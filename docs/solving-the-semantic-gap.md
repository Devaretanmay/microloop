# Solving the Semantic Gap: The Fast Path / Slow Path Architecture

Microloop's core is designed around syntactic loop detection (exact argument hashing) to achieve sub-microsecond latency. However, syntactic detection has a known limitation: it cannot catch *semantic loops*.

A semantic loop occurs when an agent takes two technically different actions that produce the same result, such as calling `delete_line(5)` and then `comment_out(5)`. Both fail, but because the syntax differs, a purely syntactic hash will not catch it.

Solving this requires embeddings and semantic comparison. But embedding models are large, and executing them takes milliseconds, which violates Microloop's <10MB and 480ns core constraints.

The solution is the **Fast Path / Slow Path** architecture, effectively acting as a Control Plane and Data Plane for AI safety guardrails.

## The Architecture: Control Plane and Data Plane

This design mirrors high-performance trading systems and network firewalls: keep the critical path as fast as possible, and use an asynchronous background process to update the rules.

### The Fast Path (Data Plane)
The Rust core continues to execute exact-match syntactic hashing. It remains a zero-dependency library operating in ~480 nanoseconds. If it catches a known syntactic loop, it blocks the LLM instantly.

### The Slow Path (Control Plane)
Asynchronously, the Microloop proxy forwards the agent's tool calls and the system's responses to a secondary, local sidecar. This sidecar runs a lightweight embedding model (such as `all-MiniLM-L6-v2`) in Python or Rust.

### The Feedback Loop (JIT Rule Compilation)
If the sidecar detects that 3 consecutive calls have different syntax but highly similar semantic embeddings (e.g., `delete_line` vs `comment_out`), it dynamically generates a new syntactic rule and pushes it to the core's state. 

The sidecar acts as a "JIT Compiler" for loop rules.

## Why This Works

1. **Zero Critical-Path Penalty:** The LLM's blocking step remains strictly in the 480ns Data Plane. The agent is never forced to wait for an embedding model to run before it can proceed.
2. **Eventual Consistency:** Because the Control Plane runs asynchronously, it might be slightly "late". The agent might make two semantic loop calls before the sidecar finishes computing the embeddings. But by the third or fourth call, the sidecar has pushed a new syntactic block-rule down to the core, and that next call is blocked instantly at 480ns. Letting one or two redundant calls slip through is an acceptable tradeoff for keeping the critical path latency effectively zero.
3. **True Opt-In:** Because the semantic engine is a completely separate process (a sidecar container), users who only want syntactic detection don't need to install PyTorch or download model weights. The core remains a lightweight binary.
