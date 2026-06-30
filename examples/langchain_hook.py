"""
Microloop + LangChain integration example.

This shows how to wrap a LangChain agent's tool execution with
Microloop's loop detector. If a loop is detected, the tool result
is intercepted and the agent is forced to pivot.

Prerequisites:
  pip install langchain langchain-openai microloop
"""

import json
from langchain.agents import AgentExecutor, create_openai_tools_agent
from langchain.tools import tool
from langchain_openai import ChatOpenAI
from microloop import Microloop

guard = Microloop("""
max_repeats: 3
""")


@tool
def write_file(path: str, content: str) -> str:
    """Write content to a file."""
    # Check with Microloop before executing
    args = json.dumps({"path": path, "content": content})
    if guard.verify("write_file", args) != 0:
        return "SYSTEM INTERCEPT: Loop detected. Try a different approach."

    # Proceed with the actual file write
    with open(path, "w") as f:
        f.write(content)
    return f"Written to {path}"


tools = [write_file]
llm = ChatOpenAI(model="gpt-4o")
agent = create_openai_tools_agent(llm, tools, ...)
executor = AgentExecutor(agent=agent, tools=tools, verbose=True)
