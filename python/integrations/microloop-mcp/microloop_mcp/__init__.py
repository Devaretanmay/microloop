import sys
import json
from microloop.microloop_core import Microloop

class MCPProxy:
    def __init__(self, target_command: list[str]):
        self.target = target_command
        self.engine = Microloop("{}")

    def run(self):
        import subprocess
        proc = subprocess.Popen(self.target, stdin=subprocess.PIPE, stdout=subprocess.PIPE)
        
        while True:
            line = sys.stdin.readline()
            if not line:
                break
                
            try:
                payload = json.loads(line)
                if payload.get("method") == "tools/call":
                    tool = payload["params"]["name"]
                    args = payload["params"].get("arguments", {})
                    if self.engine.verify(tool, json.dumps(args)) != 0:
                        sys.stdout.write(json.dumps({
                            "jsonrpc": "2.0",
                            "id": payload.get("id"),
                            "error": {"code": -32000, "message": "LoopDetected"}
                        }) + "\n")
                        sys.stdout.flush()
                        continue
            except Exception:
                pass
                
            proc.stdin.write(line.encode())
            proc.stdin.flush()
            out = proc.stdout.readline()
            sys.stdout.write(out.decode())
            sys.stdout.flush()
