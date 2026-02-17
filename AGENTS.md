# AGENTS.md

## Rust Monorepo Instructions

This repository is a Rust workspace.

### Do NOT run `cargo` directly.

Always use the Makefile targets instead. Cargo may fail in agent environments.

### Allowed Commands

Use these commands from the repository root:

- `make check` – Check all crates
- `make check-server` – Check only runelink-server
- `make build` – Build all crates
- `make build-server` – Build only runelink-server
- `make test` – Run tests

### Toolchain

The Makefile ensures a stable toolchain is used.  
Do not manually specify `rustup` or override toolchains unless explicitly instructed.

### Working Directory

Always run commands from the repository root.
