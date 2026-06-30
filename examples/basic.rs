fn main() {
    let yaml = r#"
max_repeats: 3
history_window: 8
"#;

    let mut state = microloop::state::MicroloopState::new(yaml)
        .expect("Failed to initialize Microloop");

    println!("\x1b[1m\x1b[38;2;103;58;183m▸ microloop v0.1.1\x1b[0m");
    println!("  Config: max_repeats=3, history_window=8\n");

    // --- Scene 1: Identical calls ---
    println!("\x1b[1m\x1b[38;2;147;197;253m[1] Identical tool calls\x1b[0m");
    for i in 1..=4 {
        let v = microloop::verify(
            &mut state,
            b"write_file",
            b"{\"path\": \"/tmp/a.txt\", \"content\": \"hello\"}",
        );
        let (icon, color) = if v == 0 {
            ("✓", "\x1b[32m")
        } else {
            ("✗", "\x1b[31m")
        };
        let label = if v == 0 { "ALLOW" } else { "BLOCK" };
        println!("    {color}{icon}\x1b[0m  Call {i}: write_file → {color}{label}\x1b[0m");
    }

    // Reset state for next demo
    let mut state = microloop::state::MicroloopState::new(yaml)
        .expect("Failed to reinitialize");

    // --- Scene 2: Unique calls pass through ---
    println!("\n\x1b[1m\x1b[38;2;147;197;253m[2] Unique calls (no false positives)\x1b[0m");
    let files = ["a.txt", "b.txt", "c.txt", "d.txt", "e.txt"];
    for (i, f) in files.iter().enumerate() {
        let args = format!("{{\"path\": \"/tmp/{}\", \"content\": \"data\"}}", f);
        let v = microloop::verify(&mut state, b"write_file", args.as_bytes());
        let (icon, color) = if v == 0 {
            ("✓", "\x1b[32m")
        } else {
            ("✗", "\x1b[31m")
        };
        let label = if v == 0 { "ALLOW" } else { "BLOCK" };
        println!(
            "    {color}{icon}\x1b[0m  Call {}: write_file({}) → {color}{label}\x1b[0m",
            i + 1,
            f
        );
    }

    // Reset state for next demo
    let mut state = microloop::state::MicroloopState::new(yaml)
        .expect("Failed to reinitialize");

    // --- Scene 3: Alternating loop (pattern detection) ---
    println!("\n\x1b[1m\x1b[38;2;147;197;253m[3] Alternating loop (pattern detection)\x1b[0m");
    let tools: &[(&[u8], &str)] = &[
        (b"execute_shell", "execute_shell"),
        (b"read_file", "read_file"),
    ];
    for i in 0..6 {
        let (tool_bytes, tool_name) = tools[i % 2];
        let v = microloop::verify(&mut state, tool_bytes, b"{}");
        let (icon, color) = if v == 0 {
            ("✓", "\x1b[32m")
        } else {
            ("✗", "\x1b[31m")
        };
        let label = if v == 0 { "ALLOW" } else { "BLOCK" };
        println!(
            "    {color}{icon}\x1b[0m  Call {}: {} → {color}{label}\x1b[0m",
            i + 1,
            tool_name
        );
    }

    println!("\n\x1b[2m  Zero API calls wasted. Zero tokens burned.\x1b[0m");
}
