# Microloop + LiteLLM Integration

Sub-microsecond, Rust-backed loop detection for any LiteLLM proxy or SDK usage.

## Installation

```bash
pip install "microloop[litellm]"
```

## Usage

### Python SDK

```python
import litellm
from microloop.integrations import MicroloopLiteLLMGuardrail

guardrail = MicroloopLiteLLMGuardrail(max_repeats=3)
litellm.callbacks = [guardrail]

response = litellm.completion(
    model="gpt-4",
    messages=[{"role": "user", "content": "Search for python tutorials"}],
    tools=[...],
)
```

### LiteLLM Proxy — `config.yaml`

```yaml
model_list:
  - model_name: gpt-4
    litellm_params:
      model: openai/gpt-4
      api_key: os.environ/OPENAI_API_KEY

litellm_settings:
  callbacks:
    - "microloop.integrations.MicroloopLiteLLMGuardrail"
```

## Configuration

| Parameter | Default | Description |
|---|---|---|
| `max_repeats` | `3` | Max identical calls before blocking |
| `history_window` | `10` | How many past calls to examine |
| `volatile_fields` | `[]` | JSON keys to ignore when comparing (e.g. `["req_id"]`) |

## Example

```python
from microloop.integrations import MicroloopLiteLLMGuardrail

guardrail = MicroloopLiteLLMGuardrail(
    max_repeats=3,
    history_window=10,
    volatile_fields=["req_id", "timestamp"],
)

# Each session gets its own engine automatically
litellm.callbacks = [guardrail]
```
