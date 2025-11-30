# Protocol Technical Details

## Decentralized Secure File Transfer Protocol Specification

**Document Version:** 1.0.0-DRAFT  
**Status:** Technical Specification  
**Classification:** Implementation Reference  
**Target Platform:** Linux 6.x (kernel 6.2+)  
**Primary Language:** Rust (2021 Edition)  

---

## Table of Contents

1. [Overview and Design Philosophy](#1-overview-and-design-philosophy)
2. [Wire Protocol Specification](#2-wire-protocol-specification)
3. [Cryptographic Protocol Design](#3-cryptographic-protocol-design)
4. [Frame Types and Formats](#4-frame-types-and-formats)
5. [State Machine Definitions](#5-state-machine-definitions)
6. [Traffic Obfuscation Mechanisms](#6-traffic-obfuscation-mechanisms)
7. [Discovery Protocol](#7-discovery-protocol)
8. [NAT Traversal Protocol](#8-nat-traversal-protocol)
9. [Congestion Control](#9-congestion-control)
10. [Error Handling and Recovery](#10-error-handling-and-recovery)
11. [Security Properties and Threat Model](#11-security-properties-and-threat-model)
12. [Protocol Constants and Parameters](#12-protocol-constants-and-parameters)

---

## 1. Overview and Design Philosophy

### 1.1 Core Design Principles

This protocol specification defines a novel decentralized file sharing system optimized for high-throughput (300+ Mbps), low-latency operation across all file sizes while maintaining strong security guarantees and traffic analysis resistance. The design follows these principles:

**Zero-Trust Architecture:** All communications assume hostile network environments. Every packet is authenticated and encrypted. No metadata leaks plaintext.

**Kernel-Accelerated Data Path:** The hot path bypasses the kernel network stack entirely using AF_XDP and io_uring, achieving theoretical throughput of 10-40 Gbps on commodity hardware.

**Indistinguishability by Design:** All protocol traffic is computationally indistinguishable from uniform random data. Handshakes use Elligator2-encoded key exchange; payloads use AEAD with random padding.

**Forward-Secure by Default:** Session keys are ephemeral with mandatory ratcheting. Compromise of current keys reveals nothing about past sessions.

**Stateless Recovery:** The protocol tolerates packet loss, reordering, and connection migration without complex state synchronization.

### 1.2 Protocol Layers

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        Application Layer                                │
│   File Transfer API │ Chunk Management │ Integrity Verification         │
├─────────────────────────────────────────────────────────────────────────┤
│                        Session Layer                                    │
│   Stream Multiplexing │ Flow Control │ Congestion Control               │
├─────────────────────────────────────────────────────────────────────────┤
│                        Cryptographic Transport Layer                    │
│   Noise_XX Handshake │ AEAD Encryption │ Key Ratcheting                 │
├─────────────────────────────────────────────────────────────────────────┤
│                        Obfuscation Layer                                │
│   Elligator2 Encoding │ Traffic Shaping │ Padding │ Timing Jitter       │
├─────────────────────────────────────────────────────────────────────────┤
│                        Kernel Acceleration Layer                        │
│   AF_XDP Sockets │ XDP Programs │ io_uring │ Zero-Copy DMA              │
├─────────────────────────────────────────────────────────────────────────┤
│                        Network Layer                                    │
│   UDP │ Raw Sockets │ ICMP Covert Channels │ DNS Tunneling              │
└─────────────────────────────────────────────────────────────────────────┘
```

### 1.3 Terminology

| Term | Definition |
|------|------------|
| **Peer** | A node participating in the protocol |
| **Session** | An authenticated, encrypted communication channel between two peers |
| **Stream** | A logical bidirectional byte stream within a session (file transfer) |
| **Frame** | The fundamental unit of protocol data, always encrypted |
| **Connection ID (CID)** | 64-bit identifier for session demultiplexing |
| **Chunk** | A segment of file data (default: 256 KiB) |
| **Representative** | Elligator2-encoded form of an elliptic curve point |

---

## 2. Wire Protocol Specification

### 2.1 Outer Packet Format

All packets on the wire follow this format. The outer layer provides connection demultiplexing and version negotiation before decryption.

```
Outer Packet (before decryption):
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
├─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┤
│                     Connection ID (64 bits)                   │
│                                                               │
├───────────────────────────────────────────────────────────────┤
│                     Encrypted Payload                         │
│                         (variable)                            │
│                           ...                                 │
├───────────────────────────────────────────────────────────────┤
│                     Authentication Tag (128 bits)             │
│                                                               │
│                                                               │
│                                                               │
└───────────────────────────────────────────────────────────────┘

Total overhead: 8 (CID) + 16 (tag) = 24 bytes minimum
```

**Connection ID (CID):** 64-bit value derived during handshake. The high 32 bits are random (session identifier), and the low 32 bits rotate based on packet sequence to prevent tracking. CID derivation:

```
initial_cid = BLAKE3(shared_secret || "connection-id")[0..8]
rotating_cid = initial_cid[0..4] || (initial_cid[4..8] XOR seq_num)
```

**Special CID Values:**

| CID Value | Meaning |
|-----------|---------|
| `0x0000000000000000` | Reserved (invalid) |
| `0xFFFFFFFFFFFFFFFF` | Handshake initiation packet |
| `0xFFFFFFFFFFFFFFFE` | Version negotiation |
| `0xFFFFFFFFFFFFFFFD` | Stateless reset |

### 2.2 Inner Frame Format (Post-Decryption)

After AEAD decryption, the payload reveals the inner frame structure:

```
Inner Frame Format:
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
├─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┼─┤
│                        Nonce (64 bits)                        │
│                                                               │
├───────────────────────────────────────────────────────────────┤
│     Frame Type    │     Flags       │     Stream ID           │
│      (8 bits)     │    (8 bits)     │     (16 bits)           │
├───────────────────────────────────────────────────────────────┤
│                    Sequence Number (32 bits)                  │
├───────────────────────────────────────────────────────────────┤
│                    File Offset (64 bits)                      │
│                                                               │
├───────────────────────────────────────────────────────────────┤
│    Payload Length │    Reserved     │    Payload Data...      │
│     (16 bits)     │    (16 bits)    │                         │
├───────────────────────────────────────────────────────────────┤
│                    Payload Data (continued)                   │
│                         (variable)                            │
│                           ...                                 │
├───────────────────────────────────────────────────────────────┤
│                    Padding (variable, random)                 │
│                           ...                                 │
└───────────────────────────────────────────────────────────────┘

Header size: 28 bytes fixed
Maximum payload: 1428 bytes (standard MTU) or 8928 bytes (jumbo)
```

### 2.3 Field Specifications

#### 2.3.1 Nonce (64 bits)

The nonce ensures unique AEAD encryption for every packet:

```
Nonce Structure:
├─────────────────────────────────────────────────────────────────────┤
│  Session Salt (32 bits)  │  Packet Counter (32 bits)               │
├─────────────────────────────────────────────────────────────────────┤

Session Salt: Random value generated during handshake
Packet Counter: Monotonically increasing, never reused

Full AEAD Nonce (192 bits for XChaCha20):
├─────────────────────────────────────────────────────────────────────┤
│  Zero Padding (128 bits)              │  Protocol Nonce (64 bits)  │
├─────────────────────────────────────────────────────────────────────┤
```

**Counter Overflow Protection:** If the 32-bit counter approaches `2^32 - 2^20` (leaving ~1M packets headroom), the session MUST initiate rekeying. Sending with counter at `2^32 - 1` is a protocol violation.

#### 2.3.2 Frame Type (8 bits)

| Value | Type | Description |
|-------|------|-------------|
| `0x00` | RESERVED | Invalid, MUST be rejected |
| `0x01` | DATA | File data payload |
| `0x02` | ACK | Selective acknowledgment |
| `0x03` | CONTROL | Stream management |
| `0x04` | REKEY | Forward secrecy ratchet |
| `0x05` | PING | Keepalive / RTT measurement |
| `0x06` | PONG | Response to PING |
| `0x07` | CLOSE | Session termination |
| `0x08` | PAD | Cover traffic (no payload) |
| `0x09` | STREAM_OPEN | New stream initiation |
| `0x0A` | STREAM_CLOSE | Stream termination |
| `0x0B` | STREAM_RESET | Abort stream with error |
| `0x0C` | WINDOW_UPDATE | Flow control credit |
| `0x0D` | GOAWAY | Graceful shutdown |
| `0x0E` | PATH_CHALLENGE | Connection migration |
| `0x0F` | PATH_RESPONSE | Migration acknowledgment |
| `0x10-0x1F` | RESERVED | Future protocol use |
| `0x20-0x3F` | EXTENSION | Application-defined |
| `0x40-0xFF` | INVALID | MUST be rejected |

#### 2.3.3 Flags (8 bits)

```
Flags Byte:
 7   6   5   4   3   2   1   0
├───┼───┼───┼───┼───┼───┼───┼───┤
│ R │ R │ R │CMP│PRI│ACK│FIN│SYN│
└───┴───┴───┴───┴───┴───┴───┴───┘

Bit 0 (SYN): Stream synchronization / initiation
Bit 1 (FIN): Final frame in stream
Bit 2 (ACK): Acknowledgment data present
Bit 3 (PRI): Priority frame (expedited processing)
Bit 4 (CMP): Payload is compressed (LZ4)
Bits 5-7: Reserved (MUST be zero)
```

#### 2.3.4 Stream ID (16 bits)

Streams are unidirectional logical channels within a session:

| Range | Initiator | Direction |
|-------|-----------|-----------|
| `0x0000` | N/A | Session control (non-stream) |
| `0x0001-0x3FFF` | Client | Client → Server |
| `0x4000-0x7FFF` | Server | Server → Client |
| `0x8000-0xBFFF` | Client | Client → Server (expedited) |
| `0xC000-0xFFFF` | Server | Server → Client (expedited) |

Expedited streams bypass normal flow control for control messages and small files.

#### 2.3.5 Sequence Number (32 bits)

Per-stream sequence number for ordering and loss detection. Wraps at `2^32` with protection:

```
// Sequence number comparison with wrap-around handling
fn seq_lt(a: u32, b: u32) -> bool {
    let diff = a.wrapping_sub(b);
    diff > 0x80000000  // a < b if difference > 2^31
}
```

#### 2.3.6 File Offset (64 bits)

Byte offset within the file for this DATA frame. Enables:
- Random access / seeking
- Parallel chunk fetching from multiple peers
- Resume after disconnection

For non-DATA frames, this field is repurposed per frame type.

#### 2.3.7 Payload Length (16 bits)

Actual payload size in bytes (0-65535). The difference between payload length and packet size is padding.

### 2.4 Byte Ordering and Alignment

All multi-byte fields use **big-endian (network byte order)**. Frame headers are aligned to 8-byte boundaries for efficient zero-copy DMA operations. Payload data begins at offset 28 (after fixed header), which is 4-byte aligned.

### 2.5 Maximum Transmission Unit (MTU) Considerations

| Network Type | Typical MTU | Max Payload | Recommendation |
|--------------|-------------|-------------|----------------|
| Ethernet | 1500 | 1428 | Default |
| Jumbo Frames | 9000 | 8928 | High-throughput LANs |
| VPN Tunnels | 1400 | 1328 | Conservative |
| Cellular (LTE/5G) | 1500 | 1428 | Standard |
| Satellite | 512 | 440 | Constrained |

**Path MTU Discovery:** The protocol uses PLPMTUD (Packetization Layer Path MTU Discovery) per RFC 8899, sending probe packets at increasing sizes to discover the optimal MTU without relying on ICMP (which may be blocked).

---

## 3. Cryptographic Protocol Design

### 3.1 Algorithm Suite

The protocol uses a fixed, modern cryptographic suite optimized for software performance:

| Function | Algorithm | Security Level | Rationale |
|----------|-----------|---------------|-----------|
| Key Exchange | X25519 | 128-bit | Constant-time, ~25k ops/sec |
| Key Encoding | Elligator2 | N/A | Uniform random representation |
| AEAD | XChaCha20-Poly1305 | 256-bit (key) / 128-bit (auth) | 192-bit nonce, 3x faster than AES-GCM w/o AES-NI |
| Hash | BLAKE3 | 128-bit collision | 4x faster than SHA-256, tree-parallelizable |
| KDF | HKDF-BLAKE3 | 128-bit | Standard extraction/expansion |
| Signatures | Ed25519 | 128-bit | Identity verification only |

### 3.2 Noise Protocol Handshake

The protocol implements **Noise_XX** for mutual authentication with identity hiding:

```
Noise_XX Handshake Pattern:

    Initiator (I)                      Responder (R)
    ─────────────────────────────────────────────────
    s, e                               s, e
    ─────────────────────────────────────────────────
    
    Phase 1: Initiator sends ephemeral key
    ────────────────────────────────────────────────────────────────
    → e                                [32 bytes, Elligator2 encoded]
    
    Phase 2: Responder sends ephemeral + static, encrypted
    ────────────────────────────────────────────────────────────────
    ← e, ee, s, es                     [32 + 32 + 16 bytes encrypted]
    
    Phase 3: Initiator sends static, encrypted
    ────────────────────────────────────────────────────────────────
    → s, se                            [32 + 16 bytes encrypted]
    
    ═══════════════════════════════════════════════════════════════
    Transport Mode: Both parties have symmetric session keys
```

**Noise Pattern String:** The complete pattern is `Noise_XX_25519_ChaChaPoly_BLAKE2s`.

**Note on Hash Function:** The Noise protocol framework uses **BLAKE2s** (not BLAKE3) as required by the `snow` Rust library, which currently only supports BLAKE2s for Noise protocol hashing. BLAKE2s provides equivalent cryptographic security (128-bit collision resistance) and is well-suited for the Noise handshake. BLAKE3 is still used throughout the protocol for:
- HKDF key derivation
- File chunk hashing
- DHT key generation
- Symmetric key ratcheting
- Connection ID generation

Both BLAKE2s and BLAKE3 are cryptographically sound modern hash functions from the BLAKE family.

### 3.3 Handshake Message Formats

#### 3.3.1 Phase 1: Initiator Hello

```
Initiator Hello (96 bytes):
├─────────────────────────────────────────────────────────────────────┤
│  CID = 0xFFFFFFFFFFFFFFFF (8 bytes)                                │
├─────────────────────────────────────────────────────────────────────┤
│  Protocol Version (4 bytes): 0x00000001                            │
├─────────────────────────────────────────────────────────────────────┤
│  Timestamp (8 bytes): Unix epoch microseconds                       │
├─────────────────────────────────────────────────────────────────────┤
│  Initiator Ephemeral Public Key [Elligator2 encoded] (32 bytes)    │
├─────────────────────────────────────────────────────────────────────┤
│  Random Padding (28 bytes)                                          │
├─────────────────────────────────────────────────────────────────────┤
│  MAC (16 bytes): BLAKE3(entire message || responder_static_pk)     │
└─────────────────────────────────────────────────────────────────────┘
```

**Elligator2 Encoding:** The ephemeral public key MUST be encoded using Elligator2 inverse mapping. Not all curve points are encodable (~50%), so key generation loops until an encodable point is found. The high bit of the representative MUST be randomized.

#### 3.3.2 Phase 2: Responder Response

```
Responder Response (128 bytes):
├─────────────────────────────────────────────────────────────────────┤
│  Connection ID (8 bytes): Derived from ee                          │
├─────────────────────────────────────────────────────────────────────┤
│  Responder Ephemeral Public Key [Elligator2 encoded] (32 bytes)    │
├─────────────────────────────────────────────────────────────────────┤
│  Encrypted Payload (72 bytes):                                      │
│    - Responder Static Public Key (32 bytes)                        │
│    - Timestamp Echo (8 bytes)                                       │
│    - Selected Cipher Suite (4 bytes)                               │
│    - Random Padding (12 bytes)                                      │
│    - Auth Tag (16 bytes)                                           │
├─────────────────────────────────────────────────────────────────────┤
│  MAC (16 bytes)                                                     │
└─────────────────────────────────────────────────────────────────────┘

Encryption key: HKDF(DH(ie, re) || DH(ie, rs), "resp-encrypt")
```

#### 3.3.3 Phase 3: Initiator Auth

```
Initiator Auth (80 bytes):
├─────────────────────────────────────────────────────────────────────┤
│  Connection ID (8 bytes)                                            │
├─────────────────────────────────────────────────────────────────────┤
│  Encrypted Payload (56 bytes):                                      │
│    - Initiator Static Public Key (32 bytes)                        │
│    - Session Parameters (8 bytes)                                   │
│    - Auth Tag (16 bytes)                                           │
├─────────────────────────────────────────────────────────────────────┤
│  MAC (16 bytes)                                                     │
└─────────────────────────────────────────────────────────────────────┘

Encryption key: HKDF(ee || es || se, "init-auth")
```

### 3.4 Session Key Derivation

After handshake completion, both parties derive identical session keys:

```
Input Keying Material (IKM):
    DH(ie, re) || DH(ie, rs) || DH(is, re) || DH(is, rs)
    [32 bytes]    [32 bytes]    [32 bytes]    [32 bytes]
    = 128 bytes total

Key Derivation:
    PRK = HKDF-Extract(salt="protocol-v1", IKM)
    
    initiator_send_key = HKDF-Expand(PRK, "i2r-data", 32)
    responder_send_key = HKDF-Expand(PRK, "r2i-data", 32)
    initiator_send_nonce_salt = HKDF-Expand(PRK, "i2r-nonce", 4)
    responder_send_nonce_salt = HKDF-Expand(PRK, "r2i-nonce", 4)
    connection_id = HKDF-Expand(PRK, "connection-id", 8)
```

### 3.5 Forward Secrecy Ratcheting

The protocol implements continuous forward secrecy through symmetric ratcheting combined with periodic DH ratcheting:

#### 3.5.1 Symmetric Ratchet (Every Packet)

```
After each packet:
    chain_key[n+1] = BLAKE3(chain_key[n] || 0x01)
    message_key[n] = BLAKE3(chain_key[n] || 0x02)
    
    // Zeroize immediately after use
    zeroize(chain_key[n])
    zeroize(message_key[n])
```

#### 3.5.2 DH Ratchet (Time/Volume Triggered)

Triggered when either condition is met:
- 2 minutes elapsed since last ratchet
- 1,000,000 packets sent since last ratchet

```
REKEY Frame Payload:
├─────────────────────────────────────────────────────────────────────┤
│  New Ephemeral Public Key [Elligator2 encoded] (32 bytes)          │
├─────────────────────────────────────────────────────────────────────┤
│  Ratchet Sequence Number (4 bytes)                                  │
├─────────────────────────────────────────────────────────────────────┤
│  Auth Tag (16 bytes)                                                │
└─────────────────────────────────────────────────────────────────────┘

New Key Derivation:
    new_dh = DH(local_new_ephemeral, remote_ephemeral)
    new_chain_key = HKDF(current_chain_key || new_dh, "ratchet")
    
    // Old ephemeral private key immediately zeroized
```

### 3.6 Cryptographic Implementation Requirements

**Constant-Time Operations:** All cryptographic operations MUST be constant-time to prevent timing side-channels. This includes:
- X25519 scalar multiplication
- Elligator2 encoding/decoding
- AEAD encryption/decryption
- Key comparison

**Memory Zeroization:** All sensitive key material MUST be zeroized immediately after use:
```rust
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Zeroize, ZeroizeOnDrop)]
struct SessionKeys {
    send_key: [u8; 32],
    recv_key: [u8; 32],
    chain_key: [u8; 32],
}
```

**Random Number Generation:** All randomness MUST come from the operating system CSPRNG (`/dev/urandom` or `getrandom(2)`).

---

## 4. Frame Types and Formats

### 4.1 DATA Frame (0x01)

Carries file data chunks:

```
DATA Frame Payload:
├─────────────────────────────────────────────────────────────────────┤
│  Chunk Hash [BLAKE3, truncated] (16 bytes)                         │
├─────────────────────────────────────────────────────────────────────┤
│  File Data (variable, up to payload_length - 16)                   │
└─────────────────────────────────────────────────────────────────────┘

File Offset: Byte position in the file
Stream ID: Identifies the file transfer
Flags.FIN: Set on the final chunk of the file
Flags.CMP: Set if payload is LZ4-compressed
```

**Chunk Hash:** First 16 bytes of BLAKE3(chunk_data). Receiver verifies before writing to disk.

### 4.2 ACK Frame (0x02)

Selective acknowledgment using SACK-style ranges:

```
ACK Frame Payload:
├─────────────────────────────────────────────────────────────────────┤
│  Largest Acknowledged (4 bytes): Highest seq_num received          │
├─────────────────────────────────────────────────────────────────────┤
│  ACK Delay (2 bytes): Microseconds since largest_ack received      │
├─────────────────────────────────────────────────────────────────────┤
│  ACK Range Count (1 byte): Number of gap/ack ranges following      │
├─────────────────────────────────────────────────────────────────────┤
│  First ACK Range (4 bytes): Contiguous packets before largest_ack  │
├─────────────────────────────────────────────────────────────────────┤
│  [Repeated ACK Ranges]:                                             │
│    Gap (2 bytes): Packets missing between ranges                   │
│    ACK Range Length (2 bytes): Contiguous received packets         │
└─────────────────────────────────────────────────────────────────────┘

Example: Received [1-10, 15-20, 25-30], missing [11-14, 21-24]
    Largest Acknowledged: 30
    First ACK Range: 5 (packets 26-30)
    Gap 1: 4 (packets 21-24 missing)
    Range 1: 5 (packets 16-20)
    Gap 2: 4 (packets 11-14 missing)
    Range 2: 10 (packets 1-10)
```

### 4.3 CONTROL Frame (0x03)

General stream control operations:

```
CONTROL Frame Payload:
├─────────────────────────────────────────────────────────────────────┤
│  Control Type (2 bytes)                                             │
├─────────────────────────────────────────────────────────────────────┤
│  Control Data (variable)                                            │
└─────────────────────────────────────────────────────────────────────┘

Control Types:
    0x0001: STREAM_BLOCKED - Flow control blocked
    0x0002: DATA_BLOCKED - Connection-level flow control blocked
    0x0003: MAX_STREAMS - Update maximum concurrent streams
    0x0004: STREAMS_BLOCKED - Cannot open new streams
    0x0005: PRIORITY_UPDATE - Change stream priority
```

### 4.4 REKEY Frame (0x04)

Triggers forward secrecy ratchet (see Section 3.5.2).

### 4.5 PING/PONG Frames (0x05, 0x06)

```
PING/PONG Payload:
├─────────────────────────────────────────────────────────────────────┤
│  Ping ID (8 bytes): Echoed in PONG                                 │
├─────────────────────────────────────────────────────────────────────┤
│  Timestamp (8 bytes): Sender's clock                               │
└─────────────────────────────────────────────────────────────────────┘

RTT Calculation: PONG.received_time - PING.timestamp
```

### 4.6 CLOSE Frame (0x07)

```
CLOSE Frame Payload:
├─────────────────────────────────────────────────────────────────────┤
│  Error Code (4 bytes)                                               │
├─────────────────────────────────────────────────────────────────────┤
│  Reason Length (2 bytes)                                            │
├─────────────────────────────────────────────────────────────────────┤
│  Reason (UTF-8 string, variable)                                    │
└─────────────────────────────────────────────────────────────────────┘

Error Codes:
    0x00000000: NO_ERROR - Graceful close
    0x00000001: PROTOCOL_ERROR - Unspecified protocol violation
    0x00000002: INTERNAL_ERROR - Implementation error
    0x00000003: FLOW_CONTROL_ERROR - Flow control violation
    0x00000004: STREAM_LIMIT_ERROR - Too many streams
    0x00000005: FRAME_ENCODING_ERROR - Malformed frame
    0x00000006: CRYPTO_ERROR - Cryptographic failure
    0x00000007: TIMEOUT - Idle timeout exceeded
```

### 4.7 PAD Frame (0x08)

Cover traffic frame. The entire payload is random bytes with no semantic content. Receivers MUST process PAD frames identically to other frames (decrypt, verify auth tag) but discard the plaintext.

```
PAD Frame:
    Frame Type: 0x08
    Payload: Random bytes (size determined by traffic shaping)
```

### 4.8 STREAM_OPEN Frame (0x09)

```
STREAM_OPEN Payload:
├─────────────────────────────────────────────────────────────────────┤
│  Stream Type (1 byte):                                              │
│    0x00: Unidirectional                                            │
│    0x01: Bidirectional                                             │
├─────────────────────────────────────────────────────────────────────┤
│  Priority (1 byte): 0-255, higher = more important                 │
├─────────────────────────────────────────────────────────────────────┤
│  Initial Window (4 bytes): Flow control credit in bytes            │
├─────────────────────────────────────────────────────────────────────┤
│  Metadata Length (2 bytes)                                          │
├─────────────────────────────────────────────────────────────────────┤
│  Metadata (variable):                                               │
│    For file transfers: filename, size, hash                        │
└─────────────────────────────────────────────────────────────────────┘
```

### 4.9 STREAM_CLOSE Frame (0x0A)

Graceful stream termination:

```
STREAM_CLOSE Payload:
├─────────────────────────────────────────────────────────────────────┤
│  Error Code (4 bytes):                                              │
│    0x00000000: NO_ERROR - Normal close                             │
│    0x00000001: CANCEL - User-initiated cancellation                │
│    0x00000002: STOPPED - Receiver requested stop (STOP_SENDING)    │
├─────────────────────────────────────────────────────────────────────┤
│  Final Offset (8 bytes): Last byte successfully received           │
└─────────────────────────────────────────────────────────────────────┘

Stream ID: Identifies stream to close
Flags.FIN: Set to indicate clean stream closure
```

**Behavior**: Sender stops transmitting on this stream. Receiver may continue sending until it also sends STREAM_CLOSE (for bidirectional streams).

### 4.10 STREAM_RESET Frame (0x0B)

Abrupt stream termination (error condition):

```
STREAM_RESET Payload:
├─────────────────────────────────────────────────────────────────────┤
│  Error Code (4 bytes):                                              │
│    0x00000000: NO_ERROR - Clean abort                              │
│    0x00000001: TIMEOUT - Stream idle timeout                       │
│    0x00000002: RESOURCE_LIMIT - Out of buffer space                │
│    0x00000003: INTEGRITY_ERROR - Chunk hash mismatch               │
│    0x00000004: PROTOCOL_ERROR - Invalid frame sequence             │
├─────────────────────────────────────────────────────────────────────┤
│  Final Size (8 bytes): Number of bytes sent before reset           │
└─────────────────────────────────────────────────────────────────────┘

Stream ID: Stream to reset
Flags: No flags used
```

**Behavior**: Immediately terminates the stream. All buffered data for this stream is discarded by both sender and receiver.

### 4.11 WINDOW_UPDATE Frame (0x0C)

Flow control window credit:

```
WINDOW_UPDATE Payload:
├─────────────────────────────────────────────────────────────────────┤
│  Window Increment (8 bytes): Additional bytes peer may send        │
└─────────────────────────────────────────────────────────────────────┘

Stream ID:
  - 0: Connection-level flow control (affects all streams)
  - >0: Stream-level flow control (affects single stream)

Flags: No flags used
```

**Maximum Window**: 2^62 - 1 bytes (QUIC-compatible limit).

**Behavior**: Receiver sends when it has consumed data and freed buffer space. Sender tracks available window and blocks when window exhausted.

### 4.12 GO_AWAY Frame (0x0D)

Graceful connection shutdown:

```
GO_AWAY Payload:
├─────────────────────────────────────────────────────────────────────┤
│  Last Stream ID (4 bytes): Highest stream ID processed             │
├─────────────────────────────────────────────────────────────────────┤
│  Error Code (4 bytes):                                              │
│    0x00000000: NO_ERROR - Graceful shutdown                        │
│    0x00000001: SERVER_SHUTDOWN - Planned maintenance                │
│    0x00000002: OVERLOAD - Resource exhaustion                      │
│    0x00000003: VERSION_MISMATCH - Protocol incompatibility         │
├─────────────────────────────────────────────────────────────────────┤
│  Reason Length (2 bytes)                                            │
├─────────────────────────────────────────────────────────────────────┤
│  Reason (UTF-8 string, variable)                                    │
└─────────────────────────────────────────────────────────────────────┘

Stream ID: 0 (connection-level)
Flags: No flags used
```

**Behavior**: Sender stops accepting new streams but allows existing streams (ID ≤ Last Stream ID) to complete. Receiver should finish processing existing streams then close connection.

### 4.13 PATH_CHALLENGE/PATH_RESPONSE (0x0E, 0x0F)

Connection migration validation:

```
PATH_CHALLENGE Payload:
├─────────────────────────────────────────────────────────────────────┤
│  Challenge Data (8 bytes): Random                                  │
└─────────────────────────────────────────────────────────────────────┘

PATH_RESPONSE Payload:
├─────────────────────────────────────────────────────────────────────┤
│  Challenge Data (8 bytes): Echoed from PATH_CHALLENGE              │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 5. State Machine Definitions

### 5.1 Session State Machine

```
                              ┌─────────────────┐
                              │     CLOSED      │
                              └────────┬────────┘
                                       │ connect() or accept()
                                       ▼
                        ┌──────────────────────────────┐
                        │        HANDSHAKING           │
                        │  ┌──────────────────────┐    │
                        │  │ INIT_SENT (initiator)│    │
                        │  └──────────┬───────────┘    │
                        │             │ recv Phase 2   │
                        │             ▼                │
                        │  ┌──────────────────────┐    │
                        │  │ INIT_RECV (responder)│    │
                        │  └──────────┬───────────┘    │
                        │             │ recv Phase 3   │
                        └─────────────┼────────────────┘
                                      │ handshake complete
                                      ▼
                              ┌─────────────────┐
                              │   ESTABLISHED   │◄────────────┐
                              └────────┬────────┘             │
                                       │                      │
             ┌─────────────────────────┼─────────────────────┐│
             │                         │                     ││
             ▼                         ▼                     ▼│
    ┌─────────────────┐      ┌─────────────────┐    ┌───────────────┐
    │    REKEYING     │      │    DRAINING     │    │   MIGRATING   │
    │  (DH ratchet)   │      │ (graceful close)│    │(path change)  │
    └────────┬────────┘      └────────┬────────┘    └───────┬───────┘
             │                        │                     │
             │ rekey complete         │ drain timeout       │ path validated
             └────────────────────────┼─────────────────────┘
                                      │
                                      ▼
                              ┌─────────────────┐
                              │     CLOSED      │
                              └─────────────────┘
```

### 5.2 Stream State Machine

```
                         ┌────────────────┐
                         │      IDLE      │
                         └───────┬────────┘
                                 │ STREAM_OPEN sent/recv
                                 ▼
                         ┌────────────────┐
                         │      OPEN      │
                         └───────┬────────┘
                                 │
          ┌──────────────────────┼──────────────────────┐
          │                      │                      │
          ▼                      ▼                      ▼
┌──────────────────┐   ┌──────────────────┐   ┌──────────────────┐
│   HALF_CLOSED    │   │   HALF_CLOSED    │   │   DATA_SENT      │
│   (local FIN)    │   │   (remote FIN)   │   │   (no FIN yet)   │
└────────┬─────────┘   └────────┬─────────┘   └────────┬─────────┘
         │                      │                      │
         │ recv FIN             │ send FIN             │ send/recv FIN
         └──────────────────────┼──────────────────────┘
                                │
                                ▼
                         ┌────────────────┐
                         │     CLOSED     │
                         └────────────────┘
```

### 5.3 Congestion Control State Machine

See Section 9 for detailed BBR state machine.

---

## 6. Traffic Obfuscation Mechanisms

### 6.1 Elligator2 Key Encoding

All ephemeral public keys transmitted during handshake MUST be encoded using Elligator2 to appear as uniform random bytes.

#### 6.1.1 Encoding Process

```rust
/// Generate a key pair with Elligator2-encodable public key
fn generate_elligator_keypair() -> (SecretKey, Representative) {
    loop {
        // Generate random scalar
        let secret = SecretKey::random();
        let public = PublicKey::from(&secret);
        
        // Convert Montgomery point to Edwards form
        let edwards = public.to_edwards_point();
        
        // Add random low-order component (8 options)
        // This creates "dirty" points spanning all cosets
        let low_order_idx = random_u8() & 0x07;
        let dirty_point = edwards + LOW_ORDER_POINTS[low_order_idx];
        
        // Attempt Elligator2 inverse map
        if let Some(mut repr) = elligator2_inverse(dirty_point) {
            // Randomize the high bit (not used by decoding)
            if random_bool() {
                repr[31] |= 0x80;
            }
            return (secret, repr);
        }
        // ~50% of points are encodable; loop until success
    }
}

/// Decode a representative back to a public key
fn decode_representative(repr: &Representative) -> PublicKey {
    let mut clean_repr = *repr;
    clean_repr[31] &= 0x7F;  // Clear high bit
    
    let edwards_point = elligator2_forward(&clean_repr);
    let montgomery = edwards_point.to_montgomery();
    
    PublicKey::from_montgomery(montgomery)
}
```

#### 6.1.2 Security Properties

- **Indistinguishability:** Representatives are computationally indistinguishable from uniform random 32-byte strings
- **Completeness:** ~50% of random scalars produce encodable points (acceptable retry rate)
- **No Information Leak:** The low-order component and high bit add no exploitable structure

### 6.2 Packet Padding Strategy

The protocol implements multi-level padding to defeat traffic analysis:

#### 6.2.1 Padding Classes

```
Padding Class Definitions:
├─────────────────────────────────────────────────────────────────────┤
│  Class 0 (Tiny):    64 bytes   - Control frames                    │
│  Class 1 (Small):   256 bytes  - ACKs, small metadata              │
│  Class 2 (Medium):  512 bytes  - Small file chunks                 │
│  Class 3 (Large):   1024 bytes - Typical data                      │
│  Class 4 (MTU):     1472 bytes - Maximum efficiency                │
│  Class 5 (Jumbo):   8960 bytes - High-throughput mode              │
└─────────────────────────────────────────────────────────────────────┘
```

#### 6.2.2 Padding Selection Algorithm

```rust
fn select_padding_class(payload_len: usize, mode: PaddingMode) -> usize {
    match mode {
        PaddingMode::Performance => {
            // Minimal padding for speed
            PADDING_CLASSES.iter()
                .find(|&&size| size >= payload_len + HEADER_SIZE + TAG_SIZE)
                .copied()
                .unwrap_or(JUMBO_SIZE)
        }
        PaddingMode::Privacy => {
            // Random selection among valid classes
            let valid: Vec<_> = PADDING_CLASSES.iter()
                .filter(|&&size| size >= payload_len + HEADER_SIZE + TAG_SIZE)
                .collect();
            *valid[random_usize() % valid.len()]
        }
        PaddingMode::Stealth => {
            // Match typical HTTPS packet distribution
            sample_https_packet_size_distribution()
        }
    }
}
```

#### 6.2.3 Padding Content

Padding bytes MUST be cryptographically random. They are generated using a stream cipher keyed with session material:

```
padding_key = HKDF(session_key, "padding", 32)
padding_stream = ChaCha20(padding_key, packet_nonce)
padding_bytes = padding_stream.generate(padding_length)
```

### 6.3 Timing Obfuscation

#### 6.3.1 Inter-Packet Delay

```rust
fn calculate_send_delay(mode: TimingMode) -> Duration {
    match mode {
        TimingMode::LowLatency => Duration::ZERO,
        TimingMode::Moderate => {
            // Exponential distribution with mean 5ms
            let lambda = 200.0; // 1/5ms
            let delay_ms = -1.0 / lambda * random_f64().ln();
            Duration::from_secs_f64(delay_ms / 1000.0)
        }
        TimingMode::HighPrivacy => {
            // Match HTTPS timing patterns
            sample_https_timing_distribution()
        }
    }
}
```

#### 6.3.2 Burst Shaping

Real file transfers create obvious burst patterns. The protocol mitigates this:

```
Burst Shaping Algorithm:
1. Measure outgoing data rate over 100ms windows
2. If rate exceeds target_rate * 1.5, queue excess packets
3. Inject PAD frames during low-activity periods
4. Maintain minimum 10 packets/second baseline
```

### 6.4 Cover Traffic Generation

```rust
struct CoverTrafficGenerator {
    min_rate: u32,          // Minimum packets per second (default: 10)
    max_idle: Duration,     // Maximum time between packets (default: 100ms)
    last_send: Instant,
}

impl CoverTrafficGenerator {
    fn should_send_cover(&self, pending_data: bool) -> bool {
        if pending_data {
            return false;  // Real data takes priority
        }
        
        let elapsed = self.last_send.elapsed();
        if elapsed > self.max_idle {
            return true;  // Prevent timing gaps
        }
        
        // Probabilistic cover based on target rate
        let p = elapsed.as_secs_f64() * self.min_rate as f64;
        random_f64() < p
    }
    
    fn generate_cover_frame(&self) -> Frame {
        Frame {
            frame_type: FrameType::PAD,
            stream_id: 0,
            payload: random_bytes(random_range(64, 256)),
            ..Default::default()
        }
    }
}
```

### 6.5 Protocol Mimicry Modes

The protocol supports pluggable transport modules that make traffic appear as other protocols:

#### 6.5.1 HTTPS Mimicry

Wraps protocol frames in TLS 1.3 record structure:

```
TLS Record Wrapper:
├─────────────────────────────────────────────────────────────────────┤
│  Content Type (1 byte): 0x17 (Application Data)                    │
├─────────────────────────────────────────────────────────────────────┤
│  Legacy Version (2 bytes): 0x0303 (TLS 1.2)                        │
├─────────────────────────────────────────────────────────────────────┤
│  Length (2 bytes)                                                   │
├─────────────────────────────────────────────────────────────────────┤
│  Encrypted Protocol Frame                                           │
└─────────────────────────────────────────────────────────────────────┘
```

#### 6.5.2 WebSocket Mimicry

Frames protocol data as WebSocket binary messages:

```
WebSocket Frame Wrapper:
├─────────────────────────────────────────────────────────────────────┤
│  FIN=1, RSV=000, Opcode=0x2 (binary) (1 byte)                      │
├─────────────────────────────────────────────────────────────────────┤
│  MASK=1, Payload Length (1-9 bytes)                                │
├─────────────────────────────────────────────────────────────────────┤
│  Masking Key (4 bytes)                                              │
├─────────────────────────────────────────────────────────────────────┤
│  Masked Protocol Frame                                              │
└─────────────────────────────────────────────────────────────────────┘
```

#### 6.5.3 DNS-over-HTTPS Covert Channel

Encodes small payloads in DNS queries tunneled over HTTPS:

```
DoH Covert Channel:
├─────────────────────────────────────────────────────────────────────┤
│  HTTPS POST to resolver (e.g., 1.1.1.1/dns-query)                  │
├─────────────────────────────────────────────────────────────────────┤
│  Content-Type: application/dns-message                              │
├─────────────────────────────────────────────────────────────────────┤
│  DNS Query:                                                         │
│    QNAME: <base32(payload)>.tunnel.example.com                     │
│    QTYPE: TXT                                                       │
├─────────────────────────────────────────────────────────────────────┤
│  Response TXT records contain encoded reply                         │
└─────────────────────────────────────────────────────────────────────┘

Bandwidth: ~100-500 bytes per query, 10-50 queries/second
Use case: Control channel when direct UDP is blocked
```

---

## 7. Discovery Protocol

### 7.1 Privacy-Enhanced DHT

The discovery layer uses a modified Kademlia DHT where stored values are encrypted:

#### 7.1.1 Key Derivation for Announcements

```
File Announcement Key:
    dht_key = BLAKE3(group_secret || file_hash || "announce")[0..20]
    
Peer Discovery Key:
    dht_key = BLAKE3(group_secret || peer_id || "peer")[0..20]
    
Group Membership Key:
    dht_key = BLAKE3(group_secret || "members")[0..20]
```

#### 7.1.2 Announcement Format

```
Encrypted Announcement:
├─────────────────────────────────────────────────────────────────────┤
│  Nonce (24 bytes)                                                   │
├─────────────────────────────────────────────────────────────────────┤
│  Encrypted Payload:                                                 │
│    Peer Endpoints (variable): IP:port pairs                        │
│    Timestamp (8 bytes)                                              │
│    Capabilities (4 bytes)                                           │
│    Signature (64 bytes): Ed25519 over plaintext                    │
├─────────────────────────────────────────────────────────────────────┤
│  Auth Tag (16 bytes)                                                │
└─────────────────────────────────────────────────────────────────────┘

Encryption Key:
    announcement_key = HKDF(group_secret, "dht-announce", 32)
```

#### 7.1.3 DHT Security Properties

- **Unlinkability:** DHT keys appear random; only group members can compute them
- **Confidentiality:** Announcement content encrypted; DHT nodes see only ciphertext
- **Integrity:** Signature prevents tampering by malicious DHT nodes
- **Freshness:** Timestamp prevents replay of stale announcements

### 7.2 Relay Network (DERP-Style)

For peers behind restrictive NAT, a relay network provides connectivity:

#### 7.2.1 Relay Architecture

```
                    ┌─────────────────┐
                    │   Relay Server  │
                    │  (Public IP)    │
                    └────────┬────────┘
                             │
              ┌──────────────┴──────────────┐
              │ TLS 1.3                      │ TLS 1.3
              ▼                              ▼
       ┌─────────────┐                ┌─────────────┐
       │   Peer A    │                │   Peer B    │
       │  (NAT'd)    │                │  (NAT'd)    │
       └─────────────┘                └─────────────┘
       
Relay routes by public key, NOT IP address
All payloads are end-to-end encrypted (Noise session)
Relay sees: encrypted blobs, cannot decrypt
```

#### 7.2.2 Relay Protocol

```
Relay Frame Format:
├─────────────────────────────────────────────────────────────────────┤
│  Version (1 byte): 0x01                                            │
├─────────────────────────────────────────────────────────────────────┤
│  Command (1 byte):                                                  │
│    0x01: SEND - Forward to destination                             │
│    0x02: RECV - Deliver from source                                │
│    0x03: SUBSCRIBE - Register public key                           │
│    0x04: KEEPALIVE - Maintain connection                           │
├─────────────────────────────────────────────────────────────────────┤
│  Destination Public Key (32 bytes)                                  │
├─────────────────────────────────────────────────────────────────────┤
│  Payload Length (2 bytes)                                           │
├─────────────────────────────────────────────────────────────────────┤
│  Encrypted Payload (variable)                                       │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 8. NAT Traversal Protocol

### 8.1 Endpoint Discovery

```
STUN-like Endpoint Discovery (via Relay):

1. Peer sends probe to relay with random transaction ID
2. Relay responds with observed source IP:port
3. Peer repeats to multiple relays to detect NAT type

Response:
├─────────────────────────────────────────────────────────────────────┤
│  Transaction ID (12 bytes)                                          │
├─────────────────────────────────────────────────────────────────────┤
│  Mapped Address Family (1 byte): 0x01=IPv4, 0x02=IPv6              │
├─────────────────────────────────────────────────────────────────────┤
│  Mapped Port (2 bytes)                                              │
├─────────────────────────────────────────────────────────────────────┤
│  Mapped Address (4 or 16 bytes)                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 8.2 NAT Type Classification

| NAT Type | Behavior | Traversal Strategy |
|----------|----------|-------------------|
| Full Cone | Any external host can send | Direct connection |
| Address-Restricted | Only contacted IPs can send | Simultaneous open |
| Port-Restricted | Only contacted IP:port can send | Simultaneous open |
| Symmetric | Different mapping per destination | Birthday attack or relay |

### 8.3 Hole Punching Procedure

```
Hole Punching via Relay Signaling:

1. Both peers register with relay, learn their mapped endpoints
2. Relay exchanges endpoint information between peers
3. Both peers simultaneously send UDP probes to each other's mapped endpoint
4. First received probe validates the path
5. PATH_CHALLENGE/PATH_RESPONSE confirms bidirectional connectivity
6. Session migrates from relay to direct path

Timing:
    Probe interval: 25ms
    Probe timeout: 5 seconds
    Retry with port prediction after 2 seconds
```

### 8.4 Birthday Attack for Symmetric NAT

When both peers are behind symmetric NAT:

```
Birthday Attack Parameters:
    Peer A opens: N ports (e.g., 256)
    Peer B sends: M probes to random ports in predicted range
    
    Probability of success ≈ 1 - e^(-NM/65536)
    
    With N=256, M=256: ~63% success rate
    With N=512, M=512: ~98% success rate
    
Implementation:
    1. Both peers open multiple source ports
    2. Exchange port ranges via relay
    3. Send probes from all ports to all predicted destinations
    4. First successful probe establishes path
```

---

## 9. Congestion Control

### 9.1 BBR Algorithm Overview

The protocol implements BBRv2-inspired congestion control:

```
BBR State Machine:
                      ┌──────────────┐
                      │   STARTUP    │
                      │ (exponential │
                      │   probing)   │
                      └──────┬───────┘
                             │ bandwidth plateaus
                             ▼
                      ┌──────────────┐
                      │    DRAIN     │
                      │ (reduce      │
                      │  in-flight)  │
                      └──────┬───────┘
                             │ in-flight ≤ BDP
                             ▼
              ┌──────────────────────────────┐
              │           PROBE_BW           │◄───────┐
              │  (steady state, 8 phases)    │        │
              └──────────────┬───────────────┘        │
                             │ every 10 seconds       │
                             ▼                        │
                      ┌──────────────┐                │
                      │  PROBE_RTT   │────────────────┘
                      │ (measure     │  RTT measured
                      │  min RTT)    │
                      └──────────────┘
```

### 9.2 Key BBR Variables

```rust
struct BbrState {
    // Estimated bottleneck bandwidth (bytes/sec)
    btl_bw: u64,
    
    // Minimum observed RTT
    min_rtt: Duration,
    
    // Pacing rate = btl_bw * pacing_gain
    pacing_gain: f64,
    
    // Congestion window = BDP * cwnd_gain
    cwnd_gain: f64,
    
    // Bandwidth-Delay Product
    bdp: u64,
    
    // Current state
    state: BbrPhase,
    
    // Round-trip counter
    round_count: u64,
    
    // Time when current state entered
    state_start: Instant,
}

impl BbrState {
    fn update_model(&mut self, ack: &AckInfo) {
        // Update bandwidth estimate (max filter over 10 RTTs)
        let delivery_rate = ack.bytes_delivered / ack.ack_elapsed;
        self.btl_bw = max(self.btl_bw, delivery_rate);
        
        // Update RTT estimate (min filter over 10 seconds)
        self.min_rtt = min(self.min_rtt, ack.rtt);
        
        // Calculate BDP
        self.bdp = self.btl_bw * self.min_rtt.as_secs_f64() as u64;
    }
    
    fn pacing_rate(&self) -> u64 {
        (self.btl_bw as f64 * self.pacing_gain) as u64
    }
    
    fn cwnd(&self) -> u64 {
        (self.bdp as f64 * self.cwnd_gain) as u64
    }
}
```

### 9.3 Pacing Implementation

```rust
struct Pacer {
    rate: u64,              // bytes per second
    last_send: Instant,
    tokens: f64,            // accumulated send credit
}

impl Pacer {
    fn next_send_time(&mut self, packet_size: usize) -> Option<Instant> {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_send);
        
        // Accumulate tokens
        self.tokens += elapsed.as_secs_f64() * self.rate as f64;
        self.tokens = self.tokens.min(self.rate as f64 * 0.01); // Max 10ms burst
        
        if self.tokens >= packet_size as f64 {
            self.tokens -= packet_size as f64;
            self.last_send = now;
            Some(now)
        } else {
            // Calculate when we'll have enough tokens
            let needed = packet_size as f64 - self.tokens;
            let wait = Duration::from_secs_f64(needed / self.rate as f64);
            Some(now + wait)
        }
    }
}
```

---

## 10. Error Handling and Recovery

### 10.1 Packet Loss Detection

```rust
enum LossDetectionMethod {
    // Packet considered lost if not acked within time threshold
    TimeThreshold {
        threshold: Duration,  // Default: max(1.5 * smoothed_rtt, 1ms)
    },
    
    // Packet considered lost if later packet acked (reordering threshold)
    PacketThreshold {
        threshold: u32,  // Default: 3 packets
    },
}

fn detect_losses(
    unacked: &BTreeMap<u64, SentPacket>,
    largest_acked: u64,
    config: &LossConfig,
) -> Vec<u64> {
    let mut lost = Vec::new();
    
    for (&seq, packet) in unacked.iter() {
        // Time-based loss
        if packet.sent_time.elapsed() > config.time_threshold {
            lost.push(seq);
            continue;
        }
        
        // Packet-based loss (reordering threshold)
        if largest_acked.saturating_sub(seq) >= config.packet_threshold as u64 {
            lost.push(seq);
        }
    }
    
    lost
}
```

### 10.2 Retransmission Strategy

```
Retransmission Priority:
1. Lost packets (detected by loss detection)
2. Probe packets (PTO expiry)
3. New data

PTO (Probe Timeout) Calculation:
    PTO = smoothed_rtt + max(4 * rtt_variance, 1ms) + max_ack_delay
    
    On PTO expiry:
        - Send 1-2 probe packets (ACK-eliciting)
        - Double PTO for exponential backoff
        - After 5 consecutive PTOs, close connection
```

### 10.3 Connection Migration

```rust
struct ConnectionMigration {
    current_path: SocketAddr,
    pending_path: Option<PendingPath>,
    path_challenge_data: [u8; 8],
}

struct PendingPath {
    address: SocketAddr,
    challenge_sent: Instant,
    validated: bool,
}

impl ConnectionMigration {
    fn handle_packet_from_new_address(&mut self, addr: SocketAddr) {
        if addr != self.current_path {
            // Initiate path validation
            self.pending_path = Some(PendingPath {
                address: addr,
                challenge_sent: Instant::now(),
                validated: false,
            });
            
            // Send PATH_CHALLENGE
            self.path_challenge_data = random_bytes(8);
            self.send_path_challenge(addr, self.path_challenge_data);
        }
    }
    
    fn handle_path_response(&mut self, data: &[u8]) {
        if data == self.path_challenge_data {
            if let Some(ref mut pending) = self.pending_path {
                pending.validated = true;
                self.current_path = pending.address;
                // Migrate congestion control state
                self.reset_congestion_control();
            }
        }
    }
}
```

### 10.4 Stateless Reset

When a peer receives a packet for an unknown connection:

```
Stateless Reset Token:
    token = HMAC-BLAKE3(static_reset_key, connection_id)[0..16]

Stateless Reset Packet:
├─────────────────────────────────────────────────────────────────────┤
│  Random Header (variable, 21+ bytes to look like real packet)      │
├─────────────────────────────────────────────────────────────────────┤
│  Stateless Reset Token (16 bytes, at end)                          │
└─────────────────────────────────────────────────────────────────────┘

Detection:
    Peer checks last 16 bytes against expected reset token
    If match, close connection immediately without response
```

---

## 11. Security Properties and Threat Model

### 11.1 Achieved Security Properties

| Property | Mechanism | Guarantee |
|----------|-----------|-----------|
| **Confidentiality** | XChaCha20-Poly1305 AEAD | IND-CPA secure |
| **Integrity** | Poly1305 authentication | INT-CTXT secure |
| **Authenticity** | Noise_XX mutual auth | Peers verified |
| **Forward Secrecy** | Ephemeral DH + ratchet | Past sessions protected |
| **Post-Compromise Security** | DH ratchet every 2 min | Recovery after breach |
| **Replay Protection** | Nonce + sliding window | Duplicates rejected |
| **Traffic Analysis Resistance** | Padding + timing + cover | Best-effort obfuscation |
| **Key Indistinguishability** | Elligator2 encoding | Keys look random |

### 11.2 Threat Model

**In Scope:**
- Passive network observers (ISPs, nation-states)
- Active network attackers (MITM, injection)
- Malicious DHT nodes
- Compromised relay servers
- Traffic analysis attacks

**Out of Scope:**
- Endpoint compromise (malware on peer devices)
- Global passive adversary with traffic correlation
- Cryptographic breaks in primitives
- Side-channel attacks on implementation

### 11.3 Known Limitations

**Traffic Analysis:** While the protocol resists casual traffic analysis, a determined adversary with:
- Long-term observation
- Multiple vantage points
- Machine learning classifiers

...may still detect protocol usage through:
- Packet size distributions
- Timing patterns
- Connection graphs

**Deniability:** The protocol provides limited deniability:
- No signatures on message content (good)
- But static key authentication means peers can prove communication occurred

---

## 12. Protocol Constants and Parameters

### 12.1 Timing Constants

```rust
pub mod timing {
    use std::time::Duration;
    
    /// Minimum RTT before BBR enters PROBE_RTT
    pub const MIN_RTT_EXPIRY: Duration = Duration::from_secs(10);
    
    /// Initial RTT estimate before measurement
    pub const INITIAL_RTT: Duration = Duration::from_millis(100);
    
    /// Maximum ACK delay advertised
    pub const MAX_ACK_DELAY: Duration = Duration::from_millis(25);
    
    /// Idle timeout before connection close
    pub const IDLE_TIMEOUT: Duration = Duration::from_secs(30);
    
    /// Handshake timeout
    pub const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);
    
    /// DH ratchet interval
    pub const REKEY_INTERVAL: Duration = Duration::from_secs(120);
    
    /// Minimum time between cover traffic packets
    pub const MIN_COVER_INTERVAL: Duration = Duration::from_millis(100);
}
```

### 12.2 Size Constants

```rust
pub mod sizes {
    /// Frame header size (fixed portion)
    pub const FRAME_HEADER_SIZE: usize = 28;
    
    /// AEAD authentication tag size
    pub const AUTH_TAG_SIZE: usize = 16;
    
    /// Connection ID size
    pub const CONNECTION_ID_SIZE: usize = 8;
    
    /// Minimum packet size (to avoid fingerprinting small control packets)
    pub const MIN_PACKET_SIZE: usize = 64;
    
    /// Default MTU
    pub const DEFAULT_MTU: usize = 1500;
    
    /// Maximum payload in default MTU
    pub const MAX_PAYLOAD_DEFAULT: usize = 1428;
    
    /// Jumbo frame MTU
    pub const JUMBO_MTU: usize = 9000;
    
    /// Maximum payload in jumbo MTU
    pub const MAX_PAYLOAD_JUMBO: usize = 8928;
    
    /// Default file chunk size
    pub const DEFAULT_CHUNK_SIZE: usize = 262144; // 256 KiB
    
    /// Maximum streams per connection
    pub const MAX_STREAMS: u16 = 16384;
    
    /// Initial flow control window
    pub const INITIAL_WINDOW: u64 = 1048576; // 1 MiB
    
    /// Maximum flow control window
    pub const MAX_WINDOW: u64 = 16777216; // 16 MiB
}
```

### 12.3 Cryptographic Constants

```rust
pub mod crypto {
    /// X25519 public key size
    pub const X25519_PUBLIC_KEY_SIZE: usize = 32;
    
    /// X25519 secret key size
    pub const X25519_SECRET_KEY_SIZE: usize = 32;
    
    /// Elligator2 representative size
    pub const ELLIGATOR_REPR_SIZE: usize = 32;
    
    /// XChaCha20-Poly1305 key size
    pub const XCHACHA_KEY_SIZE: usize = 32;
    
    /// XChaCha20-Poly1305 nonce size
    pub const XCHACHA_NONCE_SIZE: usize = 24;
    
    /// Protocol nonce size (embedded in full nonce)
    pub const PROTOCOL_NONCE_SIZE: usize = 8;
    
    /// BLAKE3 output size
    pub const BLAKE3_OUTPUT_SIZE: usize = 32;
    
    /// Truncated chunk hash size
    pub const CHUNK_HASH_SIZE: usize = 16;
    
    /// Ed25519 signature size
    pub const ED25519_SIGNATURE_SIZE: usize = 64;
    
    /// Stateless reset token size
    pub const RESET_TOKEN_SIZE: usize = 16;
}
```

### 12.4 Protocol Limits

```rust
pub mod limits {
    /// Maximum packet reordering before loss detection
    pub const REORDER_THRESHOLD: u32 = 3;
    
    /// Maximum consecutive PTOs before connection close
    pub const MAX_PTO_COUNT: u32 = 5;
    
    /// Maximum packets per ratchet interval
    pub const MAX_PACKETS_PER_RATCHET: u64 = 1_000_000;
    
    /// Maximum counter value before mandatory rekey
    pub const MAX_COUNTER: u32 = u32::MAX - (1 << 20);
    
    /// Maximum concurrent file transfers
    pub const MAX_CONCURRENT_TRANSFERS: usize = 256;
    
    /// Maximum pending connection attempts
    pub const MAX_PENDING_CONNECTIONS: usize = 1024;
    
    /// DHT announcement TTL
    pub const DHT_ANNOUNCE_TTL: Duration = Duration::from_secs(3600);
    
    /// Maximum relay hops
    pub const MAX_RELAY_HOPS: u8 = 3;
}
```

---

## Appendix A: Wire Format Quick Reference

```
┌─────────────────────────────────────────────────────────────────────┐
│                     OUTER PACKET STRUCTURE                         │
├─────────────────────────────────────────────────────────────────────┤
│  [CID: 8B][Encrypted Payload: variable][Auth Tag: 16B]             │
│  Minimum: 24 bytes                                                  │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│                     INNER FRAME STRUCTURE                          │
├─────────────────────────────────────────────────────────────────────┤
│  [Nonce: 8B][Type: 1B][Flags: 1B][StreamID: 2B][Seq: 4B]           │
│  [Offset: 8B][PayloadLen: 2B][Reserved: 2B][Payload][Padding]      │
│  Header: 28 bytes fixed                                             │
└─────────────────────────────────────────────────────────────────────┘

Frame Types:
  0x01 DATA    0x02 ACK     0x03 CONTROL  0x04 REKEY
  0x05 PING    0x06 PONG    0x07 CLOSE    0x08 PAD
  0x09 STREAM_OPEN          0x0A STREAM_CLOSE
  0x0B STREAM_RESET         0x0C WINDOW_UPDATE
  0x0D GOAWAY               0x0E PATH_CHALLENGE
  0x0F PATH_RESPONSE
```

---

## Appendix B: State Transition Tables

### Session States

| Current State | Event | Next State | Action |
|---------------|-------|------------|--------|
| CLOSED | connect() | HANDSHAKING | Send Phase 1 |
| CLOSED | accept() | HANDSHAKING | Wait Phase 1 |
| HANDSHAKING | Handshake complete | ESTABLISHED | Enable transport |
| HANDSHAKING | Timeout | CLOSED | Report error |
| ESTABLISHED | REKEY sent | REKEYING | Pause new data |
| REKEYING | REKEY acked | ESTABLISHED | Resume data |
| ESTABLISHED | CLOSE sent | DRAINING | Stop sending |
| DRAINING | Drain timeout | CLOSED | Release resources |
| ESTABLISHED | Path change | MIGRATING | Validate path |
| MIGRATING | PATH_RESPONSE | ESTABLISHED | Update path |

---

## Document Revision History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0-DRAFT | 2025-11 | Initial specification |

---

*End of Protocol Technical Details*
