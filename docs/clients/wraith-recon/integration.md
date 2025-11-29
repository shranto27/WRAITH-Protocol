# WRAITH-Recon Integration Guide

**Document Version:** 1.0.0
**Last Updated:** 2025-11-29

---

## 1. Integration Overview

WRAITH-Recon is designed to fit into a modern offensive security ecosystem. It can consume intelligence from other tools (e.g., scope definitions) and output structured data for analysis platforms (SIEM, Reporting tools).

---

## 2. Input Integrations

### 2.1 Scope Import
WRAITH-Recon can import scope definitions from standard formats.

*   **Nmap XML:**
    ```bash
    wraith-recon import --format nmap --input scan_results.xml --sign key.pem
    ```
    *Parses `<address addr="..." />` tags to generate an allowed CIDR list.*

*   **CSV (Asset Inventory):**
    *   Format: `IP,Hostname,Owner,Criticality`
    *   Usage: Tags assets in the internal DB with "Criticality" levels to adjust scan aggression.

### 2.2 Configuration Management
*   **Ansible/Terraform:**
    *   The `config.signed` file can be deployed via standard CM tools.
    *   The binary supports reading config from `STDIN` for pipeline integration.

---

## 3. Output Integrations

### 3.1 SIEM / Log Aggregation
WRAITH-Recon supports structured JSON logging for ingestion into ELK, Splunk, or Graylog.

**Log Format (JSON Line):**
```json
{
  "timestamp": "2025-11-29T10:00:00Z",
  "level": "INFO",
  "module": "active_scanner",
  "event": "port_open",
  "data": {
    "target": "10.0.0.5",
    "port": 443,
    "proto": "tcp",
    "fingerprint": "nginx/1.18"
  },
  "trace_id": "scan-001"
}
```

**Splunk Integration:**
*   Configure `fluend` or `filebeat` to tail `/var/log/wraith/findings.json`.
*   Dashboards can visualize "Open Ports by Subnet" or "Detected OS Distribution".

### 3.2 Vulnerability Scanners
WRAITH-Recon is *not* a vuln scanner, but it feeds them.

*   **Output:** `hosts.txt` (List of live IPs).
*   **Integration:**
    ```bash
    wraith-recon --mode active --out live_hosts.txt
    nessus-cli --input live_hosts.txt --launch "Basic Scan"
    ```

---

## 4. WRAITH Protocol Stack Integration

### 4.1 wraith-core Integration

**Session Management:**
```rust
use wraith_core::session::{Session, SessionConfig};

let config = SessionConfig {
    conn_id: [0u8; 8],  // Derived during handshake
    local_addr: "0.0.0.0:0".parse().unwrap(),
    peer_addr: listener_addr,
    max_streams: 16,
    initial_window: 1048576,  // 1MB
};

let mut session = Session::new(config)?;
```

**Frame Construction:**
```rust
use wraith_core::frame::{Frame, FrameType};

let recon_data = Frame::new(
    FrameType::DATA,
    stream_id,
    payload,
    &obfuscator,  // Handles padding
)?;
```

### 4.2 wraith-crypto Integration

**Noise Handshake:**
```rust
use wraith_crypto::noise::{NoiseBuilder, NoisePattern};

let noise = NoiseBuilder::new(NoisePattern::XX)
    .local_private_key(&static_key)
    .build()?;

// Perform 3-phase handshake
let (transport_state, peer_static_key) = noise.handshake(stream).await?;
```

**AEAD Encryption:**
```rust
use wraith_crypto::aead::{Aead, XChaCha20Poly1305};

let cipher = XChaCha20Poly1305::new(&session_key);
let nonce = generate_nonce(session_salt, packet_counter);
let ciphertext = cipher.encrypt(&nonce, plaintext)?;
```

**Elligator2 Encoding:**
```rust
use wraith_crypto::elligator2::Elligator2Point;

loop {
    let ephemeral = EphemeralSecret::new(&mut OsRng);
    let public_key = PublicKey::from(&ephemeral);

    if let Some(encoded) = Elligator2Point::encode(&public_key) {
        // Key successfully encoded as random-looking bytes
        break encoded;
    }
    // ~50% retry rate, acceptable
}
```

