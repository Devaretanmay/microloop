# Fix lockfile references: convert old @earendil-works/pi-* package names
# to the new @microloop/* names. Run this to migrate legacy lockfiles.

import re

files = ["package-lock.json", "packages/coding-agent/install-lock/package-lock.json"]
replacements = [
    (r"@earendil-works/pi-coding-agent-install", r"@microloop/harness-install"),
    (r"@earendil-works/pi-coding-agent", r"@microloop/harness"),
    (r"@earendil-works/pi-agent-core", r"@microloop/agent"),
    (r"@earendil-works/pi-ai", r"@microloop/ai"),
    (r"@earendil-works/pi-tui", r"@microloop/tui"),
    (r"@earendil-works/pi-", r"@microloop/"),
]

for f in files:
    with open(f, 'r') as fp:
        content = fp.read()
    for old, new in replacements:
        content = re.sub(old, new, content)
    with open(f, 'w') as fp:
        fp.write(content)
print("Done")
