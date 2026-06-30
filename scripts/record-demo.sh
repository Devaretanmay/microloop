#!/usr/bin/env bash
set -euo pipefail

# Generate the README demo GIF
# Requires: asciinema, agg (brew install asciinema agg)

cd "$(dirname "$0")/.."

echo "Building example..."
cargo build --example basic

echo "Recording terminal session..."
asciinema rec /tmp/microloop-demo.cast \
  --command "cargo run --example basic" \
  --overwrite

echo "Generating GIF..."
agg --font-size 16 --cols 60 --rows 12 \
  /tmp/microloop-demo.cast assets/demo.gif

echo "Done! assets/demo.gif generated ($(wc -c < assets/demo.gif) bytes)"