### 4.3 wraith-transport Integration

**Transport Modes:**
```rust
use wraith_transport::{Transport, TransportMode};

let transport = match config.mode {
    "afxdp" => Transport::new(TransportMode::AfXdp {
        interface: "eth0",
        queue_id: 0,
        umem_size: 67108864,  // 64MB
    })?,
    "iouring" => Transport::new(TransportMode::IoUring {
        ring_size: 4096,
        flags: IoUringFlags::SQPOLL,
    })?,
    "udp" => Transport::new(TransportMode::Udp {
        bind_addr: "0.0.0.0:0".parse()?,
    })?,
};
```

### 4.4 wraith-obfuscation Integration

**Padding Strategy:**
```rust
use wraith_obfuscation::padding::{PaddingMode, Padder};

let padder = Padder::new(PaddingMode::Stealth {
    distribution: vec![
        (64, 0.10),
        (256, 0.15),
        (512, 0.20),
        (1024, 0.25),
        (1472, 0.20),
        (8960, 0.10),
    ],
});

let padded_payload = padder.pad(original_data)?;
```

**Timing Obfuscation:**
```rust
use wraith_obfuscation::timing::{TimingProfile, JitterEngine};

let profile = TimingProfile::Exponential {
    mean_ms: 5.0,
    lambda: 200.0,
};

let delay = JitterEngine::calculate_delay(&profile);
tokio::time::sleep(delay).await;
```

**Protocol Mimicry:**
```rust
use wraith_obfuscation::mimicry::{MimicryProfile, TlsWrapper};

let wrapper = TlsWrapper::new(MimicryProfile::Tls13 {
    ja3_fingerprint: "771,4865-4866-4867,0-23-65281,29-23-24,0",
    sni: "cdn.example.com",
});

let tls_wrapped = wrapper.wrap_frame(&wraith_frame)?;
```

### 4.5 wraith-files Integration

**Chunking for Exfiltration:**
```rust
use wraith_files::chunker::{Chunker, ChunkSize};

let chunker = Chunker::new(ChunkSize::Fixed(1024));
let chunks = chunker.chunk_file(&file_data)?;

for (idx, chunk) in chunks.enumerate() {
    let hash = BLAKE3::hash(&chunk);
    send_chunk(idx, chunk, hash).await?;
}
```

### 4.6 Performance Optimization

**Thread-per-Core Model:**
- Each reconnaissance stream pinned to dedicated CPU core
- Lock-free data structures for cross-core communication
- NUMA-aware memory allocation for asset database

**Zero-Copy Operations:**
```rust
// AF_XDP UMEM direct access
let rx_desc = rx_queue.consume(1)?;
let packet_data = &umem[rx_desc.addr..][..rx_desc.len];
// Process without copying
process_packet_zerocopy(packet_data)?;
```

**BBR Congestion Control:**
- Bottleneck bandwidth estimation for maximum scan throughput
- RTT probing for adaptive pacing
- Integrated via wraith-core session layer

---

## 4. Interoperability with WRAITH Ecosystem

### 4.1 WRAITH-RedOps
*   **Role:** WRAITH-Recon acts as the "Eyes" for RedOps "Hands".
*   **Workflow:**
    1.  Recon maps the network and identifies a Gateway.
    2.  Recon exports the Gateway IP and open UDP port.
    3.  RedOps configures a listener profile matching that open port.

### 4.2 WRAITH-Relay
*   WRAITH-Recon can route its scan traffic *through* a WRAITH-Relay mesh to anonymize the source IP.
*   **Config:** `[transport] proxy = "wraith://10.10.10.10:9000"`

---

## 5. API Reference (IPC)

WRAITH-Recon exposes a local Unix Domain Socket for IPC control (if enabled).

**Endpoint:** `/var/run/wraith-recon.sock`
**Protocol:** JSON-RPC 2.0

**Methods:**
*   `status()`: Returns current scan progress.
*   `pause()`: Temporarily halts packet generation.
*   `resume()`: Resumes operations.
*   `dump_assets()`: Returns the current Asset Graph JSON.
