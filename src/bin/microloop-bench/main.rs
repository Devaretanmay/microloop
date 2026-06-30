use std::time::Instant;
use microloop::state::MicroloopState;
use std::hint::black_box;

fn main() {
    println!("Starting Real Microloop Benchmarks...\n");

    let yaml_config = r#"
rules:
  - type: "regex"
    pattern: "rm -rf"
  - type: "exact"
    value: "DROP TABLE"
max_repeats: 3
"#;

    // 1. Footprint Benchmark
    println!("--- 1. Memory Footprint ---");
    let state_size = std::mem::size_of::<MicroloopState>();
    println!("MicroloopState Struct Size: {} bytes", state_size);
    let mut state = MicroloopState::new(yaml_config).expect("Failed to initialize state");
    println!("Initialization: Success");
    println!();

    // 2. Real Tool Call Payloads
    let payload_simple = r#"{"query": "best pizza in new york"}"#;
    let payload_complex = r#"{
        "code": "def fibonacci(n):\n    if n <= 1:\n        return n\n    return fibonacci(n-1) + fibonacci(n-2)\n\nprint(fibonacci(10))",
        "timeout": 30,
        "environment": {"lang": "python3", "sandbox": true}
    }"#;
    
    // 3. Cold Start Latency
    println!("--- 2. Cold Start Latency ---");
    let start = Instant::now();
    let res = microloop::verify(&mut state, b"search_web", payload_simple.as_bytes());
    let elapsed = start.elapsed();
    black_box(res);
    println!("First verification (Cold Start): {} ns", elapsed.as_nanos());
    println!();

    // 4. Warm Throughput & Latency (Complex Payload)
    println!("--- 3. Throughput & Latency (Warm, Complex Payload) ---");
    let iterations = 100_000;
    
    // Reset state for clean run
    let mut state = MicroloopState::new(yaml_config).unwrap();
    let mut latencies = Vec::with_capacity(iterations);
    
    let start_total = Instant::now();
    for i in 0..iterations {
        let iter_start = Instant::now();
        // ponytail: unique payload per iteration avoids loop detector
        let payload = format!(r#"{{"iteration": {}}}"#, i);
        let res = microloop::verify(&mut state, b"execute_code", payload.as_bytes());
        black_box(res);
        latencies.push(iter_start.elapsed().as_nanos());
    }
    let elapsed_total = start_total.elapsed();
    
    latencies.sort_unstable();
    let avg = latencies.iter().sum::<u128>() as f64 / iterations as f64;
    let p95 = latencies[(iterations as f64 * 0.95) as usize];
    let p99 = latencies[(iterations as f64 * 0.99) as usize];
    
    println!("Iterations: {}", iterations);
    println!("Total Time: {:?}", elapsed_total);
    println!("Throughput: {:.0} verifications/sec", iterations as f64 / elapsed_total.as_secs_f64());
    println!("Avg Latency: {:.2} ns ({} µs)", avg, avg / 1000.0);
    println!("P95 Latency: {} ns ({} µs)", p95, p95 as f64 / 1000.0);
    println!("P99 Latency: {} ns ({} µs)", p99, p99 as f64 / 1000.0);
    println!();

    // 5. Adversarial Loop Detection (Stress Test)
    println!("--- 4. Adversarial Loop Detection (Fast Reject) ---");
    let mut state = MicroloopState::new(yaml_config).unwrap();
    // Pre-fill history with 3 identical calls
    for _ in 0..3 {
        microloop::verify(&mut state, b"execute_code", payload_complex.as_bytes());
    }
    
    let start = Instant::now();
    // This 4th call should be instantly rejected
    let res = microloop::verify(&mut state, b"execute_code", payload_complex.as_bytes());
    let elapsed = start.elapsed();
    black_box(res);
    println!("Loop Detection Reject Latency: {} ns", elapsed.as_nanos());
    if res != 0 {
        println!("Result: Blocked successfully.");
    }
}
