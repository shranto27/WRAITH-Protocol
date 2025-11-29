# WRAITH Protocol

**W**ire-speed **R**esilient **A**uthenticated **I**nvisible **T**ransfer **H**andler

A decentralized secure file transfer protocol optimized for high-throughput, low-latency operation with strong security guarantees and traffic analysis resistance.

![WRAITH Protocol Banner](images/wraith-protocol_banner-graphic.jpg)

## Features

### Performance
- **Wire-Speed Transfers**: 10+ Gbps throughput with AF_XDP kernel bypass
- **Sub-Millisecond Latency**: <1ms packet processing with io_uring
- **Zero-Copy I/O**: Direct NIC-to-application data path
- **BBR Congestion Control**: Optimal bandwidth utilization

### Security
- **Strong Encryption**: XChaCha20-Poly1305 AEAD (256-bit security)
- **Perfect Forward Secrecy**: Double ratchet key derivation
- **Mutual Authentication**: Noise_XX handshake pattern
- **Memory Safety**: Pure Rust implementation

### Privacy
- **Traffic Analysis Resistance**: Elligator2 key encoding
- **Protocol Mimicry**: TLS, WebSocket, DNS-over-HTTPS wrappers
- **Timing Obfuscation**: Configurable packet timing
- **Cover Traffic**: Constant-rate transmission mode

### Decentralization
- **Privacy-Enhanced DHT**: Anonymous peer discovery
- **NAT Traversal**: STUN-like hole punching, relay fallback
- **Connection Migration**: Seamless IP address changes
- **No Central Servers**: Fully peer-to-peer operation

## Quick Start

```bash
# Clone the repository
git clone https://github.com/doublegate/WRAITH-Protocol.git
cd WRAITH-Protocol

# Build all crates
cargo build --release

# Run tests
cargo test --workspace

# Send a file
wraith send document.pdf alice@peer.key

# Receive files
wraith receive --output ./downloads

# Run as daemon
wraith daemon --bind 0.0.0.0:0

# Generate a keypair
wraith keygen --output ~/.wraith/identity.key
```

![WRAITH Protocol Architecture](images/wraith-protocol_arch-infographic.jpg)

## Project Structure

```
WRAITH-Protocol/
├── crates/                      # Rust workspace crates
│   ├── wraith-core/            # Frame encoding, sessions, congestion control
│   ├── wraith-crypto/          # Noise handshake, AEAD, Elligator2, ratcheting
│   ├── wraith-transport/       # AF_XDP, io_uring, UDP sockets
│   ├── wraith-obfuscation/     # Padding, timing, cover traffic, mimicry
│   ├── wraith-discovery/       # DHT, relay, NAT traversal
│   ├── wraith-files/           # Chunking, integrity, transfer state
│   ├── wraith-cli/             # Command-line interface
│   └── wraith-xdp/             # eBPF/XDP programs (Linux-only)
├── docs/                        # Comprehensive documentation
│   ├── architecture/           # Protocol design (5 docs)
│   ├── engineering/            # Development guides (4 docs)
│   ├── integration/            # Embedding & platform support (3 docs)
│   ├── testing/                # Testing strategies (3 docs)
│   ├── operations/             # Deployment & monitoring (3 docs)
│   └── clients/                # Client application docs (25 docs)
├── to-dos/                      # Sprint planning
│   ├── protocol/               # 7 implementation phases
│   ├── clients/                # 8 client application sprints
│   └── ROADMAP.md              # Project roadmap
├── ref-docs/                    # Technical specifications
└── xtask/                       # Build automation
```

## Client Applications

WRAITH Protocol powers a suite of secure applications:

| Client | Description | Status |
|--------|-------------|--------|
| **WRAITH-Transfer** | Direct P2P file transfer with drag-and-drop | Tier 1 |
| **WRAITH-Chat** | E2EE messaging with Double Ratchet | Tier 1 |
| **WRAITH-Sync** | Serverless backup synchronization | Tier 2 |
| **WRAITH-Share** | Distributed anonymous file sharing | Tier 2 |
| **WRAITH-Stream** | Secure media streaming (AV1/Opus) | Tier 3 |
| **WRAITH-Mesh** | IoT mesh networking | Tier 3 |
| **WRAITH-Publish** | Censorship-resistant publishing | Tier 3 |
| **WRAITH-Vault** | Distributed secret storage (Shamir SSS) | Tier 3 |

