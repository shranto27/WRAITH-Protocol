# WRAITH Protocol Testing Strategy

**Document Version:** 1.0.0
**Last Updated:** 2025-11-28
**Status:** Testing Documentation

---

## Overview

This document outlines the comprehensive testing strategy for the WRAITH Protocol. Our testing approach ensures correctness, performance, security, and reliability across all components.

**Testing Pyramid:**
```
        ┌─────────────┐
        │  E2E Tests  │  (10%)
        ├─────────────┤
        │ Integration │  (30%)
        │    Tests    │
        ├─────────────┤
        │    Unit     │  (60%)
        │    Tests    │
        └─────────────┘
```

**Coverage Goals:**
- **Unit Tests:** >80% line coverage
- **Integration Tests:** All critical paths
- **E2E Tests:** All user workflows
- **Security Tests:** 100% coverage for crypto operations

---

## Testing Levels

### Unit Tests

**Scope:** Individual functions and modules in isolation.

**Location:** `#[cfg(test)]` modules within each source file.

**Example:**
```rust
// wraith-crypto/src/blake3.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake3_hash_empty() {
        let hash = Blake3Hash::hash(b"");
        assert_eq!(
            hash.to_hex(),
            "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"
        );
    }

    #[test]
    fn test_blake3_hash_consistency() {
        let data = b"test data";
        let hash1 = Blake3Hash::hash(data);
        let hash2 = Blake3Hash::hash(data);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_blake3_keyed_hash() {
        let key = [0u8; 32];
        let data = b"message";
        let hash = Blake3Hash::keyed_hash(&key, data);

        // Verify hash is different from unkeyed
        assert_ne!(hash, Blake3Hash::hash(data));
    }
}
```

**Run unit tests:**
```bash
# All unit tests
cargo test --lib

# Specific crate
cargo test -p wraith-crypto --lib

# Specific test
cargo test test_blake3_hash_consistency

# With coverage
cargo tarpaulin --lib --out Html
```

### Integration Tests

**Scope:** Multiple components working together.

**Location:** `tests/` directory in workspace root.

**Example:**
```rust
// tests/session_handshake.rs
use wraith_core::{Session, Keypair};
use wraith_transport::UdpTransport;
use tokio::time::timeout;
use std::time::Duration;

#[tokio::test]
async fn test_noise_handshake_successful() {
    // Setup initiator
    let initiator_keypair = Keypair::generate();
    let initiator_transport = UdpTransport::bind("127.0.0.1:0".parse().unwrap())
        .await
        .unwrap();
    let initiator_addr = initiator_transport.local_addr();

    // Setup responder
    let responder_keypair = Keypair::generate();
    let responder_transport = UdpTransport::bind("127.0.0.1:0".parse().unwrap())
        .await
        .unwrap();
    let responder_addr = responder_transport.local_addr();

    // Perform handshake concurrently
    let (initiator_result, responder_result) = tokio::join!(
        Session::connect(
            initiator_keypair.clone(),
            responder_addr,
            initiator_transport,
        ),
        Session::accept(
            responder_keypair.clone(),
            responder_transport,
        )
    );

    // Verify both sides succeeded
    assert!(initiator_result.is_ok());
    assert!(responder_result.is_ok());

    let initiator_session = initiator_result.unwrap();
    let responder_session = responder_result.unwrap();

    // Verify mutual authentication
    assert_eq!(
        initiator_session.peer_id(),
        responder_keypair.public()
    );
    assert_eq!(
        responder_session.peer_id(),
        initiator_keypair.public()
    );
}

#[tokio::test]
async fn test_handshake_timeout() {
    let keypair = Keypair::generate();
    let transport = UdpTransport::bind("127.0.0.1:0".parse().unwrap())
        .await
        .unwrap();

    // Try to connect to non-existent peer
    let nonexistent_peer = "127.0.0.1:9999".parse().unwrap();

    let result = timeout(
        Duration::from_secs(5),
        Session::connect(keypair, nonexistent_peer, transport)
    ).await;

    assert!(result.is_err() || result.unwrap().is_err());
}
```

**Run integration tests:**
```bash
# All integration tests
cargo test --tests

# Specific integration test
cargo test --test session_handshake

# With logging
RUST_LOG=debug cargo test --tests -- --nocapture
```

### End-to-End Tests

**Scope:** Complete user workflows across multiple processes.

**Location:** `tests/e2e/` directory.

