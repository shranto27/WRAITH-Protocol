# WRAITH-RedOps Reference Architecture

**Document Version:** 1.3.0 (Technical Deep Dive)
**Last Updated:** 2025-11-29
**Classification:** Reference Architecture
**Governance:** See [Security Testing Parameters](../../../ref-docs/WRAITH-Security-Testing-Parameters-v1.0.md)

---

## 1. Executive Summary

WRAITH-RedOps is a comprehensive Adversary Emulation Platform designed for authorized Red Team engagements. It provides a secure, resilient Command and Control (C2) infrastructure that leverages the WRAITH protocol's intrinsic stealth capabilities.

The platform consists of three primary components:
1.  **Team Server:** A multi-user collaboration hub managing state and tasking.
2.  **Operator Client:** A cross-platform GUI for campaign management.
3.  **Spectre Implant:** A modular, memory-resident agent ("Beacon") designed for stealth and evasion.

**Authorized Use Cases Only:**
- Executive-authorized red team exercises.
- Purple team collaborative assessments.
- Adversary emulation with defined objectives.

---

## 2. System Architecture

### 2.1 Component Topology

```mermaid
graph TD
    subgraph "Operator Network (Safe Zone)"
        Client[Operator Client (Tauri)]
        Dev[DevOps / Builder]
    end

    subgraph "C2 Infrastructure (Cloud/Redirectors)"
        TS[Team Server (PostgreSQL + WRAITH)]
        Red_UDP[UDP Redirector]
        Red_HTTP[HTTPS Redirector]
        Red_DNS[DNS Redirector]
    end

    subgraph "Target Network (Compromised)"
        Beacon_A[Spectre Implant (Gateway)]
        Beacon_B[Spectre Implant (SMB Peer)]
        Beacon_C[Spectre Implant (TCP Peer)]
    end

    Client <-->|gRPC/TLS| TS
    Dev -->|Build Artifacts| TS
    
    TS <-->|WRAITH Tunnel| Red_UDP
    TS <-->|HTTPS Tunnel| Red_HTTP
    
    Red_UDP <-->|WRAITH/UDP| Beacon_A
    Red_HTTP <-->|HTTPS| Beacon_A
    
    Beacon_A <-->|SMB Pipe| Beacon_B
    Beacon_B <-->|TCP Socket| Beacon_C
```

### 2.2 Component Descriptions

#### A. Operator Console (Client)
*   **Purpose:** Centralized management interface for red team operators.
*   **UI:** Tauri (Rust backend) + React (Frontend).
*   **Capabilities:**
    *   **Session Management:** Real-time interactive terminal for each beacon.
    *   **Graph View:** Visualizes the peer-to-peer graph of beacons.
    *   **Campaign Management:** Organization of engagement activities.

#### B. Team Server (Backend)
*   **Purpose:** The brain of the operation. Manages state, tasking, and data aggregation.
*   **Architecture:** Rust (`axum`) with PostgreSQL.
*   **Listener Bus:** Manages multiple listening ports (UDP, TCP, HTTP) and routes traffic to specific sessions.
*   **Builder:** Compiles unique implant artifacts per campaign using a patched LLVM toolchain.

#### C. "Spectre" Implant (Agent)
*   **Purpose:** The deployed agent executing on target systems.
*   **Design:** `no_std` Rust binary (freestanding). Zero runtime dependencies (no libc/msvcrt).
*   **Memory Model:** Position Independent Code (PIC). Can be injected as Shellcode (sRDI), DLL, or EXE.
*   **Stealth Features:**
    *   **Sleep Mask:** Obfuscates memory during sleep intervals.
    *   **Stack Spoofing:** Rewrites call stack frames to look legitimate.
    *   **Indirect Syscalls:** Bypasses user-mode hooks (EDR).

#### D. Governance Layer
*   **Purpose:** Enforce engagement parameters and maintain accountability.
*   **Controls:**
    *   **Scope Enforcement:** Target whitelist/blacklist checks kernel-side in the implant.
    *   **Time-to-Live (TTL):** Implants self-destruct after a specific date.
    *   **Audit Logging:** Immutable logs of every command sent.

---

## 3. Operational Workflow

