use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, exit};

/// Relative path from the project root to the harness CLI entry point.
const HARNESS_RELATIVE_PATH: &str =
    "apps/microloop-harness/packages/coding-agent/src/cli.ts";

/// Number of parent directories to traverse from the binary to reach the
/// project root in a standard Cargo build layout.
const BINARY_TO_PROJECT_ROOT_DEPTH: usize = 3; // target/debug/ -> project root

/// Try to resolve the harness path from an environment variable override.
fn from_env_override() -> Option<PathBuf> {
    env::var("MICROLOOP_HARNESS_PATH").ok().map(PathBuf::from)
}

/// Try to resolve the harness path relative to the current executable.
///
/// Searches upward from the binary's location for the project root
/// (identified by containing a `Cargo.toml` with a `[workspace]` section).
fn from_exe_location() -> Option<PathBuf> {
    let exe = env::current_exe().ok()?;
    let exe_dir = exe.parent()?;

    // Walk up the directory tree to find the project root
    for depth in 0..=BINARY_TO_PROJECT_ROOT_DEPTH + 2 {
        let mut probe = exe_dir.to_path_buf();
        for _ in 0..depth {
            if !probe.pop() {
                break;
            }
        }
        let candidate = probe.join(HARNESS_RELATIVE_PATH);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}

/// Try to resolve the harness path relative to the current working directory.
///
/// Checks if the CWD is the project root (contains a Cargo.toml with
/// `members =` indicating a workspace) and appends the relative harness path.
fn from_cwd() -> Option<PathBuf> {
    let cwd = env::current_dir().ok()?;
    let candidate = cwd.join(HARNESS_RELATIVE_PATH);
    if candidate.exists() {
        return Some(candidate);
    }

    // Also try one level up (in case the user is in a subdirectory)
    if let Some(parent) = cwd.parent() {
        let candidate = parent.join(HARNESS_RELATIVE_PATH);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}

/// Resolve the path to the harness CLI script using all available strategies.
fn find_harness_script() -> Result<PathBuf, String> {
    // 1. Environment variable override
    if let Some(path) = from_env_override() {
        if path.exists() {
            return Ok(std::fs::canonicalize(&path).unwrap_or(path));
        }
        eprintln!(
            "Warning: MICROLOOP_HARNESS_PATH={} does not exist, falling back",
            path.display()
        );
    }

    // 2. Relative to the executable's location
    if let Some(path) = from_exe_location() {
        return Ok(path);
    }

    // 3. Relative to the current working directory
    if let Some(path) = from_cwd() {
        return Ok(path);
    }

    Err(format!(
        "Could not find harness script at '{}'. \
         Make sure you are running from the project root, or set \
         MICROLOOP_HARNESS_PATH to the absolute path of cli.ts",
        HARNESS_RELATIVE_PATH
    ))
}

/// Given the path to the harness script, find the tsconfig.json for the
/// microloop-harness project.
///
/// Script path: apps/microloop-harness/packages/coding-agent/src/cli.ts
/// tsconfig.json: apps/microloop-harness/tsconfig.json
fn find_tsconfig(script_path: &Path) -> Result<PathBuf, String> {
    // Navigate: cli.ts → src/ → coding-agent/ → packages/ → microloop-harness/
    let script_dir = script_path.parent().ok_or_else(|| {
        format!("Script path has no parent: {}", script_path.display())
    })?;

    let src_dir = script_dir.parent().ok_or_else(|| {
        format!("Could not find src directory from: {}", script_dir.display())
    })?;

    let pkg_dir = src_dir.parent().ok_or_else(|| {
        format!("Could not find package directory from: {}", src_dir.display())
    })?;

    let harness_dir = pkg_dir.parent().ok_or_else(|| {
        format!("Could not find harness directory from: {}", pkg_dir.display())
    })?;

    let tsconfig = harness_dir.join("tsconfig.json");
    if !tsconfig.exists() {
        return Err(format!(
            "tsconfig.json not found at {}",
            tsconfig.display()
        ));
    }

    Ok(tsconfig)
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let script_path = match find_harness_script() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: {e}");
            exit(1);
        }
    };

    let tsconfig_path = match find_tsconfig(&script_path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: {e}");
            exit(1);
        }
    };

    let status = Command::new("npx")
        .arg("tsx")
        .arg("--tsconfig")
        .arg(&tsconfig_path)
        .arg(&script_path)
        .args(&args)
        .status()
        .expect("Failed to execute npx tsx for microloop harness");

    exit(status.code().unwrap_or(1));
}
