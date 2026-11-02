# How to Build apple-message-forensics from Source

This guide explains how to compile the tool and its bundled report reader from source code, including cross-compilation for Windows targets.

## Prerequisites

- [Rust](https://rustup.rs/) (edition 2024 compatible toolchain).
- [Homebrew](https://brew.sh/) on macOS.
- `just` task runner.

Install build tools:

```bash
brew install just
```

## Clone and Run Tests

1. Clone the repository:

```bash
git clone https://github.com/McFlip/apple-message-forensics.git
cd apple-message-forensics
```

2. Run the test suite:

```bash
just check
```

This runs `cargo check`, `cargo clippy`, and `cargo test`.

## Build Native Binaries

Compile release binaries for your host architecture:

```bash
cargo build --release --bins
```

The output executable is placed at `target/release/apple-message-forensics`.

## Cross-Compile Reader for Windows

The desktop reader uses an Axum backend and can be compiled to run on Windows targets from a Mac host using MinGW:

1. Install the MinGW compiler:

```bash
brew install mingw-w64
```

2. Add the Windows compilation target:

```bash
rustup target add x86_64-pc-windows-gnu
```

3. Build the reader binaries:

```bash
just build-reader
```

This generates `target/x86_64-pc-windows-gnu/release/reader.exe` and `report-server.exe` and copies them into `bin/`.

## Bundle the Reader with a Generated Report

Once an HTML report is generated in `output/report/`:

```bash
just bundle-reader
```

This verifies `output/report/index.html` exists and copies the reader executables alongside it.
