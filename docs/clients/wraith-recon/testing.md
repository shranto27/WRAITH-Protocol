# WRAITH-Recon Testing Strategy

**Document Version:** 1.0.0
**Last Updated:** 2025-11-29
**Governance:** See [Security Testing Parameters](../../../ref-docs/WRAITH-Security-Testing-Parameters-v1.0.md)

---

## 1. Overview
This document outlines the validation strategy for WRAITH-Recon. Given the tool's capability to generate high-velocity network traffic and its use in sensitive engagements, testing must rigorously verify **safety**, **stealth**, and **correctness**.

---

## 2. Safety Verification (The "Kill Switch" Tests)
These tests ensure the tool **never** violates the Rules of Engagement (RoE).

### 2.1 Governance Unit Tests
*   **Goal:** Verify the `SafetyController` logic.
*   **Method:** Property-based testing (using `proptest`).
*   **Scenarios:**
    *   **CIDR Boundary:** Generate random IPs; assert `check()` only returns true if IP is in allowlist.
    *   **Blacklist Priority:** Generate IP present in *both* allowlist and blacklist; assert `check()` returns false.
    *   **Expiry:** Mock system clock to `expiry + 1s`; assert `check()` returns false.
    *   **Kill Switch:** Set `KILL_SWITCH` atomic; assert `check()` returns false for *any* IP.

### 2.2 Fuzzing the Configuration Parser
*   **Goal:** Ensure malformed or malicious config files cannot crash the agent or bypass checks.
*   **Tool:** `cargo-fuzz`.
*   **Targets:** `Config::parse()`, `Signature::verify()`.

---

## 3. Protocol Correctness Testing

### 3.1 Cryptographic Verification

**Noise_XX Handshake:**
*   **Goal:** Verify 3-phase handshake completes successfully
*   **Test Cases:**
    1. Valid mutual authentication
    2. MITM detection (invalid static key)
    3. Replay attack resistance
    4. Nonce uniqueness verification

**AEAD Encryption/Decryption:**
```rust
#[test]
fn test_xchacha20_poly1305_roundtrip() {
    let key = [0x42u8; 32];
    let nonce = [0x13u8; 24];
    let plaintext = b"reconnaissance data";

    let cipher = XChaCha20Poly1305::new(&key);
    let ciphertext = cipher.encrypt(&nonce, plaintext).unwrap();

    // Verify 16-byte auth tag appended
    assert_eq!(ciphertext.len(), plaintext.len() + 16);

    let decrypted = cipher.decrypt(&nonce, &ciphertext).unwrap();
    assert_eq!(decrypted, plaintext);
}
```

**Elligator2 Encoding:**
*   **Goal:** Verify encoded keys are statistically indistinguishable from random
*   **Method:** Chi-squared test on 10,000 encoded public keys
*   **Pass Criteria:** p-value > 0.05 (cannot reject random hypothesis)

**Ratcheting Correctness:**
```rust
#[test]
fn test_symmetric_ratchet() {
    let mut chain_key = [0x00u8; 32];

    for i in 0..1_000_000 {
        let message_key = derive_message_key(&chain_key);
        chain_key = derive_next_chain_key(&chain_key);

        // Verify zeroization
        assert_all_zeros(&message_key);

        if i % 100_000 == 0 {
            // Verify DH ratchet trigger
            assert!(should_dh_ratchet(i, elapsed_time));
        }
    }
}
```

### 3.2 Wire Format Validation

**Frame Encoding/Decoding:**
```rust
#[test]
fn test_frame_structure() {
    let frame = Frame {
        nonce: [0x11; 8],
        frame_type: FrameType::DATA,
        flags: Flags::ACK,
        stream_id: 42,
        sequence: 1337,
        file_offset: 0,
        payload_length: 256,
        payload: vec![0xAA; 256],
        padding: vec![0x00; 128],
    };

    let encoded = frame.encode();

    // Verify header is exactly 28 bytes
    assert_eq!(&encoded[0..28].len(), 28);

    // Verify frame decodes correctly
    let decoded = Frame::decode(&encoded).unwrap();
    assert_eq!(decoded.frame_type, FrameType::DATA);
    assert_eq!(decoded.payload_length, 256);
}
```

