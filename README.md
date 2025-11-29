# WRAITH Protocol

**W**ire-speed **R**esilient **A**uthenticated **I**nvisible **T**ransfer **H**andler

A decentralized secure file transfer protocol optimized for high-throughput, low-latency operation with strong security guarantees and traffic analysis resistance.

## Features

- **Fast**: 300+ Mbps throughput, sub-millisecond latency with kernel bypass (AF_XDP)
- **Secure**: XChaCha20-Poly1305 encryption, Noise_XX mutual authentication, forward secrecy
- **Invisible**: Elligator2 key encoding, traffic padding, timing obfuscation, protocol mimicry
- **Decentralized**: Privacy-enhanced DHT, NAT traversal, relay fallback
- **Resilient**: BBR congestion control, connection migration, resume support

## Quick Start

```bash
# Build
cargo build --release

# Send a file
wraith send document.pdf alice@peer.key

# Receive files
wraith receive --output ./downloads

# Run as daemon
wraith daemon --bind 0.0.0.0:0
```

## Project Structure

```
crates/
├── wraith-core/        # Frame encoding, session state, congestion control
├── wraith-crypto/      # Noise handshake, AEAD, Elligator2, ratcheting
├── wraith-transport/   # AF_XDP, io_uring, UDP sockets
├── wraith-obfuscation/ # Padding, timing, cover traffic, mimicry
├── wraith-discovery/   # DHT, relay, NAT traversal
├── wraith-files/       # Chunking, integrity, transfer state
├── wraith-cli/         # Command-line interface
└── wraith-xdp/         # eBPF/XDP programs (Linux-only)
```

## Development

```bash
# Run tests
cargo test --workspace

# Run lints
cargo clippy --workspace -- -D warnings

# Format code
cargo fmt --all

# Run all CI checks
cargo xtask ci

# Generate documentation
cargo doc --workspace --open
```

## Requirements

- Rust 1.75+ (2021 edition)
- Linux 6.2+ (for AF_XDP and io_uring features)
- x86_64 or aarch64 architecture

## Documentation

- [Protocol Technical Specification](ref-docs/protocol_technical_details.md)
- [Implementation Guide](ref-docs/protocol_implementation_guide.md)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
