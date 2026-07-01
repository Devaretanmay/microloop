from fastapi import FastAPI, Request
from fastapi.responses import JSONResponse
import uvicorn

app = FastAPI()

@app.post("/v1/chat/completions")
async def completions(request: Request):
    data = await request.json()
    
    # We will simulate an LLM returning a tool call, and then returning an error
    # To trigger the semantic loop, the agent will send identical or similar tools
    
    # The proxy will parse the "response" body.
    # Our mock will just return a static LLM error response for ANY request
    # Since we want to trigger a block, we will pretend the LLM's response has a tool call 
    # BUT wait, the LLM response itself needs to be a "tool_calls" block AND the error 
    # response needs to be visible. Wait, the proxy parses `response` for tool calls, 
    # but where does the LLM error response come from? 
    # In `intercept_tool_calls`, we did:
    # let llm_error_response = ... msg.get("content").as_str()
    # So if the upstream returns BOTH `tool_calls` AND `content` (which represents the error message from the previous step, or perhaps reasoning), we can use that.
    # Actually, in Microloop, the Proxy intercepts the upstream response, extracts tool_calls, 
    # and if the state blocks it, it injects an error. 
    # Wait, where does the `llm_error_response` come from? 
    # In `intercept_tool_calls`:
    # let llm_error_response = if let Some(content) = msg.get("content").and_then(|c| c.as_str()) { content.to_string() } ...
    
    # Let's extract what tool the user is trying to call from the *request*.
    # If the user sends a tool call in the request? No, the user sends a prompt, the LLM returns the tool call.
    # We'll have our agent send the requested tool via the "content" of the last message!
    last_msg = data.get("messages", [])[-1].get("content", "")
    requested_tool = "unknown"
    args = "{}"
    if "delete_line" in last_msg:
        requested_tool = "delete_line"
        args = '{"line": 5}'
    elif "remove_line" in last_msg:
        requested_tool = "remove_line"
        args = '{"line": 5}'
    elif "erase_line" in last_msg:
        requested_tool = "erase_line"
        args = '{"line": 5}'
    elif "clear_line" in last_msg:
        requested_tool = "clear_line"
        args = '{"line": 5}'

    return JSONResponse(content={
        "id": "chatcmpl-mock",
        "object": "chat.completion",
        "created": 1677652288,
        "model": "gpt-4",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "I encountered an error previously: 'Line 5 is already empty'. Let me try again.",
                "tool_calls": [{
                    "id": "call_mock",
                    "type": "function",
                    "function": {
                        "name": requested_tool,
                        "arguments": args
                    }
                }]
            },
            "finish_reason": "tool_calls"
        }]
    })

if __name__ == "__main__":
    uvicorn.run(app, host="127.0.0.1", port=8082)
