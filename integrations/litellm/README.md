# Microloop LiteLLM Guardrail

Drop-in LiteLLM guardrail that detects deterministic tool call loops.
Blocks identical tool+argument trajectories BEFORE they reach the LLM.

## Quick Start

```python
import litellm
from integrations.litellm.microloop_guardrail import MicroloopGuardrail

litellm.callbacks = [MicroloopGuardrail(max_repeats=3)]
```

## Features

- Deterministic detection (hash-based, sub-microsecond)
- Volatile field auto-inference
- Session isolation
- Adaptive thresholding
- OpenAI + Anthropic tool call formats

See `microloop_guardrail.py` for full API docs.