```
┌──────────────────────────────────────────────────────────────────────────┐
│                      Red Team Engagement Workflow                        │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │ PHASE 1: Pre-Engagement                                          │    │
│  │                                                                  │    │
│  │  • Authorization acquisition (executive sign-off)                │    │
│  │  • Scope definition and documentation                            │    │
│  │  • Infrastructure preparation (Redirectors, C2 Domains)          │    │
│  │  • Payload Generation (Builder)                                  │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                  │                                      │
│                                  ▼                                      │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │ PHASE 2: Operations                                              │    │
│  │                                                                  │    │
│  │  • Access: Initial Access vectors (Phishing, Exploit)            │    │
│  │  • Establish: Beacon check-in, key exchange                      │    │
│  │  • Persistence: Maintain access across restarts                  │    │
│  │  • Lateral Movement: SMB/TCP Peer-to-Peer chaining               │    │
│  │  • Objectives: Data staging, exfiltration (via WRAITH-Recon)     │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                  │                                      │
│                                  ▼                                      │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │ PHASE 3: Post-Engagement                                         │    │
│  │                                                                  │    │
│  │  • Operations cessation                                          │    │
│  │  • Cleanup: Remove artifacts, revoke keys                        │    │
│  │  • Reporting: Generate Timeline, Finding Report                  │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

---

## 4. WRAITH Protocol Integration

### 4.1 C2 Channel Cryptography

**Algorithm Suite (Same as Core Protocol):**
| Function | Algorithm | Security Level | C2 Application |
|----------|-----------|----------------|----------------|
| Key Exchange | X25519 | 128-bit | Beacon-to-Server authentication |
| Key Encoding | Elligator2 | N/A | Traffic indistinguishability |
| AEAD | XChaCha20-Poly1305 | 256-bit key, 128-bit auth | Command/response encryption |
| Hash | BLAKE3 | 128-bit collision resistance | Integrity, fingerprinting |
| KDF | HKDF-BLAKE3 | 128-bit | Session key derivation |

**Noise_XX Handshake for C2:**
```
Beacon (Initiator)         Team Server (Responder)
    |                              |
    |  -> e (96 bytes)             |  Phase 1: Ephemeral key (Elligator2)
    |                              |
    |  <- e, ee, s, es (128 bytes) |  Phase 2: Server static key revealed
    |                              |
    |  -> s, se (80 bytes)         |  Phase 3: Beacon static key revealed
    |                              |
    |  [Mutually Authenticated]    |  Result: Forward-secret C2 channel
```

**Session Key Derivation:**
```rust
// After Noise handshake
let handshake_hash = noise_state.get_handshake_hash();
let prk = HKDF_Extract(salt: "wraith-c2-v1", ikm: DH_outputs);

// Derive separate keys for each direction
let beacon_tx_key = HKDF_Expand(prk, "beacon-tx", 32);
let beacon_rx_key = HKDF_Expand(prk, "beacon-rx", 32);
let conn_id = HKDF_Expand(prk, "c2-conn-id", 8);
```

### 4.2 Wire Protocol Details

**Outer WRAITH Packet (Network Layer):**
```
├─ Connection ID (8 bytes) - Stable identifier for C2 session
├─ Encrypted C2 Payload (variable) - XChaCha20-Poly1305 ciphertext
└─ Authentication Tag (16 bytes) - Poly1305 MAC
Total overhead: 24 bytes minimum
```

**Inner WRAITH Frame (After AEAD Decryption):**
```
├─ Nonce (8 bytes) - Session salt || Packet counter
├─ Frame Type (1 byte) - DATA for C2 messages
├─ Flags (1 byte) - SYN (initial check-in), ACK, FIN (beacon exit)
├─ Stream ID (2 bytes) - Multiplexed task channels
├─ Sequence Number (4 bytes) - Per-task ordering
├─ File Offset (8 bytes) - For file uploads/downloads
├─ Payload Length (2 bytes) - C2 message size
├─ Reserved (2 bytes) - Future protocol extensions
├─ C2 Message (variable) - Nested RedOps protocol
└─ Padding (variable) - Traffic shaping
Header size: 28 bytes fixed
```

**Frame Types Used in C2:**
| Type | Value | C2 Usage |
|------|-------|----------|
| DATA | 0x01 | Task commands, beacon responses |
| ACK | 0x02 | Task acknowledgment |
| CONTROL | 0x03 | Session management (sleep, exit) |
| REKEY | 0x04 | Initiate DH ratchet |
| PING/PONG | 0x05/0x06 | Keepalive, latency measurement |
| PAD | 0x08 | Cover traffic during idle |
| STREAM_OPEN | 0x09 | New task stream |
| STREAM_CLOSE | 0x0A | Task completion |
| PATH_CHALLENGE | 0x0E | Connection migration probe |

### 4.3 Transport Layer Options

**Primary Transport (WRAITH/UDP):**
```rust
use wraith_transport::{Transport, TransportConfig};

