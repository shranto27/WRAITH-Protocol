# WRAITH Protocol Security Testing

**Document Version:** 1.0.0
**Last Updated:** 2025-11-28
**Status:** Testing Documentation

---

## Overview

Security testing ensures that WRAITH Protocol maintains its security and privacy guarantees under attack. This document describes security testing methodologies, attack scenarios, and validation procedures.

**Security Properties Under Test:**
- Confidentiality (encryption)
- Integrity (authentication)
- Availability (DoS resistance)
- Privacy (traffic analysis resistance)
- Forward secrecy
- Replay protection

---

## Cryptographic Testing

### Constant-Time Verification

**Purpose:** Ensure cryptographic operations don't leak secrets via timing side-channels.

**Test framework:**
```rust
use criterion::{black_box, criterion_group, Criterion};
use std::time::Duration;

fn test_constant_time_compare(c: &mut Criterion) {
    let mut group = c.benchmark_group("constant_time");
    group.measurement_time(Duration::from_secs(60));

    // Generate test cases
    let tag1 = [0xAAu8; 16];
    let tag2_same = [0xAAu8; 16];
    let tag2_diff = [0xABu8; 16];

    // Measure timing for same tags
    let mut same_times = Vec::new();
    group.bench_function("compare_same", |b| {
        b.iter(|| {
            let start = Instant::now();
            let result = constant_time_eq(black_box(&tag1), black_box(&tag2_same));
            same_times.push(start.elapsed());
            result
        })
    });

    // Measure timing for different tags
    let mut diff_times = Vec::new();
    group.bench_function("compare_different", |b| {
        b.iter(|| {
            let start = Instant::now();
            let result = constant_time_eq(black_box(&tag1), black_box(&tag2_diff));
            diff_times.push(start.elapsed());
            result
        })
    });

    // Statistical analysis
    let same_mean = average(&same_times);
    let diff_mean = average(&diff_times);
    let t_statistic = t_test(&same_times, &diff_times);

    // Verify constant-time property (p > 0.05)
    assert!(t_statistic.p_value > 0.05, "Timing leak detected!");

    group.finish();
}
```

**Validation:**
- Run with CPU pinning to reduce noise
- Collect thousands of samples
- Use statistical tests (t-test, KS-test)
- Verify p-value > 0.05 (no distinguishable difference)

### Zeroization Testing

**Purpose:** Verify secret data is properly erased from memory.

**Test code:**
```rust
#[test]
fn test_keys_zeroized_on_drop() {
    use std::ptr;

    // Allocate keys
    let keys = SymmetricKeys::new_test();
    let keys_ptr = &keys as *const SymmetricKeys as *const u8;

    // Read memory before drop
    let mut before = vec![0u8; std::mem::size_of::<SymmetricKeys>()];
    unsafe {
        ptr::copy_nonoverlapping(keys_ptr, before.as_mut_ptr(), before.len());
    }

    // Verify keys are non-zero
    assert!(before.iter().any(|&b| b != 0), "Keys should be non-zero");

    // Drop keys (zeroization triggered)
    drop(keys);

    // Read memory after drop
    let mut after = vec![0u8; std::mem::size_of::<SymmetricKeys>()];
    unsafe {
        ptr::copy_nonoverlapping(keys_ptr, after.as_mut_ptr(), after.len());
    }

    // Verify memory is zeroed
    assert!(after.iter().all(|&b| b == 0), "Keys not properly zeroized!");
}
```

**Note:** This test is brittle (memory may be reused). Use under Valgrind/MSAN for better validation.

### Randomness Testing

**Purpose:** Verify RNG produces high-quality random numbers.