**Example:**
```rust
// tests/e2e/file_transfer.rs
use std::process::{Command, Stdio};
use tempfile::TempDir;

#[test]
fn test_cli_file_transfer_e2e() {
    let temp_dir = TempDir::new().unwrap();

    // Create test file (10 MB)
    let test_file = temp_dir.path().join("test.bin");
    let test_data: Vec<u8> = (0..10_000_000).map(|i| (i % 256) as u8).collect();
    std::fs::write(&test_file, &test_data).unwrap();

    // Start receiver in background
    let receiver_output = temp_dir.path().join("received.bin");
    let mut receiver = Command::new("./target/debug/wraith-cli")
        .args(&["recv", "--output", receiver_output.to_str().unwrap()])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    // Wait for receiver to start
    std::thread::sleep(std::time::Duration::from_secs(1));

    // Send file
    let sender_status = Command::new("./target/debug/wraith-cli")
        .args(&[
            "send",
            test_file.to_str().unwrap(),
            "--peer",
            "127.0.0.1:41641",
        ])
        .status()
        .unwrap();

    assert!(sender_status.success());

    // Wait for receiver
    let receiver_status = receiver.wait().unwrap();
    assert!(receiver_status.success());

    // Verify received file
    let received_data = std::fs::read(&receiver_output).unwrap();
    assert_eq!(test_data, received_data);
}
```

**Run E2E tests:**
```bash
# Build binaries first
cargo build --release

# Run E2E tests
cargo test --test '*e2e*' -- --test-threads=1
```

---

## Property-Based Testing

**Use proptest for randomized testing:**
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_frame_serialization_roundtrip(
        frame_type in 0u8..=5,
        payload in prop::collection::vec(any::<u8>(), 0..1200)
    ) {
        let frame = Frame {
            frame_type: FrameType::from_u8(frame_type).unwrap(),
            payload: payload.clone(),
        };

        let serialized = frame.to_bytes();
        let deserialized = Frame::parse(&serialized).unwrap();

        prop_assert_eq!(frame.frame_type, deserialized.frame_type);
        prop_assert_eq!(frame.payload, deserialized.payload);
    }

    #[test]
    fn test_encryption_roundtrip(
        plaintext in prop::collection::vec(any::<u8>(), 0..10000)
    ) {
        let mut keys = SymmetricKeys::new_test();

        let ciphertext = keys.encrypt(&plaintext);
        let decrypted = keys.decrypt(&ciphertext).unwrap();

        prop_assert_eq!(plaintext, decrypted);
    }
}
```

---

## Test Fixtures and Helpers

### Test Keypairs

```rust
// tests/common/mod.rs
pub fn test_keypair(seed: u8) -> Keypair {
    let seed_array = [seed; 32];
    Keypair::from_seed(&seed_array)
}

pub fn alice_keypair() -> Keypair {
    test_keypair(1)
}

pub fn bob_keypair() -> Keypair {
    test_keypair(2)
}
```

### Mock Transport

```rust
pub struct MockTransport {
    sent: Arc<Mutex<Vec<Vec<u8>>>>,
    recv_queue: Arc<Mutex<VecDeque<Vec<u8>>>>,
}

impl MockTransport {
    pub fn new() -> Self {
        Self {
            sent: Arc::new(Mutex::new(Vec::new())),
            recv_queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub async fn send(&self, data: Vec<u8>) {
        self.sent.lock().await.push(data);
    }

    pub async fn recv(&self) -> Option<Vec<u8>> {
        self.recv_queue.lock().await.pop_front()
    }

    pub async fn inject_recv(&self, data: Vec<u8>) {
        self.recv_queue.lock().await.push_back(data);
    }

    pub async fn get_sent(&self) -> Vec<Vec<u8>> {
        self.sent.lock().await.clone()
    }
}
```

### Test DHT Network

```rust
pub struct TestDhtNetwork {
    nodes: HashMap<[u8; 20], DhtNode>,
}

impl TestDhtNetwork {
    pub fn new(node_count: usize) -> Self {
        let nodes = (0..node_count)
            .map(|i| {
                let node_id = blake3_hash(&[i as u8]);
                let node = DhtNode::new_test(node_id);
                (node_id, node)
            })
            .collect();

        Self { nodes }
    }

    pub async fn put(&mut self, key: &[u8; 20], value: Vec<u8>) {
        // Store in k closest nodes
        let closest = self.find_closest_nodes(key, 20);
        for node_id in closest {
            self.nodes.get_mut(node_id).unwrap().store(key, value.clone());
        }
    }

    pub async fn get(&self, key: &[u8; 20]) -> Option<Vec<u8>> {
        let closest = self.find_closest_nodes(key, 20);
        for node_id in closest {
            if let Some(value) = self.nodes.get(node_id).unwrap().retrieve(key) {
                return Some(value);
            }
        }
        None
    }
}
```

---

## Benchmarking

### Criterion Benchmarks

**Location:** `benches/` directory.

**Example:**
```rust
// benches/crypto_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use wraith_crypto::Blake3Hash;

fn bench_blake3_hash(c: &mut Criterion) {
    let sizes = [1024, 1024 * 1024, 10 * 1024 * 1024];

    for size in sizes {
        let data = vec![0u8; size];

        let mut group = c.benchmark_group("blake3");
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_function(format!("hash_{}_bytes", size), |b| {
            b.iter(|| Blake3Hash::hash(black_box(&data)))
        });

        group.finish();
    }
}

fn bench_encryption(c: &mut Criterion) {
    let sizes = [1024, 64 * 1024, 1024 * 1024];
    let mut keys = SymmetricKeys::new_test();

    for size in sizes {
        let plaintext = vec![0u8; size];

        let mut group = c.benchmark_group("encryption");
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_function(format!("encrypt_{}_bytes", size), |b| {
            b.iter(|| keys.encrypt(black_box(&plaintext)))
        });

        group.finish();
    }
}

criterion_group!(benches, bench_blake3_hash, bench_encryption);
criterion_main!(benches);
```

**Run benchmarks:**
```bash
# All benchmarks
cargo bench

# Specific benchmark
cargo bench crypto_bench

# Generate flamegraph
cargo flamegraph --bench crypto_bench
```

---

## Test Coverage

### Measure Coverage

**Using cargo-tarpaulin:**
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate HTML coverage report
cargo tarpaulin --out Html --output-dir coverage

# Generate multiple formats
cargo tarpaulin --out Html --out Xml --out Lcov

# Exclude specific files
cargo tarpaulin --exclude-files 'tests/*' --exclude-files 'benches/*'

# Coverage for specific package
cargo tarpaulin -p wraith-crypto
```

**CI Integration:**
```yaml
# .github/workflows/coverage.yml
- name: Code Coverage
  run: |
    cargo tarpaulin --out Xml --output-dir coverage

- name: Upload to Codecov
  uses: codecov/codecov-action@v3
  with:
    files: coverage/cobertura.xml
```

---

## Continuous Integration

### GitHub Actions Workflow

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, nightly]

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: ${{ matrix.rust }}
          override: true
          components: rustfmt, clippy