let config = TransportConfig {
    mode: TransportMode::Udp {
        bind_addr: "0.0.0.0:0".parse().unwrap(),
    },
    buffer_size: 65536,
    timeout: Duration::from_secs(30),
};

let transport = Transport::new(config)?;
```

**Fallback Transports:**
- **HTTPS:** WRAITH frames wrapped in TLS 1.3, HTTP/2 POST requests
- **DNS:** Tunneled via TXT records (Base32-encoded WRAITH frames)
- **SMB:** Named pipes for peer-to-peer lateral movement
- **ICMP:** Echo Request/Reply padding field

**Transport Selection Logic:**
```rust
match beacon_config.channel_priority {
    vec!["wraith-udp", "https", "dns"] => {
        // Try WRAITH UDP first
        if let Ok(conn) = try_wraith_udp().await {
            return conn;
        }
        // Fall back to HTTPS
        if let Ok(conn) = try_https().await {
            return conn;
        }
        // Last resort: DNS tunneling
        try_dns_tunnel().await?
    }
}
```

### 4.4 Obfuscation for C2 Traffic

**Elligator2 Key Encoding:**
- All beacon ephemeral keys appear as random bytes
- Prevents DPI from identifying X25519 key exchanges
- ~50% encoding success rate (acceptable for beaconing)

**Beaconing Obfuscation:**
```rust
// Exponential jitter for check-in intervals
let base_interval = 60; // seconds
let jitter_percent = 20;
let delay = base_interval as f64 * (1.0 + (random::<f64>() - 0.5) * (jitter_percent as f64 / 100.0));

sleep(Duration::from_secs_f64(delay)).await;
```

**Packet Padding for C2:**
- **Performance Mode:** Minimal padding (next size class)
- **Stealth Mode:** Match HTTPS distribution (64B, 256B, 512B, 1024B, 1472B)
- **Bandwidth:** ~5-15% overhead depending on mode

**Protocol Mimicry:**
```rust
use wraith_obfuscation::mimicry::{MimicryProfile, TlsWrapper};

// Wrap C2 traffic in TLS 1.3
let wrapper = TlsWrapper::new(MimicryProfile::Tls13 {
    ja3_fingerprint: "771,4865-4866-4867,0-23-65281,29-23-24,0", // Chrome
    sni: "c2.legitimate-cdn.com",
    alpn: vec!["h2".to_string()],
});

let disguised_c2 = wrapper.wrap_frame(&c2_frame)?;
```

### 4.5 Ratcheting for Long-Duration Operations

**Symmetric Ratchet (Per-Packet):**
```rust
// After sending/receiving each C2 message
chain_key_next = BLAKE3(chain_key_current || 0x01);
message_key = BLAKE3(chain_key_current || 0x02);

// Immediate zeroization to prevent memory forensics
zeroize(&mut chain_key_current);
zeroize(&mut message_key);
```

**DH Ratchet (Periodic):**
- **Trigger:** Every 2 minutes OR 1,000,000 packets
- **Beacon:** Generates new ephemeral key, sends REKEY frame
- **Server:** Responds with new ephemeral, performs DH
- **Result:** New chain key derived, forward secrecy restored

**Post-Compromise Security:**
```
T=0:     Adversary compromises current session keys
T=1min:  Normal beacon check-in (adversary can decrypt)
T=2min:  DH ratchet triggered
T=2min+: Adversary loses decryption capability (new DH secret)
```

### 4.6 Performance Characteristics

**Transport Modes:**
| Mode | Throughput | Latency | Overhead |
|------|------------|---------|----------|
| WRAITH/UDP | 300+ Mbps | 10-50ms | 24 bytes/packet |
| WRAITH/AF_XDP | 10-40 Gbps | <1ms | 24 bytes/packet |
| HTTPS Wrapper | 200+ Mbps | 20-100ms | 5% bandwidth |
| DNS Tunnel | 10-50 Kbps | 100-500ms | 80% bandwidth |

**Thread Model:**
- Thread-per-beacon architecture
- Lock-free task queue
- NUMA-aware allocation for multi-socket servers

---

## 4. C2 Protocol Specification

### 4.1 Transport Layer
*   **Primary:** WRAITH Protocol (UDP/Noise_XX). Provides encryption, authentication, and NAT traversal.
*   **Fallback:** HTTPS (TLS 1.3), DNS (DoH), SMB (Named Pipes).

### 4.2 Presentation Layer (C2 Payload)
Encapsulated *inside* the transport layer.
*   **Header:** `[Magic:4][SessionID:4][TaskID:4][Opcode:2][Length:4]`
*   **Payload:** Protobuf-serialized data (Commands or Results).
*   **Encryption:** Inner layer Chacha20-Poly1305 (Session Key) + Outer layer Transport Encryption.

### 4.3 Protocol Data Unit (PDU) Definitions
We use Google Protocol Buffers (proto3) for defining the C2 schema.

```protobuf
syntax = "proto3";

