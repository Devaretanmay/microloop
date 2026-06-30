"""
Microloop + OpenAI proxy example.

Prerequisites:
  1. Start the proxy: cargo run --release --bin microloop-proxy
  2. Install: pip install openai

The proxy sits between your agent and the LLM API. It intercepts tool calls,
checks them against Microloop's loop detector, and blocks redundant
trajectories before they reach the LLM.

Non-tool calls and streaming responses pass through transparently
with zero parsing overhead.
"""

import os
from openai import OpenAI

# Point the client at the local proxy instead of the OpenAI API.
# The proxy handles authentication forwarding.
client = OpenAI(
    base_url="http://localhost:8080/v1",
    api_key=os.environ.get("OPENAI_API_KEY", "sk-...")
)

response = client.chat.completions.create(
    model="gpt-4o",
    messages=[{"role": "user", "content": "Write hello world to a file."}],
    tools=[{
        "type": "function",
        "function": {
            "name": "write_file",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {"type": "string"},
                    "content": {"type": "string"}
                }
            }
        }
    }]
)

print(response.choices[0].message)
