#!/bin/bash

# Ensure dependencies in a virtualenv
echo "Setting up virtual environment..."
python3 -m venv .venv
source .venv/bin/activate
pip install fastapi uvicorn requests > /dev/null 2>&1

echo "Pre-compiling binaries..."
cargo build -p microloop-semantic
cargo build -p microloop-proxy

echo "Starting Mock LLM (port 8082)..."
python3 python/mock_upstream.py > mock.log 2>&1 &
MOCK_PID=$!

echo "Starting Semantic Sidecar (port 8081)..."
cargo run -p microloop-semantic > sidecar.log 2>&1 &
SIDECAR_PID=$!

echo "Starting Proxy (port 8080)..."
TARGET_API_URL=http://127.0.0.1:8082 cargo run -p microloop-proxy > proxy.log 2>&1 &
PROXY_PID=$!

# Wait for them to start up
echo "Waiting 20 seconds for services to start and weights to download..."
sleep 20

echo "Running E2E Test..."
python3 python/run_e2e_test.py

echo "--- SIDECAR LOGS ---"
cat sidecar.log
echo "--- PROXY LOGS ---"
cat proxy.log

# Cleanup
echo "Killing background processes..."
kill $MOCK_PID
kill $SIDECAR_PID
kill $PROXY_PID
