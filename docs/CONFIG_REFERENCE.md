# WRAITH Protocol Configuration Reference

**Version:** 0.7.0
**Last Updated:** 2025-12-01

---

## Overview

WRAITH uses TOML configuration files for all settings. The configuration system supports:
- Default configuration file at `~/.config/wraith/config.toml`
- System-wide configuration at `/etc/wraith/config.toml`
- Environment variable overrides
- Command-line argument overrides

**Configuration Precedence (highest to lowest):**
1. Command-line arguments
2. Environment variables
3. User configuration (`~/.config/wraith/config.toml`)
4. System configuration (`/etc/wraith/config.toml`)
5. Built-in defaults

---

## Complete Configuration Reference

### [node] - Node Identity

```toml
[node]
# 32-byte public key (auto-generated)
# Read-only, do not modify manually
public_key = "a1b2c3d4e5f67890..."

# Path to private key file (required)
# Must be chmod 600 for security
private_key_file = "~/.config/wraith/keypair.secret"

# Node nickname (optional, for display purposes)
# Max 64 characters, alphanumeric + underscore
nickname = "my_wraith_node"
```

**Environment Variables:**
- `WRAITH_PRIVATE_KEY_FILE` - Override private key path
- `WRAITH_NICKNAME` - Override node nickname

---

### [network] - Network Settings

```toml
[network]
# Address to listen on
# Format: "IP:PORT" or "0.0.0.0:PORT" for all interfaces
listen_addr = "0.0.0.0:41641"

# Public address (optional)
# Override auto-detected public IP
# Required for nodes behind NAT with manual port forwarding
public_addr = ""

# Enable UPnP port mapping
enable_upnp = true

# Enable NAT-PMP port mapping
enable_nat_pmp = true

# Idle connection timeout
# Connections without activity are closed after this duration
idle_timeout = "30s"

# Handshake timeout
# Maximum time to complete handshake
handshake_timeout = "10s"

# Maximum concurrent connections
max_connections = 1000

# Connection rate limit (per second)
connection_rate_limit = 100
```

**Environment Variables:**
- `WRAITH_LISTEN_ADDR` - Override listen address
- `WRAITH_PUBLIC_ADDR` - Override public address
- `WRAITH_MAX_CONNECTIONS` - Override max connections

---

### [transport] - Transport Layer

```toml
[transport]
# Transport mode: "auto", "udp", "af-xdp"
# - auto: Try AF_XDP, fall back to UDP
# - udp: Force UDP (works everywhere)
# - af-xdp: Force AF_XDP (Linux only, requires capabilities)
mode = "auto"

# Network interface for AF_XDP (required if mode = "af-xdp")
interface = "eth0"

# Queue ID for AF_XDP (typically 0)
queue_id = 0

# Maximum packet size (bytes)
# Default: 1472 (fits in 1500 MTU with headers)
max_packet_size = 1472

# Socket buffer sizes (bytes)
send_buffer_size = 2097152  # 2 MB
recv_buffer_size = 2097152  # 2 MB

# AF_XDP UMEM size (bytes)
# Shared memory for zero-copy I/O
xdp_umem_size = 16777216  # 16 MB

# Number of XDP ring entries
xdp_ring_size = 4096

# Enable busy polling (reduces latency, increases CPU)
busy_poll = false

# Busy poll timeout (microseconds)
busy_poll_timeout = 50
```

**Environment Variables:**
- `WRAITH_TRANSPORT_MODE` - Override transport mode
- `WRAITH_INTERFACE` - Override network interface

---

### [session] - Session Management

```toml
[session]
# Maximum concurrent sessions
max_sessions = 1000

# Session idle timeout
idle_timeout = "60s"

# Keep-alive interval
# Sends ping if no activity for this duration
keepalive_interval = "15s"

# Maximum retransmissions before session failure
max_retransmissions = 5

# Retransmission timeout (initial)
rto_initial = "200ms"

# Retransmission timeout (minimum)
rto_min = "100ms"

# Retransmission timeout (maximum)
rto_max = "10s"

# Enable fast retransmit (3 duplicate ACKs)
fast_retransmit = true

# Enable selective acknowledgment (SACK)
sack_enabled = true
```

---

### [congestion] - BBR Congestion Control

```toml
[congestion]
# Congestion control algorithm: "bbr", "cubic", "reno"
algorithm = "bbr"

# BBR: Pacing gain for STARTUP phase
startup_gain = 2.89

# BBR: Pacing gain for DRAIN phase
drain_gain = 0.75

# BBR: Pacing gain for PROBE_BW steady state
probe_bw_gain = 1.0

# BBR: Pacing gain for PROBE_RTT phase
probe_rtt_gain = 0.75

# BBR: Target RTT probe interval
probe_rtt_interval = "10s"

# BBR: RTT probe duration
probe_rtt_duration = "200ms"

# BBR: Maximum bandwidth window (RTTs)
bw_window_length = 10

# BBR: Minimum RTT filter window
min_rtt_window = "10s"

# Initial congestion window (packets)
initial_cwnd = 10

# Minimum congestion window (packets)
min_cwnd = 4

# Maximum congestion window (packets)
max_cwnd = 10000
```

