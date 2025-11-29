# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

WRAITH (Wire-speed Resilient Authenticated Invisible Transfer Handler) is a decentralized secure file transfer protocol. This repository contains the Rust implementation along with design specifications.

**Current Status:** Initial implementation scaffolding complete, core modules need implementation.

## Build & Development Commands

```bash
# Build the workspace
cargo build --workspace

# Run tests
cargo test --workspace

# Run lints
cargo clippy --workspace -- -D warnings

# Format code
cargo fmt --all

# Run all CI checks
cargo xtask ci

# Build release
cargo build --release

# Generate documentation
cargo doc --workspace --open

# Run the CLI
cargo run -p wraith-cli -- --help
```

## Repository Structure

```
WRAITH-Protocol/
├── crates/
│   ├── wraith-core/        # Frame encoding, session state, congestion control
│   ├── wraith-crypto/      # Noise handshake, AEAD, Elligator2, ratcheting
│   ├── wraith-transport/   # AF_XDP, io_uring, UDP sockets
│   ├── wraith-obfuscation/ # Padding, timing, cover traffic
│   ├── wraith-discovery/   # DHT, relay, NAT traversal
│   ├── wraith-files/       # Chunking, integrity, transfer state
│   ├── wraith-cli/         # Command-line interface (wraith binary)
│   └── wraith-xdp/         # eBPF/XDP programs (Linux-only, excluded from default build)
├── xtask/                  # Build automation (cargo xtask <cmd>)
├── ref-docs/               # Protocol specifications
│   ├── protocol_technical_details.md
│   └── protocol_implementation_guide.md
├── images/                 # Branding assets
├── tests/                  # Integration tests
└── benches/                # Benchmarks
```

## Protocol Architecture

Six-layer design (bottom to top):
1. **Network Layer** - UDP, raw sockets, covert channels
2. **Kernel Acceleration** - AF_XDP, io_uring, zero-copy DMA
3. **Obfuscation Layer** - Elligator2, padding, timing jitter
4. **Crypto Transport** - Noise_XX, XChaCha20-Poly1305, ratcheting
5. **Session Layer** - Stream mux, flow control, BBR congestion
6. **Application Layer** - File transfer, chunking, integrity

## Key Technical Details

### Cryptographic Suite
- **Key Exchange:** X25519 with Elligator2 encoding
- **AEAD:** XChaCha20-Poly1305 (192-bit nonce)
- **Hash:** BLAKE3 (tree-parallelizable)
- **Handshake:** Noise_XX (mutual auth, identity hiding)

### Wire Format
- **Outer Packet:** 8B CID + encrypted payload + 16B auth tag
- **Inner Frame:** 28B header + payload + random padding
- **Frame Types:** DATA, ACK, CONTROL, REKEY, PING/PONG, CLOSE, PAD, STREAM_*, PATH_*

### Performance Targets
- Throughput: 300+ Mbps (10-40 Gbps with kernel bypass)
- Latency: Sub-millisecond with AF_XDP
- Forward secrecy: Ratchet every 2 min or 1M packets

## Development Notes

### Target Platform
- Linux 6.2+ (for AF_XDP, io_uring)
- Primary: x86_64, Secondary: aarch64
- Rust 1.75+ (2021 Edition)

### Key Dependencies
- `chacha20poly1305`, `x25519-dalek`, `blake3` - Cryptography
- `snow` - Noise Protocol framework
- `io-uring` - Async file I/O (Linux)
- `tokio` - Async runtime
- `clap` - CLI parsing

### Threading Model
Thread-per-core with no locks in hot path. Sessions pinned to cores, NUMA-aware allocation.

## Implementation Status

| Crate | Status | Notes |
|-------|--------|-------|
| wraith-core | Scaffolded | Frame parsing works, session/stream need impl |
| wraith-crypto | Scaffolded | AEAD works, Noise handshake needs impl |
| wraith-transport | Scaffolded | UDP fallback, io_uring stub |
| wraith-obfuscation | Scaffolded | Padding modes, timing stubs |
| wraith-discovery | Scaffolded | DHT key derivation, relay stub |
| wraith-files | Scaffolded | Chunker, hasher work |
| wraith-cli | Scaffolded | CLI structure, no functionality |
| wraith-xdp | Not started | Requires eBPF toolchain |
