import subprocess
import json
import os

def run_cargo_check():
    res = subprocess.run(
        ["cargo", "check", "--message-format=json"],
        cwd="crates/microloop-app",
        capture_output=True,
        text=True
    )
    return res.stdout

def fix_errors():
    deps_added = set()
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
                
            msg_text = message["message"].lower()
            
            spans = message["spans"]
            if not spans: continue
            primary_span = next((s for s in spans if s["is_primary"]), spans[0])
            file_name = primary_span["file_name"]
            
            # extract the crate path from file_name (e.g. core/src/lib.rs -> core)
            parts = file_name.split("/")
            if len(parts) >= 2 and parts[-2] == "src":
                crate_dir = "/".join(parts[:-2])
            elif len(parts) >= 3 and parts[-3] == "src":
                crate_dir = "/".join(parts[:-3])
            elif len(parts) >= 4 and parts[-4] == "src":
                crate_dir = "/".join(parts[:-4])
            else:
                crate_dir = parts[0]
                
            cargo_toml_path = os.path.join("crates/microloop-app", crate_dir, "Cargo.toml")
            if not os.path.exists(cargo_toml_path):
                continue
                
            missing_dep = None
            if "codex_login" in msg_text: missing_dep = "codex-login"
            if "codex_analytics" in msg_text: missing_dep = "codex-analytics"
            if "codex_backend_client" in msg_text: missing_dep = "codex-backend-client"
            if "codex_cloud_config" in msg_text: missing_dep = "codex-cloud-config"
            
            if missing_dep:
                key = (cargo_toml_path, missing_dep)
                if key not in deps_added:
                    # Append it under [dependencies]
                    with open(cargo_toml_path, "r") as f:
                        content = f.read()
                        
                    if "[dependencies]" in content:
                        dep_line = f'{missing_dep} = {{ workspace = true }}\n'
                        if dep_line not in content:
                            content = content.replace("[dependencies]", "[dependencies]\n" + dep_line)
                            with open(cargo_toml_path, "w") as f:
                                f.write(content)
                            deps_added.add(key)
                            added_this_round += 1
                            print(f"Added {missing_dep} to {cargo_toml_path}")
        
        if added_this_round == 0:
            print("No more dependencies to add automatically.")
            break

if __name__ == "__main__":
    fix_errors()