---

### [obfuscation] - Traffic Obfuscation

```toml
[obfuscation]
# Default obfuscation level: "none", "low", "medium", "high", "paranoid"
default_level = "medium"

# Enable TLS mimicry (looks like HTTPS traffic)
tls_mimicry = false

# TLS server name for mimicry
tls_server_name = "cloudflare.com"

# Enable HTTP mimicry (looks like HTTP traffic)
http_mimicry = false

# HTTP host header for mimicry
http_host = "www.google.com"

# Padding mode: "random", "size_class", "constant"
padding_mode = "size_class"

# Size classes for size_class padding (bytes)
size_classes = [64, 128, 256, 512, 1024, 1472]

# Random padding range (bytes) for random mode
padding_min = 0
padding_max = 256

# Enable timing jitter
timing_jitter = true

# Timing jitter range (milliseconds)
jitter_min = 0
jitter_max = 50

# Enable cover traffic (dummy packets)
cover_traffic = false

# Cover traffic rate (packets per second)
cover_traffic_rate = 10

# Enable constant rate transmission (paranoid mode)
constant_rate = false

# Target constant rate (bytes per second)
constant_rate_bps = 1000000  # 1 MB/s
```

---

### [discovery] - Peer Discovery

```toml
[discovery]
# Enable DHT discovery
dht_enabled = true

# DHT bootstrap nodes
bootstrap_nodes = [
    "bootstrap1.wraith.network:41641",
    "bootstrap2.wraith.network:41641",
    "bootstrap3.wraith.network:41641",
]

# DHT replication factor (k)
replication_factor = 20

# DHT lookup concurrency (alpha)
query_concurrency = 3

# DHT bucket refresh interval
refresh_interval = "1h"

# DHT announce interval
announce_interval = "30m"

# Enable relay discovery
relay_enabled = true

# Relay servers
relay_servers = [
    "relay1.wraith.network:41641",
    "relay2.wraith.network:41641",
]

# Relay connection timeout
relay_timeout = "10s"

# Enable local discovery (mDNS/DNS-SD)
local_discovery = true

# Local discovery interface
local_interface = ""  # Empty = all interfaces

# Enable IPv6
ipv6_enabled = true
```

---

### [transfer] - File Transfer

```toml
[transfer]
# Default chunk size (bytes)
# Larger = more efficient, smaller = better resume granularity
chunk_size = 262144  # 256 KB

# Maximum parallel chunks
max_parallel_chunks = 16

# Verify each chunk (integrity check)
verify_chunks = true

# Output directory for received files
output_dir = "~/Downloads/wraith"

# Auto-accept transfers from known peers
auto_accept = false

# Auto-accept threshold (previous successful transfers)
auto_accept_threshold = 3

# Maximum file size (0 = unlimited)
max_file_size = 0

# Resume incomplete transfers
resume_enabled = true

# Resume metadata directory
resume_dir = "~/.local/share/wraith/resume"

# Resume metadata TTL (delete after this duration)
resume_ttl = "7d"

# Multi-peer download enabled
multi_peer = true

# Maximum peers per download
max_peers_per_download = 5

# Peer request timeout
peer_timeout = "30s"
```

---

### [files] - File I/O

```toml
[files]
# I/O backend: "auto", "standard", "io_uring"
# - auto: io_uring on Linux 6.2+, standard elsewhere
# - standard: Traditional file I/O
# - io_uring: Async I/O (Linux only)
backend = "auto"

# io_uring ring size (entries)
ring_size = 2048

# Enable io_uring polling mode (lower latency, higher CPU)
io_polling = false

# Enable direct I/O (bypass page cache)
direct_io = false

# Read buffer size (bytes)
read_buffer_size = 1048576  # 1 MB

# Write buffer size (bytes)
write_buffer_size = 1048576  # 1 MB

# Sync after each chunk
sync_on_write = false

# Pre-allocate files (faster writes)
preallocate = true
```

---

### [logging] - Logging Configuration

```toml
[logging]
# Log level: "trace", "debug", "info", "warn", "error"
level = "info"

# Log format: "text", "json"
format = "text"

# Log output: "stdout", "stderr", or file path
output = "stdout"

# Maximum log file size (with rotation)
max_size = "100MB"

# Maximum rotated log files to keep
max_files = 10

# Compress rotated logs
compress = true

# Include timestamps
timestamps = true

# Include thread IDs
thread_ids = false

# Include file/line numbers
file_line = false

# Enable audit logging
audit_enabled = false

# Audit log file
audit_file = "/var/log/wraith/audit.log"

# Audit events to log
audit_events = [
    "handshake_complete",
    "transfer_start",
    "transfer_complete",
    "peer_connected",
    "peer_disconnected",
    "authentication_failure",
]
```

