import subprocess
import json
import os
import re

def run_cargo_check():
    res = subprocess.run(
        ["cargo", "check", "--message-format=json"],
        cwd="crates/microloop-app",
        capture_output=True,
        text=True
    )
    return res.stdout

def fix_errors():
    stubbed = set()
    while True:
        print("Running cargo check...")
        stdout = run_cargo_check()
        added_this_round = 0
        for line in stdout.splitlines():
            if not line: continue
            try:
                msg = json.loads(line)
            except:
                continue
            
            if msg.get("reason") != "compiler-message":
                continue
            
            message = msg["message"]
            if message["level"] != "error":
                continue
                
            msg_text = message["message"]
            
            # Match missing type: "cannot find struct, variant or union type `AppInvocation` in module `codex_analytics`"
            # Or "cannot find type `AppInvocation` in module `codex_analytics`"
            # Or "cannot find value `build_track_events_context` in module `codex_analytics`"
            # Or "cannot find function `build_track_events_context` in module `codex_analytics`"
            m = re.search(r"cannot find .*? `(\w+)` in module `(codex_analytics|codex_login|codex_backend_client|codex_cloud_config)`", msg_text)
            if not m:
                # sometimes it says "unresolved import `codex_analytics::AppInvocation`"
                m = re.search(r"unresolved import `(codex_analytics|codex_login|codex_backend_client|codex_cloud_config)::(\w+)`", msg_text)
                if m:
                    crate = m.group(1)
                    item = m.group(2)
                else:
                    continue
            else:
                item = m.group(1)
                crate = m.group(2)
                
            key = (crate, item)
            if key not in stubbed:
                folder = crate.replace("codex_", "").replace("_", "-")
                lib_path = os.path.join("crates/microloop-app", folder, "src/lib.rs")
                
                # Check if it's likely a function (starts with lowercase) or struct (starts with uppercase)
                if item[0].islower():
                    # It's a function or variable. Let's make a generic macro or function.
                    # A macro is safer because it can take any args, but we can't easily export a macro with the exact name without `macro_rules!`.
                    # Let's just do a function that takes `...args`? Rust doesn't support that.
                    # But we can define a macro and alias it? No.
                    # Wait, if we define it as a function, the number of args might mismatch!
                    # What if we just comment out the line in the target file? No, we are building stubs!
                    stub = f"\npub fn {item}<T>() -> Option<T> {{ None }}\n"
                else:
                    # It's a struct/enum/trait
                    stub = f"\n#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]\npub struct {item} {{}}\n"
                    stub += f"impl {item} {{\n    pub fn new() -> Self {{ Self {{}} }}\n}}\n"
                    
                with open(lib_path, "a") as f:
                    f.write(stub)
                stubbed.add(key)
                added_this_round += 1
                print(f"Added {item} to {crate}")
                
        if added_this_round == 0:
            print("No more stubs to add automatically.")
            break

if __name__ == "__main__":
    fix_errors()
