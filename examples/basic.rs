fn main() {
    let yaml = r#"
max_repeats: 3
"#;

    let mut state = microloop::state::MicroloopState::new(yaml)
        .expect("Failed to initialize Microloop");

    println!("Microloop initialized. Sending identical tool calls...\n");

    for i in 1..=4 {
        let verdict = microloop::verify(&mut state, b"write_file", b"{\"path\": \"/tmp/test.txt\", \"content\": \"hello\"}");
        let label = match verdict {
            0 => "ALLOW",
            1 | 2 | 3 => "BLOCK",
            _ => "UNKNOWN",
        };
        print!("  Call {}: write_file → {}", i, label);
        if verdict != 0 {
            let err = std::str::from_utf8(&state.error_buffer)
                .unwrap_or("")
                .trim_end_matches('\0');
            println!("  ({})", err);
        } else {
            println!();
        }
    }

    println!("\nThe 3rd identical call was blocked instantly — no API roundtrip needed.");
}