      - name: Cache cargo
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Format Check
        run: cargo fmt -- --check

      - name: Clippy
        run: cargo clippy -- -D warnings

      - name: Unit Tests
        run: cargo test --lib

      - name: Integration Tests
        run: cargo test --tests

      - name: Doc Tests
        run: cargo test --doc
```

---

## Stress Testing

### Load Tests

```rust
#[tokio::test]
#[ignore]  // Run manually: cargo test --ignored
async fn stress_test_concurrent_transfers() {
    let concurrent_transfers = 100;
    let file_size = 10 * 1024 * 1024;  // 10 MB

    let mut handles = Vec::new();

    for i in 0..concurrent_transfers {
        let handle = tokio::spawn(async move {
            let session = create_test_session().await;
            let mut transfer = FileTransfer::new(session, Default::default());

            let data = vec![i as u8; file_size];
            transfer.send_data(&data).await.unwrap();
        });

        handles.push(handle);
    }

    // Wait for all transfers
    for handle in handles {
        handle.await.unwrap();
    }
}
```

### Fuzz Testing

```rust
// fuzz/fuzz_targets/frame_parser.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use wraith_core::Frame;

fuzz_target!(|data: &[u8]| {
    // Should never panic
    let _ = Frame::parse(data);
});
```

**Run fuzzer:**
```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Run fuzzer
cargo fuzz run frame_parser

# Run for specific duration
cargo fuzz run frame_parser -- -max_total_time=3600
```

---

## Test Organization

### Directory Structure

```
WRAITH-Protocol/
├── tests/                      # Integration tests
│   ├── common/
│   │   └── mod.rs             # Shared test helpers
│   ├── session_handshake.rs
│   ├── file_transfer.rs
│   └── e2e/
│       ├── cli_transfer.rs
│       └── multi_peer.rs
├── benches/                    # Benchmarks
│   ├── crypto_bench.rs
│   └── transport_bench.rs
├── fuzz/                       # Fuzz tests
│   └── fuzz_targets/
│       ├── frame_parser.rs
│       └── crypto_ops.rs
└── crates/
    └── wraith-core/
        └── src/
            └── session.rs     # Unit tests in #[cfg(test)]
```

---

## Best Practices

1. **Test Naming:** Use descriptive names (`test_<module>_<scenario>_<expected_result>`)
2. **Test Independence:** Each test should run independently
3. **Deterministic Tests:** Avoid timing-dependent assertions
4. **Fast Tests:** Unit tests should complete in <1s
5. **Clear Assertions:** Use descriptive assertion messages
6. **Test Coverage:** Aim for >80% line coverage
7. **Continuous Testing:** Run tests in CI on every commit

---

## See Also

- [Performance Benchmarks](performance-benchmarks.md)
- [Security Testing](security-testing.md)
- [Development Guide](../engineering/development-guide.md)