**Test suite:**
```rust
use rand::RngCore;

#[test]
fn test_rng_statistical_properties() {
    let mut rng = OsRng;
    let sample_size = 1_000_000;

    // Generate random bytes
    let mut bytes = vec![0u8; sample_size];
    rng.fill_bytes(&mut bytes);

    // Test 1: Chi-square test for uniform distribution
    let chi_square = chi_square_test(&bytes);
    assert!(chi_square < 293.25, "RNG fails chi-square test");  // p=0.05, df=255

    // Test 2: Serial correlation test
    let correlation = serial_correlation(&bytes);
    assert!(correlation.abs() < 0.01, "RNG has serial correlation");

    // Test 3: Runs test
    let runs = count_runs(&bytes);
    let expected_runs = sample_size / 2;
    let std_dev = (sample_size / 4.0).sqrt();
    assert!((runs as f64 - expected_runs as f64).abs() < 3.0 * std_dev);
}

fn chi_square_test(data: &[u8]) -> f64 {
    let mut counts = [0usize; 256];
    for &byte in data {
        counts[byte as usize] += 1;
    }

    let expected = data.len() as f64 / 256.0;
    counts.iter()
        .map(|&count| {
            let diff = count as f64 - expected;
            diff * diff / expected
        })
        .sum()
}
```

---

## Protocol Security Testing

### Replay Attack Prevention

**Test code:**
```rust
#[tokio::test]
async fn test_replay_attack_rejected() {
    let (mut sender, mut receiver) = create_test_session_pair().await;

    // Send legitimate message
    let message = b"hello";
    sender.send(message).await.unwrap();

    // Capture encrypted packet
    let packet = capture_last_packet(&sender).await;

    // Receiver processes message
    let received = receiver.recv().await.unwrap();
    assert_eq!(received, message);

    // Attempt to replay captured packet
    inject_packet(&receiver, packet).await;

    // Should reject replay
    match receiver.recv().await {
        Err(SessionError::ReplayAttack) => {
            // Expected
        }
        Ok(_) => panic!("Replay attack not detected!"),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}
```

### Man-in-the-Middle (MITM) Attack

**Test code:**
```rust
#[tokio::test]
async fn test_mitm_attack_detected() {
    let alice_keypair = Keypair::generate();
    let bob_keypair = Keypair::generate();
    let mallory_keypair = Keypair::generate();

    // Alice tries to connect to Bob
    let alice_transport = UdpTransport::bind("127.0.0.1:0".parse().unwrap()).await.unwrap();
    let bob_addr = "127.0.0.1:41641".parse().unwrap();

    // Mallory intercepts and impersonates Bob
    let mallory_transport = UdpTransport::bind(bob_addr).await.unwrap();

    // Alice initiates handshake (expects Bob's key)
    let alice_result = Session::connect_with_expected_peer(
        alice_keypair,
        bob_keypair.public(),  // Expects Bob
        bob_addr,
        alice_transport,
    ).await;

    // Mallory responds (but uses Mallory's key)
    let mallory_session = Session::accept(
        mallory_keypair,
        mallory_transport,
    ).await.unwrap();

    // Alice should reject (key mismatch)
    assert!(matches!(alice_result, Err(SessionError::PeerKeyMismatch)));
}
```

### Message Ordering

**Test code:**
```rust
#[tokio::test]
async fn test_out_of_order_messages() {
    let (mut sender, mut receiver) = create_test_session_pair().await;

    // Send multiple messages
    sender.send(b"message 1").await.unwrap();
    sender.send(b"message 2").await.unwrap();
    sender.send(b"message 3").await.unwrap();

    // Capture packets
    let packet1 = capture_packet(&sender, 0).await;
    let packet2 = capture_packet(&sender, 1).await;
    let packet3 = capture_packet(&sender, 2).await;

    // Inject out of order: 3, 1, 2
    inject_packet(&receiver, packet3).await;
    inject_packet(&receiver, packet1).await;
    inject_packet(&receiver, packet2).await;

    // Receiver should reorder or buffer
    let recv1 = receiver.recv().await.unwrap();
    let recv2 = receiver.recv().await.unwrap();
    let recv3 = receiver.recv().await.unwrap();

    assert_eq!(recv1, b"message 1");
    assert_eq!(recv2, b"message 2");
    assert_eq!(recv3, b"message 3");
}
```

