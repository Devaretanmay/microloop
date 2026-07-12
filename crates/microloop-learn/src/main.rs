//! # Microloop Learn — CLI
//!
//! Analyzes agent trajectory logs and auto-generates optimal safety policies.
//!
//! ## Usage
//!
//! ```bash
//! # Analyze a JSONL trajectory file
//! microloop learn --input trajectories.jsonl --output policy.yaml
//!
//! # Pipe from stdin
//! cat trajectories.jsonl | microloop learn > policy.yaml
//!
//! # Show summary without writing
//! microloop learn --input trajectories.jsonl --summarize
//! ```

use std::path::PathBuf;

use clap::Parser;
use microloop_learn::{learn, parse_jsonl, render_policy};

/// Analyze agent trajectory logs and auto-generate safety policies.
///
/// Reads tool-call history (JSONL format) and reverse-engineers the
/// agent's normal behavior patterns. Outputs a YAML policy file tuned
/// to the agent's actual usage — tighter thresholds for error-prone
/// tools, looser for productive multi-step workflows.
#[derive(Parser, Debug)]
#[command(name = "microloop-learn", version, about)]
struct Args {
    /// Path to JSONL trajectory file (use '-' for stdin)
    #[arg(short, long, default_value = "-")]
    input: PathBuf,

    /// Output YAML policy file (omit to print to stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Print a human-readable summary of findings
    #[arg(short, long)]
    summarize: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    // Read input
    let entries = if args.input.to_string_lossy() == "-" {
        // Read from stdin
        let stdin = std::io::stdin();
        let lock = stdin.lock();
        match parse_jsonl(lock) {
            Ok(entries) => entries,
            Err(e) => {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
    } else {
        // Read from file
        let file = match std::fs::File::open(&args.input) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Error opening {}: {e}", args.input.display());
                std::process::exit(1);
            }
        };
        match parse_jsonl(file) {
            Ok(entries) => entries,
            Err(e) => {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
    };

    if entries.is_empty() {
        eprintln!("No valid trajectory entries found.");
        std::process::exit(0);
    }

    // Analyze
    let report = learn(&entries);

    // Summary
    if args.summarize || args.verbose {
        println!("── Microloop Learn — Trajectory Analysis ──────────────────");
        println!("  Tools found:     {}", report.tools.len());
        println!("  Total calls:     {}", report.total_calls);
        println!("  Total loops:     {}", report.total_loops);
        println!("  Global error rate: {:.1}%", report.global_error_rate * 100.0);
        println!();
        println!("  Suggested global max_repeats: {}", report.suggested_global_max_repeats);
        println!();

        for tool in &report.tools {
            let vol = if tool.volatile_fields.is_empty() {
                "none".to_string()
            } else {
                tool.volatile_fields.join(", ")
            };
            println!(
                "  {:24} calls={:<5} repeats={:<3} errors={:<3} rate={:<5.1}%  threshold={}  volatile=[{}]{}",
                tool.name,
                tool.call_count,
                tool.repeat_count,
                tool.error_count,
                tool.error_rate * 100.0,
                tool.suggested_max_repeats,
                vol,
                if tool.oscillation_detected { "  ⚠️  oscillation" } else { "" },
            );
        }
        println!("────────────────────────────────────────────────────────────");
    }

    // Generate policy
    let yaml = render_policy(&report);

    if args.verbose {
        println!();
    }

    // Write output
    if let Some(path) = &args.output {
        match std::fs::write(path, &yaml) {
            Ok(_) => {
                if args.verbose || args.summarize {
                    println!("Written to {}", path.display());
                }
            }
            Err(e) => {
                eprintln!("Error writing {}: {e}", path.display());
                std::process::exit(1);
            }
        }
    } else {
        // Print to stdout
        print!("{yaml}");
    }
}
