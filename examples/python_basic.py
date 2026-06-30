import json
from microloop import Microloop

yaml_config = """
max_repeats: 3
"""

engine = Microloop(yaml_config)
print("Microloop initialized. Sending identical tool calls...\n")

for i in range(1, 5):
    verdict = engine.verify("write_file", json.dumps({"path": "/tmp/test.txt", "content": "hello"}))
    label = "ALLOW" if verdict == 0 else "BLOCK"
    print(f"  Call {i}: write_file -> {label}")

print("\nThe 3rd identical call was blocked instantly - no API roundtrip needed.")