---

## Privacy Testing

### Traffic Analysis Resistance

**Test: Packet size distribution**
```rust
#[test]
fn test_packet_size_uniformity() {
    let mut packet_sizes = Vec::new();

    // Send various message sizes
    for size in [10, 100, 500, 1000] {
        let message = vec![0u8; size];
        let packet = encrypt_and_frame(&message);
        packet_sizes.push(packet.len());
    }

    // Verify all packets have uniform size (with padding)
    let expected_size = 1472;  // MTU
    for size in packet_sizes {
        assert_eq!(size, expected_size, "Packet size varies, leaks info!");
    }
}
```

**Test: Timing analysis resistance**
```rust
#[tokio::test]
async fn test_timing_obfuscation() {
    let mut intervals = Vec::new();

    let start = Instant::now();
    for _ in 0..100 {
        send_with_cover_traffic().await;
        intervals.push(start.elapsed());
    }

    // Verify inter-packet timing has low correlation
    let correlation = autocorrelation(&intervals);
    assert!(correlation < 0.1, "Timing patterns detectable!");
}
```

### DHT Query Unlinkability

**Test code:**
```rust
#[tokio::test]
async fn test_dht_query_unlinkability() {
    let group_secret1 = [1u8; 32];
    let group_secret2 = [2u8; 32];
    let file_hash = Blake3Hash::hash(b"file");

    // Derive DHT keys
    let dht_key1 = derive_dht_key(&group_secret1, &file_hash);
    let dht_key2 = derive_dht_key(&group_secret2, &file_hash);

    // Keys should appear random and uncorrelated
    assert_ne!(dht_key1, dht_key2);

    // Statistical test: Hamming distance ~50%
    let hamming = hamming_distance(&dht_key1, &dht_key2);
    let expected = dht_key1.len() * 8 / 2;
    assert!((hamming as i32 - expected as i32).abs() < 20,
        "DHT keys not sufficiently random");
}
```

---

## Denial of Service (DoS) Testing

### Resource Exhaustion

**Test: Memory exhaustion**
```rust
#[tokio::test]
async fn test_memory_exhaustion_resistance() {
    let listener = create_test_listener().await;

    // Attempt to create excessive sessions
    let mut sessions = Vec::new();
    for _ in 0..10000 {
        let session = listener.accept_with_limit().await;
        if session.is_err() {
            break;  // Limit reached
        }
        sessions.push(session.unwrap());
    }

    // Verify limit enforced
    assert!(sessions.len() < 1000, "No session limit!");

    // Verify memory bounded
    let memory = get_memory_usage();
    assert!(memory < 500_000_000, "Excessive memory usage!");  // 500 MB
}
```

**Test: Handshake flood**
```rust
#[tokio::test]
async fn test_handshake_flood_mitigation() {
    let responder = create_test_responder().await;

    // Send many handshake initiations
    let mut handles = Vec::new();
    for _ in 0..1000 {
        let handle = tokio::spawn(async move {
            send_handshake_init().await
        });
        handles.push(handle);
    }

    // Wait briefly
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Verify responder still functional
    let session = establish_legitimate_session(&responder).await;
    assert!(session.is_ok(), "Responder DoS'd!");
}
```

### Amplification Attack Prevention

**Test code:**
```rust
#[test]
fn test_no_amplification() {
    // Send minimal handshake message
    let request = create_handshake_init();
    let request_size = request.len();

    // Receive response
    let response = process_handshake_init(&request);
    let response_size = response.len();

    // Verify amplification factor < 2
    let amplification = response_size as f64 / request_size as f64;
    assert!(amplification < 2.0, "Amplification attack possible!");
}
```

---

## Fuzzing

### Frame Parser Fuzzing

**Fuzz target:**
```rust
// fuzz/fuzz_targets/frame_parser.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use wraith_core::Frame;

fuzz_target!(|data: &[u8]| {
    // Should never panic or crash
    let _ = Frame::parse(data);
});
```

