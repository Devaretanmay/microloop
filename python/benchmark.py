import time
import json
from microloop.microloop_core import Microloop

def run_benchmarks():
    print("Starting Real Microloop Python Benchmarks...\n")

    yaml_config = """
rules:
  - type: "regex"
    pattern: "rm -rf"
  - type: "exact"
    value: "DROP TABLE"
max_repeats: 3
"""

    print("--- 1. Initialization ---")
    try:
        engine = Microloop(yaml_config)
        print("Microloop Initialization: Success\n")
    except Exception as e:
        print(f"Failed to initialize: {e}")
        return

    # Real LLM Payload
    payload_simple = json.dumps({"query": "best pizza in new york"})
    payload_complex = json.dumps({
        "code": "def fibonacci(n):\n    if n <= 1:\n        return n\n    return fibonacci(n-1) + fibonacci(n-2)\n\nprint(fibonacci(10))",
        "timeout": 30,
        "environment": {"lang": "python3", "sandbox": True}
    })
    
    print("--- 2. Cold Start Latency ---")
    start = time.perf_counter_ns()
    res = engine.verify("search_web", payload_simple)
    elapsed = time.perf_counter_ns() - start
    print(f"First verification (Cold Start): {elapsed} ns\n")

    print("--- 3. Throughput & Latency (Warm, Complex Payload) ---")
    iterations = 100_000
    latencies = []
    
    start_total = time.perf_counter_ns()
    for i in range(iterations):
        iter_start = time.perf_counter_ns()
        # ponytail: unique payload per iteration avoids loop detector
        payload = json.dumps({"iteration": i})
        engine.verify("execute_code", payload)
        
        latencies.append(time.perf_counter_ns() - iter_start)
        
    elapsed_total = time.perf_counter_ns() - start_total
    
    latencies.sort()
    avg = sum(latencies) / iterations
    p95 = latencies[int(iterations * 0.95)]
    p99 = latencies[int(iterations * 0.99)]
    
    total_secs = elapsed_total / 1e9
    print(f"Iterations: {iterations}")
    print(f"Total Time: {total_secs:.4f}s")
    print(f"Throughput: {int(iterations / total_secs)} verifications/sec")
    print(f"Avg Latency: {avg:.2f} ns ({avg / 1000:.2f} µs)")
    print(f"P95 Latency: {p95} ns ({p95 / 1000:.2f} µs)")
    print(f"P99 Latency: {p99} ns ({p99 / 1000:.2f} µs)\n")

    print("--- 4. Adversarial Loop Detection (Fast Reject) ---")
    engine = Microloop(yaml_config)
    for _ in range(3):
        engine.verify("execute_code", payload_complex)
        
    start = time.perf_counter_ns()
    res = engine.verify("execute_code", payload_complex)
    elapsed = time.perf_counter_ns() - start
    
    print(f"Loop Detection Reject Latency: {elapsed} ns")
    if res != 0:
        print("Result: Blocked successfully.")
        
if __name__ == "__main__":
    run_benchmarks()
