import requests
import json
import time
import subprocess

def test_auto_inference():
    print("Starting Proxy...")
    proxy_proc = subprocess.Popen(["cargo", "run", "-p", "microloop-proxy"])
    time.sleep(3) # Wait for proxy to start

    try:
        url = "http://127.0.0.1:8080/v1/chat/completions"
        headers = {
            "Content-Type": "application/json",
            "x-session-id": "test_session",
            "Authorization": "Bearer test" # Need auth to hit upstream, but proxy will block before upstream
        }
        
        # We craft a history where the agent called search 3 times.
        # The query is the same, but the 'req_id' keeps changing.
        # This simulates a high-entropy field that evades normal syntactic detection.
        payload = {
            "model": "gpt-4",
            "messages": [
                {
                    "role": "assistant",
                    "tool_calls": [
                        {"id": "call_1", "type": "function", "function": {"name": "search", "arguments": '{"query": "test", "req_id": "1"}'}}
                    ]
                },
                {
                    "role": "tool",
                    "tool_call_id": "call_1",
                    "content": "Result 1"
                },
                {
                    "role": "assistant",
                    "tool_calls": [
                        {"id": "call_2", "type": "function", "function": {"name": "search", "arguments": '{"query": "test", "req_id": "2"}'}}
                    ]
                },
                {
                    "role": "tool",
                    "tool_call_id": "call_2",
                    "content": "Result 2"
                }
            ],
            # Now the agent is making the 3rd call
            "tools": [{"type": "function", "function": {"name": "search"}}],
        }

        # Wait, the proxy parses the tool calls from the *response* to block it!
        # The proxy is a reverse proxy. It intercepts the LLM's response, not the request!
        # So we can't easily unit test the proxy's interception logic just by sending a request,
        # because the request goes to the UPSTREAM, and the UPSTREAM returns the tool call.
        # We need to mock the upstream.
        pass
    finally:
        proxy_proc.terminate()

if __name__ == "__main__":
    print("Ah, proxy intercepts responses. Let's use a unit test inside proxy.rs instead.")