message BeaconTask {
    uint32 task_id = 1;
    CommandType command = 2;
    bytes arguments = 3; // Serialized args specific to command
}

enum CommandType {
    SLEEP = 0;
    SHELL = 1;
    UPLOAD = 2;
    DOWNLOAD = 3;
    EXECUTE_BOF = 4;
    INJECT = 5;
    EXIT = 99;
}

message BeaconResponse {
    uint32 task_id = 1;
    uint32 status_code = 2; // 0 = Success
    bytes output = 3;
    string error_msg = 4;
}
```

---

## 5. Data Structures & Schema

### 5.1 Team Server Database (PostgreSQL)

```sql
CREATE TABLE listeners (
    id SERIAL PRIMARY KEY,
    name VARCHAR(64) UNIQUE NOT NULL,
    type VARCHAR(16) NOT NULL, -- UDP, HTTP, SMB
    bind_address INET NOT NULL,
    config JSONB NOT NULL
);

CREATE TABLE beacons (
    id CHAR(16) PRIMARY KEY, -- Random Hex ID
    internal_ip INET,
    external_ip INET,
    hostname VARCHAR(255),
    user_name VARCHAR(255),
    process_id INT,
    arch VARCHAR(8), -- x64, x86
    linked_beacon_id CHAR(16) REFERENCES beacons(id), -- Parent for P2P
    last_seen TIMESTAMP WITH TIME ZONE,
    status VARCHAR(16) -- ALIVE, DEAD, EXITING
);

CREATE TABLE tasks (
    id SERIAL PRIMARY KEY,
    beacon_id CHAR(16) REFERENCES beacons(id),
    command_type INT NOT NULL,
    arguments BYTEA,
    queued_at TIMESTAMP DEFAULT NOW(),
    sent_at TIMESTAMP,
    completed_at TIMESTAMP,
    result_output BYTEA,
    operator_id INT REFERENCES users(id)
);
```

---

## 6. Detection Considerations

To support defensive improvement, WRAITH-RedOps produces detectable artifacts:

### Network Indicators
| Indicator Type | Description | Detection Approach |
|----------------|-------------|-------------------|
| **Beaconing** | Periodic communications | Interval analysis / Jitter analysis |
| **Data Patterns** | Unusual traffic volumes | Baseline deviation |
| **Certificate Analysis** | TLS certificate properties | Certificate transparency |

### Endpoint Indicators
| Indicator Type | Description | Detection Approach |
|----------------|-------------|-------------------|
| **Process Ancestry** | Unusual process relationships | EDR process tracking |
| **Memory Indicators** | Unbacked RWX pages (if sleep mask fails) | Memory scanning (Moneta) |
| **Named Pipes** | Abnormal pipe names (e.g., `\pipe\msagent_12`) | Sysmon Event ID 17/18 |

---

## 7. Audit and Accountability

### 7.1 Logging Requirements
| Log Category | Contents | Integrity |
|--------------|----------|-----------|
| **Operator Log** | All operator actions | Signed, attributed |
| **Communications Log** | Channel activity | Cryptographic chain |
| **Operations Log** | Task execution details | Append-only, encrypted |

### 7.2 Chain of Custody
1.  **Executive Authorization:** Signed authorization blob.
2.  **WRAITH-RedOps Platform:** Enforces constraints, logs all activity.
3.  **Audit Trail:** Available for review, incident response, legal.

---

## 8. Deployment Considerations

### Prerequisites
*   **Signed Rules of Engagement document.**
*   **Scope configuration file.**
*   **Operator credentials.**
*   **Kill switch endpoint configuration.**

### Infrastructure Requirements
*   **Team Server:** Hardened Linux VPS (4 vCPU, 8GB RAM).
*   **Redirectors:** Ephemeral VPS instances (dumb pipes).
*   **Domains:** Categorized/Aged domains for HTTP/DNS C2.