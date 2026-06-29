# Microloop

Microloop is a lightweight, ultra-fast runtime safety layer for autonomous agents. It detects and blocks tool-call loops before they burn tokens or crash your infrastructure.

This repository contains the `no_std` Rust core library, C ABI FFI bindings, and Python adapters (LangChain, CrewAI, AutoGen, LangGraph).

## Features
- **Ultra-fast**: Executes in under 70 microseconds.
- **Dependency-free**: Pure `no_std` Rust core.
- **Universal**: Plugs into any language or harness via C FFI and adapters.

## Building
```bash
cargo build --release
```
