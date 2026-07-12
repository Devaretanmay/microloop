import os
import sys
import subprocess

def run_bundled(args):
    """Run the bundled microloop Node.js standalone executable."""
    base_dir = os.path.dirname(os.path.abspath(__file__))
    binary_path = os.path.join(base_dir, "bin", "microloop")

    if not os.path.exists(binary_path):
        print(f"Error: Bundled executable not found at {binary_path}", file=sys.stderr)
        sys.exit(1)

    try:
        # We use os.execv to completely replace the python process with the bundled binary
        # This ensures TUI terminal capabilities (like raw mode) work perfectly without subprocess interference
        os.execv(binary_path, [binary_path] + args)
    except Exception as e:
        print(f"Failed to launch {binary_path}: {e}", file=sys.stderr)
        sys.exit(1)

def run_cli():
    """Entrypoint for the 'ml' command (CLI)."""
    run_bundled(sys.argv[1:])

def run_tui():
    """Entrypoint for the 'microloop' command (TUI)."""
    run_bundled(sys.argv[1:])