**Environment Variables:**
- `WRAITH_LOG_LEVEL` or `RUST_LOG` - Override log level
- `WRAITH_LOG_FORMAT` - Override log format
- `WRAITH_LOG_OUTPUT` - Override log output

---

### [security] - Security Settings

```toml
[security]
# Keypair file path
keypair_path = "~/.config/wraith/keypair.secret"

# Group secrets directory
group_secrets_path = "~/.config/wraith/groups/"

# Trusted peers file
trusted_peers_file = "~/.config/wraith/trusted_peers.toml"

# Blocked peers file
blocked_peers_file = "~/.config/wraith/blocked_peers.toml"

# Enable replay protection
replay_protection = true

# Replay window size (nonces)
replay_window_size = 1024

# Ratchet key rotation interval
ratchet_interval = "2m"

# Ratchet key rotation packet threshold
ratchet_packet_threshold = 1000000

# Minimum protocol version to accept
min_protocol_version = 1

# Enable certificate pinning (for relays)
cert_pinning = false

# Pinned relay certificates
pinned_certs = []
```

---

### [metrics] - Prometheus Metrics

```toml
[metrics]
# Enable Prometheus metrics endpoint
enabled = false

# Metrics listen address
listen_addr = "127.0.0.1:9090"

# Metrics path
path = "/metrics"

# Include histogram metrics (more detail, more overhead)
histograms = true

# Histogram buckets for latency (milliseconds)
latency_buckets = [1, 5, 10, 25, 50, 100, 250, 500, 1000]

# Histogram buckets for throughput (KB/s)
throughput_buckets = [100, 500, 1000, 5000, 10000, 50000, 100000]
```

---

## Environment Variables Summary

| Variable | Description | Default |
|----------|-------------|---------|
| `WRAITH_CONFIG` | Config file path | `~/.config/wraith/config.toml` |
| `WRAITH_PRIVATE_KEY_FILE` | Private key path | `~/.config/wraith/keypair.secret` |
| `WRAITH_LISTEN_ADDR` | Listen address | `0.0.0.0:41641` |
| `WRAITH_PUBLIC_ADDR` | Public address | (auto-detected) |
| `WRAITH_TRANSPORT_MODE` | Transport mode | `auto` |
| `WRAITH_INTERFACE` | Network interface | `eth0` |
| `WRAITH_LOG_LEVEL` | Log level | `info` |
| `WRAITH_LOG_FORMAT` | Log format | `text` |
| `WRAITH_LOG_OUTPUT` | Log output | `stdout` |
| `WRAITH_OUTPUT_DIR` | Download directory | `~/Downloads/wraith` |
| `WRAITH_MAX_CONNECTIONS` | Max connections | `1000` |

---

## Configuration Examples

### Minimal Configuration

```toml
# ~/.config/wraith/config.toml
[node]
private_key_file = "~/.config/wraith/keypair.secret"

[network]
listen_addr = "0.0.0.0:41641"
```

### High-Performance Server

```toml
[transport]
mode = "af-xdp"
interface = "eth0"
xdp_umem_size = 67108864  # 64 MB
xdp_ring_size = 8192
busy_poll = true

[session]
max_sessions = 10000
max_retransmissions = 3
rto_initial = "100ms"

[congestion]
algorithm = "bbr"
max_cwnd = 50000

[transfer]
chunk_size = 1048576  # 1 MB
max_parallel_chunks = 32
verify_chunks = true

[files]
backend = "io_uring"
ring_size = 4096
io_polling = true
direct_io = true
```

### Privacy-Focused Configuration

```toml
[obfuscation]
default_level = "paranoid"
tls_mimicry = true
tls_server_name = "cloudflare.com"
padding_mode = "constant"
timing_jitter = true
jitter_min = 10
jitter_max = 100
cover_traffic = true
cover_traffic_rate = 5
constant_rate = true
constant_rate_bps = 500000

[discovery]
dht_enabled = true
relay_enabled = true
local_discovery = false

[logging]
level = "warn"
audit_enabled = false
```

### Relay Server Configuration

```toml
[node]
nickname = "relay-us-east-1"

[network]
listen_addr = "0.0.0.0:41641"
public_addr = "relay1.wraith.network:41641"
enable_upnp = false
enable_nat_pmp = false
max_connections = 50000

[transport]
mode = "af-xdp"
interface = "eth0"
xdp_umem_size = 134217728  # 128 MB

[session]
max_sessions = 50000
idle_timeout = "120s"

[discovery]
dht_enabled = true
relay_enabled = false  # This IS a relay

[metrics]
enabled = true
listen_addr = "127.0.0.1:9090"
histograms = true
```

---

## Validation

Validate configuration file:

```bash
# Check configuration syntax
wraith config validate --config ~/.config/wraith/config.toml

# Show effective configuration (with defaults)
wraith config show --config ~/.config/wraith/config.toml

# Generate default configuration
wraith config init --output ~/.config/wraith/config.toml
```

---

## See Also

- [User Guide](USER_GUIDE.md)
- [Deployment Guide](operations/deployment-guide.md)
- [API Reference](engineering/api-reference.md)