**Connection ID Derivation:**
*   Verify CID is derived from handshake transcript
*   Verify CID uniqueness across sessions
*   Verify CID is exactly 8 bytes

### 3.3 Obfuscation Verification

**Padding Distribution Test:**
```rust
#[test]
fn test_stealth_padding_distribution() {
    let padder = Padder::new(PaddingMode::Stealth);
    let mut size_counts = HashMap::new();

    for _ in 0..10_000 {
        let padded = padder.pad(&[0x00; 100]);
        *size_counts.entry(padded.len()).or_insert(0) += 1;
    }

    // Verify distribution matches spec (64B: 10%, 256B: 15%, etc.)
    assert_approx_eq!(size_counts[&64] as f64 / 10_000.0, 0.10, epsilon = 0.02);
    assert_approx_eq!(size_counts[&256] as f64 / 10_000.0, 0.15, epsilon = 0.02);
}
```

**Timing Jitter Test:**
*   Measure inter-packet delays for 1000 packets
*   Verify exponential distribution (Kolmogorov-Smirnov test)
*   Assert no deterministic patterns

---

## 3. Network Capability Testing

### 3.1 AF_XDP Loopback Test
*   **Goal:** Verify kernel-bypass read/write without physical network hardware.
*   **Setup:** Use `veth` pairs in a network namespace.
*   **Procedure:**
    1.  Create namespace `ns1`.
    2.  Bind WRAITH-Recon to `veth0`.
    3.  Run `tcpdump` on `veth1`.
    4.  Send 1M packets.
    5.  Verify packet count and data integrity.

### 3.2 Throughput Benchmark
*   **Goal:** Validate 10Gbps capability.
*   **Environment:** Bare-metal Linux server with Intel X520/X710 NIC.
*   **Metric:** Packet Per Second (PPS) vs CPU usage.
*   **Pass Criteria:** > 10M PPS at < 50% CPU on 1 core.

---

## 4. Detection & Stealth Testing

### 4.1 Blue Team Simulation
*   **Goal:** Verify effectiveness of Obfuscation profiles.
*   **Setup:**
    *   **Attacker:** WRAITH-Recon running "Stealth Scan".
    *   **Defender:** Snort/Suricata with "ET Open" and "Snort Subscriber" rulesets.
*   **Scenarios:**
    1.  **Baseline:** Run `nmap -sS -T4`. Expect: Detection.
    2.  **Test:** Run `wraith-recon --profile stealth`. Expect: No Alerts.
    3.  **Test:** Run `wraith-recon --profile jitter-pareto`. Expect: No Behavioral Alerts.

### 4.2 Mimicry Validation
*   **Goal:** Ensure "DNS" traffic looks like DNS.
*   **Tool:** Wireshark / `tshark`.
*   **Procedure:**
    1.  Capture generated traffic.
    2.  Run `tshark -r capture.pcap -V`.
    3.  Assert: No "Malformed Packet" errors.
    4.  Assert: All fields (Flags, Opcode, RCODE) match RFC 1035.

---

## 5. Integration Testing

### 5.1 End-to-End Exfiltration
*   **Setup:**
    *   Client: WRAITH-Recon (Exfil Mode).
    *   Server: WRAITH-Listener (with reassembly logic).
*   **Procedure:**
    1.  Generate 100MB random file.
    2.  Exfiltrate via DNS Tunnel.
    3.  Compare SHA256 of source and destination files.
    *   **Pass Criteria:** Hashes match.

---

## 6. CI/CD Pipeline

*   **Stage 1: Static Analysis** (`cargo clippy`, `cargo fmt`, `audit`).
*   **Stage 2: Unit Tests** (`cargo test`).
*   **Stage 3: Safety Tests** (Governance logic verification).
*   **Stage 4: Build** (Release binary generation).
*   **Stage 5: Artifact Signing** (Sign binary with dev key).
