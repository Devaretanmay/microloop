import json
import requests
import time
import uuid

PROXY_URL = "http://127.0.0.1:8080/v1/chat/completions"

def send_call(session_id, tool_name, args, error_msg):
    payload = {
        "model": "gpt-4",
        "messages": [
            {"role": "user", "content": "Fix the bug."}
        ],
        # Mocking the payload shape the proxy expects to intercept
        # Since we are just testing the interceptor logic, we send the "response" shape 
        # But wait, the proxy intercepts the UPSTREAM response. 
        # If we just hit the proxy, the proxy forwards to target API (OpenAI).
        # We need to test the sidecar analysis endpoint directly, OR we need a mock upstream.
    }
    pass

def test_sidecar_directly():
    session_id = str(uuid.uuid4())
    print(f"Testing Semantic Sidecar directly with session {session_id}")
    
    # 1. Send first failure
    requests.post("http://127.0.0.1:8081/analyze", json={
        "session_id": session_id,
        "tool_call": "delete_line(5)",
        "llm_error_response": "Line 5 is already empty"
    })
    
    # 2. Send second failure (same meaning, different syntax)
    requests.post("http://127.0.0.1:8081/analyze", json={
        "session_id": session_id,
        "tool_call": "remove_line(5)",
        "llm_error_response": "Line 5 is already empty"
    })
    
    # 3. Send third failure (same meaning)
    # This should trigger the semantic loop block rule injection!
    requests.post("http://127.0.0.1:8081/analyze", json={
        "session_id": session_id,
        "tool_call": "erase_line(5)",
        "llm_error_response": "Line 5 is already empty"
    })
    
    # Let the sidecar process and send the block rule to proxy
    time.sleep(2)
    
    
    # --- TEST 2: Valid Similarity (No Error) ---
    print("\nTesting Valid Similarity (should NOT block)...")
    valid_session = str(uuid.uuid4())
    
    requests.post("http://127.0.0.1:8081/analyze", json={
        "session_id": valid_session,
        "tool_call": "search(rust)",
        "llm_error_response": "" # Success!
    })
    
    requests.post("http://127.0.0.1:8081/analyze", json={
        "session_id": valid_session,
        "tool_call": "search(rust programming)",
        "llm_error_response": "" # Success!
    })
    
    requests.post("http://127.0.0.1:8081/analyze", json={
        "session_id": valid_session,
        "tool_call": "search(rust rustlang)",
        "llm_error_response": "" # Success!
    })
    
    time.sleep(2)
    print("Test payloads sent! Check the proxy/sidecar logs.")

if __name__ == "__main__":
    test_sidecar_directly()