See [Client Documentation](docs/clients/overview.md) for details.

## Development

### Prerequisites

- **Rust 1.75+** (2021 edition)
- **Linux 6.2+** (for AF_XDP and io_uring)
- **x86_64 or aarch64** architecture
- **clang/LLVM** (for XDP/eBPF compilation)

### Build Commands

```bash
# Development build
cargo build --workspace

# Release build with optimizations
cargo build --release

# Run all tests
cargo test --workspace

# Run lints
cargo clippy --workspace -- -D warnings

# Format code
cargo fmt --all

# Run all CI checks
cargo xtask ci

# Generate API documentation
cargo doc --workspace --open

# Run benchmarks
cargo bench --workspace
```

### Testing

```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test '*'

# Property-based tests
cargo test --features proptest

# Run with coverage
cargo tarpaulin --workspace --out Html
```

## Documentation

### Architecture & Design
- [Protocol Overview](docs/architecture/protocol-overview.md)
- [Layer Design](docs/architecture/layer-design.md)
- [Security Model](docs/architecture/security-model.md)
- [Performance Architecture](docs/architecture/performance-architecture.md)
- [Network Topology](docs/architecture/network-topology.md)

### Development
- [Development Guide](docs/engineering/development-guide.md)
- [Coding Standards](docs/engineering/coding-standards.md)
- [API Reference](docs/engineering/api-reference.md)
- [Dependency Management](docs/engineering/dependency-management.md)

### Integration
- [Embedding Guide](docs/integration/embedding-guide.md)
- [Platform Support](docs/integration/platform-support.md)
- [Interoperability](docs/integration/interoperability.md)

### Testing & Operations
- [Testing Strategy](docs/testing/testing-strategy.md)
- [Performance Benchmarks](docs/testing/performance-benchmarks.md)
- [Deployment Guide](docs/operations/deployment-guide.md)
- [Monitoring](docs/operations/monitoring.md)

### Specifications
- [Protocol Technical Details](ref-docs/protocol_technical_details.md)
- [Implementation Guide](ref-docs/protocol_implementation_guide.md)

### Project Planning
- [Project Roadmap](to-dos/ROADMAP.md)
- [Documentation Status](docs/DOCUMENTATION_STATUS.md)

## Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| Throughput (10 GbE) | >9 Gbps | AF_XDP with zero-copy |
| Throughput (1 GbE) | >950 Mbps | With encryption |
| Handshake Latency | <50 ms | LAN conditions |
| Packet Latency | <1 ms | NIC to application |
| Memory per Session | <10 MB | Including buffers |
| CPU @ 10 Gbps | <50% | 8-core system |

## Security

WRAITH Protocol is designed with security as a core principle:

- **Cryptography**: XChaCha20-Poly1305, X25519, BLAKE3, Noise_XX
- **Forward Secrecy**: Double ratchet with DH and symmetric ratchets
- **Traffic Analysis Resistance**: Elligator2, padding, timing obfuscation
- **Memory Safety**: Rust with no unsafe code in crypto paths
- **Constant-Time Operations**: Side-channel resistant implementations

For security issues, please see [SECURITY.md](SECURITY.md).

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'feat: add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.

## Acknowledgments

WRAITH Protocol builds on the work of many excellent projects:

- [Noise Protocol Framework](https://noiseprotocol.org/) - Handshake patterns
- [WireGuard](https://www.wireguard.com/) - Inspiration for simplicity
- [QUIC](https://quicwg.org/) - Connection migration concepts
- [libp2p](https://libp2p.io/) - DHT and NAT traversal patterns
- [Signal Protocol](https://signal.org/docs/) - Double ratchet algorithm

---

**WRAITH Protocol** - *Secure. Fast. Invisible.*
