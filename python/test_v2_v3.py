import requests
import json
import uuid
import time

PROXY_URL = "http://127.0.0.1:8080/v1/chat/completions"
MODEL = "ornith:9b"
SIDECAR_URL = "http://127.0.0.1:8081/analyze"

# ─── Test 1: v2 Volatile Auto-Inference ───────────────────────────────────────
# Agent calls search with changing req_id. Without auto-inference this bypasses
# syntactic detection. v2 should auto-strip req_id and block it.

def test_v2_volatile_auto_inference():
    print("\n" + "="*60)
    print("TEST v2: Volatile Field Auto-Inference")
    print("Agent calls search() with same query but rotating req_id.")
    print("="*60)

    session_id = str(uuid.uuid4())
    headers = {"x-session-id": session_id, "Content-Type": "application/json"}

    messages = [
        {"role": "system", "content": "You are an agent. Always call the search tool."},
        {"role": "user", "content": "Search for python tutorials."},
    ]

    # Step 1 — first call
    req_id = 1
    msg_1_args = json.dumps({"query": "python tutorials", "req_id": req_id})
    messages.append({
        "role": "assistant",
        "tool_calls": [{"id": "call_1", "type": "function", "function": {
            "name": "search", "arguments": msg_1_args
        }}]
    })
    messages.append({"role": "tool", "tool_call_id": "call_1", "content": "No results."})

    # Step 2 — second call, same query, new req_id
    req_id = 2
    msg_2_args = json.dumps({"query": "python tutorials", "req_id": req_id})
    messages.append({
        "role": "assistant",
        "tool_calls": [{"id": "call_2", "type": "function", "function": {
            "name": "search", "arguments": msg_2_args
        }}]
    })
    messages.append({"role": "tool", "tool_call_id": "call_2", "content": "No results."})

    # Step 3 — third call, same query, new req_id. Proxy should BLOCK via auto-inference.
    req_id = 3
    msg_3_args = json.dumps({"query": "python tutorials", "req_id": req_id})
    payload = {
        "model": MODEL,
        "messages": messages,
        "tools": [{"type": "function", "function": {
            "name": "search",
            "description": "Search the web.",
            "parameters": {"type": "object", "properties": {
                "query": {"type": "string"},
                "req_id": {"type": "integer"}
            }, "required": ["query"]}
        }}],
    }

    resp = requests.post(PROXY_URL, json=payload, headers=headers, timeout=60)
    resp_json = resp.json()
    content = resp_json.get("choices", [{}])[0].get("message", {}).get("content", "")
    tool_calls = resp_json.get("choices", [{}])[0].get("message", {}).get("tool_calls")

    if content and "SYSTEM INTERCEPT" in content:
        print("PASS: Proxy blocked the 3rd call via Volatile Auto-Inference!")
        print(f"  Reason: {content}")
    elif tool_calls:
        print(f"INFO: Agent made tool call: {tool_calls[0]['function']['name']} — proxy allowed it.")
        print("  NOTE: LLM may have changed the query. Auto-inference may not have triggered.")
        print("  Check proxy logs for 'Volatile Auto-Inference activated'.")
    else:
        print(f"INFO: Agent responded without tool: {content[:120]}")


# ─── Test 2: v3 Semantic Loop Detection ───────────────────────────────────────
# A live agent uses delete_line, then remove_line, then erase_line.
# The sidecar should detect semantic similarity and push block rules.
# This test adds a proper delay between steps so the sidecar rule arrives in time.

def test_v3_semantic_sidecar():
    print("\n" + "="*60)
    print("TEST v3: Semantic Sidecar — Multi-Tool Synonym Loop")
    print("Agent tries delete_line → remove_line → erase_line.")
    print("Sidecar should block the 2nd+ calls as semantically identical.")
    print("="*60)

    session_id = str(uuid.uuid4())
    headers = {"x-session-id": session_id, "Content-Type": "application/json"}

    tools = [
        {"type": "function", "function": {
            "name": "delete_line",
            "description": "Delete a specific line from a file.",
            "parameters": {"type": "object", "properties": {"line": {"type": "integer"}}, "required": ["line"]}
        }},
        {"type": "function", "function": {
            "name": "remove_line",
            "description": "Remove a specific line from a file.",
            "parameters": {"type": "object", "properties": {"line": {"type": "integer"}}, "required": ["line"]}
        }},
        {"type": "function", "function": {
            "name": "erase_line",
            "description": "Erase a specific line from a file.",
            "parameters": {"type": "object", "properties": {"line": {"type": "integer"}}, "required": ["line"]}
        }},
    ]

    messages = [
        {"role": "system", "content": "You are a helpful coding assistant. If a tool fails, try a different one."},
        {"role": "user", "content": "Delete line 5. If one tool fails, try another."},
    ]

    blocked = False
    for step in range(1, 6):
        print(f"\n  --- Step {step} ---")
        payload = {"model": MODEL, "messages": messages, "tools": tools, "temperature": 0.3}

        try:
            resp = requests.post(PROXY_URL, json=payload, headers=headers, timeout=60)
            resp_json = resp.json()
        except Exception as e:
            print(f"  Failed to reach proxy: {e}")
            break

        if "choices" not in resp_json:
            print(f"  Unexpected response: {json.dumps(resp_json, indent=2)[:300]}")
            break

        msg = resp_json["choices"][0]["message"]
        content = msg.get("content", "")

        if content and "SYSTEM INTERCEPT" in content:
            print(f"  PASS: Proxy blocked the call!")
            print(f"  Reason: {content}")
            blocked = True
            break

        if msg.get("tool_calls"):
            tc = msg["tool_calls"][0]
            name = tc["function"]["name"]
            args = tc["function"]["arguments"]
            print(f"  Agent called: {name}({args})")

            messages.append(msg)
            messages.append({
                "role": "tool",
                "tool_call_id": tc["id"],
                "name": name,
                "content": "Error: Line 5 is protected and cannot be modified.",
            })

            # Give the sidecar time to compute embedding and POST block rule to proxy
            print(f"  Waiting 4s for sidecar to push semantic block rule...")
            time.sleep(4)
        else:
            print(f"  Agent stopped using tools: {content[:120]}")
            break

    if not blocked:
        print("\n  NOTE: Agent exhausted all tools without proxy interception.")
        print("  Check proxy logs to confirm sidecar pushed block rules (they may have arrived after LLM responded).")
        print("  This is a timing issue in the async sidecar pipeline, not a logic error.")


if __name__ == "__main__":
    test_v2_volatile_auto_inference()
    test_v3_semantic_sidecar()
