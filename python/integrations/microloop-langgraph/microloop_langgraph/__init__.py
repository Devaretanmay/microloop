import json
from typing import Callable, Any, Dict
from microloop.microloop_core import Microloop

class MicroloopMiddleware:
    def __init__(self, config: str = "{}"):
        self.engine = Microloop(config)

    def verify_tool(self, tool_name: str, args: Dict[str, Any]) -> None:
        args_json = json.dumps(args)
        verdict = self.engine.verify(tool_name, args_json)
        if verdict != 0:
            raise Exception("LoopDetected: Trajectory redundant")

    def wrap_node(self, func: Callable) -> Callable:
        def wrapper(state: dict):
            if "tool_calls" in state:
                for call in state["tool_calls"]:
                    self.verify_tool(call.get("name", ""), call.get("args", {}))
            return func(state)
        return wrapper
