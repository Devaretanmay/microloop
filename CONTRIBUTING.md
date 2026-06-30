# Contributing to Microloop

Thanks for your interest. Microloop is small, focused, and meant to stay that way. Contributions that align with the project's scope are welcome.

## Code of Conduct

Be respectful. Disagreement is fine; personal attacks are not. This project follows the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct).

## Scope

Microloop is an agent loop detection engine. It is not:
- An observability platform (that's what upstream dashboards are for)
- A model guardrail (content filtering, PII detection)
- An agent framework

Contributions should stay within the guardrail proxy and state engine domain.

## Getting Started

### 🌟 Good First Issues
If you are new to the project, looking for a place to start? We actively curate our issue tracker with the label **`good first issue`**. These are issues that have a clear scope, don't require deep architectural knowledge, and are perfect for your first PR!

You can find them by searching the issue tracker for `is:issue is:open label:"good first issue"`.

```bash
git clone https://github.com/tanmaydevare/microloop
cd microloop
cargo build --release
cargo test
```

### Prerequisites

- Rust 2024 edition or later (`rustup update stable`)
- `cbindgen` for C header generation (`cargo install cbindgen`)
- No external runtime dependencies (the C core is `no_std`)

## Pull Request Process

1. Open an issue first describing what you want to change. Small bug fixes can skip this.
2. Fork the repo and create a branch (`git checkout -b feature/your-feature`).
3. Make your changes. Keep diffs small.
4. Add or update tests in `src/tests.rs`.
5. Run `cargo test -- --test-threads=1` (the state engine is not thread-safe).
6. Run `cargo build --release` and verify the proxy starts.
7. Update the C header if you change the public C API:

```bash
make cbindgen
```

8. Open a PR. Include a clear description of what changed and why.

## Coding Standards

- **No comments in code** — the code should be self-documenting. Use descriptive names.
- **`no_std` compatible** — the core library (`src/lib.rs`) must compile without the standard library. Keep dependencies minimal.
- **No panicking in the hot path** — verification must always return a valid verdict, even on malformed input.
- **Return early** — exit fast on allow. Latency is a feature.

## Testing

```bash
# Unit tests (state engine, trajectory, output gates)
cargo test

# Integration test (proxy against mock upstream)
cargo run --release --bin microloop-proxy &
python examples/test.py
```

The Python test exercises the C library through ctypes. Both must pass.

## Architecture Notes

- `src/lib.rs` — Public C API. Three entry points: `microloop_init`, `microloop_verify`, `microloop_free`.
- `src/state.rs` — Configuration parsing, rule compilation, error buffer.
- `src/trajectory.rs` — Hash-based sliding window loop detection.
- `src/bin/microloop-proxy/` — Axum HTTP proxy. Thin layer over the C API.