**Run fuzzer:**
```bash
cargo fuzz run frame_parser -- -max_total_time=3600 -dict=fuzz/dict/frames.dict
```

### Crypto Fuzzing

**Fuzz target:**
```rust
// fuzz/fuzz_targets/crypto.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use wraith_crypto::SymmetricKeys;

fuzz_target!(|data: &[u8]| {
    let mut keys = SymmetricKeys::new_test();

    // Attempt decryption of arbitrary data
    let _ = keys.decrypt(data);

    // Should never:
    // - Panic
    // - Read out of bounds
    // - Leak secrets
});
```

**Coverage tracking:**
```bash
# Build with coverage
cargo fuzz coverage frame_parser

# View coverage report
cargo cov -- show target/*/release/frame_parser \
    -instr-profile=fuzz/coverage/frame_parser/coverage.profdata \
    --format=html > coverage.html
```

---

## Penetration Testing

### Attack Scenarios

**Scenario 1: Malicious peer**
```
Attacker role: Malicious file sender
Goal: Crash receiver or leak information
Techniques:
  - Send malformed chunks
  - Send excessive data
  - Attempt buffer overflow
  - Replay old chunks
```

**Scenario 2: Network adversary**
```
Attacker role: Man-in-the-middle
Goal: Decrypt traffic or impersonate peer
Techniques:
  - Intercept handshake
  - Downgrade attack
  - Reflection attack
  - Timing analysis
```

**Scenario 3: DHT pollution**
```
Attacker role: Malicious DHT node
Goal: Disrupt file discovery
Techniques:
  - Store incorrect peer info
  - Eclipse attack (surround target)
  - Sybil attack (many fake nodes)
```

### Security Audit Checklist

**Cryptography:**
- [ ] All crypto operations constant-time
- [ ] Keys properly zeroized
- [ ] RNG from secure source (OsRng)
- [ ] No weak cipher suites
- [ ] Forward secrecy enabled
- [ ] Replay protection active

**Protocol:**
- [ ] Mutual authentication enforced
- [ ] No amplification attacks
- [ ] Resource limits enforced
- [ ] Input validation comprehensive
- [ ] No information leaks in errors

**Privacy:**
- [ ] Packet sizes uniform
- [ ] Timing obfuscation enabled
- [ ] DHT queries unlinkable
- [ ] No metadata leakage

---

## Automated Security Scanning

### Dependency Audit

```bash
# Install cargo-audit
cargo install cargo-audit

# Scan for vulnerabilities
cargo audit

# Fail on any vulnerabilities
cargo audit --deny warnings
```

### Static Analysis

```bash
# Install cargo-clippy
rustup component add clippy

# Run security lints
cargo clippy -- \
    -W clippy::all \
    -W clippy::pedantic \
    -W clippy::nursery \
    -W clippy::cargo \
    -D warnings
```

### Memory Safety

```bash
# Run with AddressSanitizer
RUSTFLAGS="-Z sanitizer=address" cargo test --target x86_64-unknown-linux-gnu

# Run with MemorySanitizer
RUSTFLAGS="-Z sanitizer=memory" cargo test --target x86_64-unknown-linux-gnu
```

---

## Continuous Security Testing

### CI Integration

```yaml
# .github/workflows/security.yml
name: Security Tests

on: [push, pull_request]

jobs:
  security:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Dependency Audit
        run: |
          cargo install cargo-audit
          cargo audit --deny warnings

      - name: Security Clippy
        run: cargo clippy -- -D warnings

      - name: Constant-Time Tests
        run: cargo test constant_time --release

      - name: Fuzzing
        run: |
          cargo install cargo-fuzz
          cargo fuzz run frame_parser -- -max_total_time=300
```

---

## See Also

- [Testing Strategy](testing-strategy.md)
- [Performance Benchmarks](performance-benchmarks.md)
- [Security Model](../architecture/security-model.md)
