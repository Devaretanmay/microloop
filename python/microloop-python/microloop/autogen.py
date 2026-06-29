import json
from .microloop_core import Microloop

class MicroloopAutoGen:
    def __init__(self, config_yaml: str):
        self.engine = Microloop(config_yaml)
        
    def verify(self, tool_name: str, tool_args: dict) -> bool:
        """
        Verifies an AutoGen tool execution.
        Returns True if allowed, False if blocked.
        """
        args_json = json.dumps(tool_args)
        return self.engine.verify(tool_name, args_json) == 0
