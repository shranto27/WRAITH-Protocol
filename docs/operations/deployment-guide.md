# WRAITH Protocol Deployment Guide

**Document Version:** 1.0.0
**Last Updated:** 2025-11-28
**Status:** Operations Documentation

---

## Overview

This guide provides comprehensive instructions for deploying WRAITH Protocol in production environments, covering infrastructure setup, configuration management, and operational best practices.

**Deployment Models:**
- Standalone peer nodes
- DHT bootstrap nodes
- Relay servers
- Client applications

---

## Production System Requirements

### Hardware Specifications

**Peer Node (Basic):**
```
CPU: 2+ cores
RAM: 4 GB
Network: 100 Mbps+
Storage: 20 GB
```

**Peer Node (High-Performance):**
```
CPU: 8+ cores @ 3.0+ GHz (AVX2/NEON)
RAM: 16 GB
Network: 10 Gbps (XDP-capable NIC)
Storage: NVMe SSD
```

**DHT Bootstrap Node:**
```
CPU: 4+ cores
RAM: 8 GB
Network: 1 Gbps (public IP required)
Storage: 50 GB
Uptime: 99.9%+ required
```

**Relay Server:**
```
CPU: 8+ cores
RAM: 16 GB
Network: 10+ Gbps (public IP, low latency)
Storage: 20 GB
TLS certificate (Let's Encrypt)
```

### Operating System

**Supported Platforms:**
- Ubuntu Server 22.04 LTS / 24.04 LTS
- Fedora Server 38+
- Debian 12+
- Rocky Linux 9+

**Kernel Requirements:**
- Linux 6.2+ (for AF_XDP/io_uring)
- Kernel features: `CONFIG_XDP_SOCKETS=y`, `CONFIG_IO_URING=y`, `CONFIG_BPF=y`

---

## Installation

### Binary Installation

**From GitHub Releases:**
```bash
# Download latest release
VERSION="0.1.0"
wget https://github.com/wraith/wraith-protocol/releases/download/v${VERSION}/wraith-cli-linux-x86_64

# Verify checksum
sha256sum -c wraith-cli-linux-x86_64.sha256

# Install
sudo install -m 755 wraith-cli-linux-x86_64 /usr/local/bin/wraith-cli

# Verify
wraith-cli --version
```

### Building from Source

```bash
# Install dependencies
sudo apt install build-essential pkg-config libssl-dev libbpf-dev

# Clone repository
git clone https://github.com/wraith/wraith-protocol.git
cd wraith-protocol

# Build release binary
RUSTFLAGS="-C target-cpu=native -C lto=fat" cargo build --release --features af-xdp,io-uring

# Install
sudo install -m 755 target/release/wraith-cli /usr/local/bin/
```

---

## Configuration

### Configuration File

**Default location:** `/etc/wraith/config.toml`

```toml
[network]
bind_address = "0.0.0.0:41641"
public_address = "203.0.113.50:41641"  # Optional: override detected IP
enable_upnp = true
enable_nat_pmp = true

[transport]
mode = "auto"  # auto, udp, af-xdp
interface = "eth0"  # For AF_XDP
max_packet_size = 1472
send_buffer_size = 2097152  # 2 MB
recv_buffer_size = 2097152

[session]
handshake_timeout = "10s"
idle_timeout = "30s"
max_concurrent_sessions = 1000

[dht]
enabled = true
bootstrap_nodes = [
    "dht1.wraith.network:41641",
    "dht2.wraith.network:41641",
    "dht3.wraith.network:41641",
]
replication_factor = 20
query_concurrency = 3

[relay]
enabled = true
servers = [
    "relay1.wraith.network:41641",
    "relay2.wraith.network:41641",
]

[files]
chunk_size = 1048576  # 1 MB
max_parallel_chunks = 16
verify_chunks = true
storage_path = "/var/lib/wraith/files"

[logging]
level = "info"
format = "json"
output = "/var/log/wraith/wraith.log"

[security]
keypair_path = "/etc/wraith/keypair.secret"
group_secrets_path = "/etc/wraith/groups/"
```

### Key Generation

```bash
# Generate node keypair
wraith-cli keygen --output /etc/wraith/keypair.secret

# Set permissions (critical!)
sudo chmod 600 /etc/wraith/keypair.secret
sudo chown wraith:wraith /etc/wraith/keypair.secret

# Verify
wraith-cli keypair info --keypair /etc/wraith/keypair.secret
```

---

## Systemd Service

### Service File

**Location:** `/etc/systemd/system/wraith.service`

```ini
[Unit]
Description=WRAITH Protocol Node
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=wraith
Group=wraith
ExecStart=/usr/local/bin/wraith-cli daemon --config /etc/wraith/config.toml
Restart=on-failure
RestartSec=10s
StandardOutput=journal
StandardError=journal

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/wraith /var/log/wraith
CapabilityBoundingSet=CAP_NET_RAW CAP_NET_ADMIN CAP_BPF
AmbientCapabilities=CAP_NET_RAW CAP_NET_ADMIN CAP_BPF

# Resource limits
LimitNOFILE=65536
LimitNPROC=512
MemoryMax=2G
TasksMax=512

[Install]
WantedBy=multi-user.target
```

### Service Management

```bash
# Create wraith user
sudo useradd -r -s /bin/false wraith

# Create directories
sudo mkdir -p /etc/wraith /var/lib/wraith /var/log/wraith
sudo chown -R wraith:wraith /var/lib/wraith /var/log/wraith

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable wraith
sudo systemctl start wraith

# Check status
sudo systemctl status wraith

# View logs
sudo journalctl -u wraith -f
```

---

## Network Configuration

### Firewall Rules

**iptables:**
```bash
# Allow WRAITH traffic
sudo iptables -A INPUT -p udp --dport 41641 -j ACCEPT

# Save rules
sudo iptables-save | sudo tee /etc/iptables/rules.v4
```

**firewalld:**
```bash
# Add WRAITH service
sudo firewall-cmd --permanent --add-port=41641/udp
sudo firewall-cmd --reload
```

**ufw:**
```bash
# Allow WRAITH port
sudo ufw allow 41641/udp
sudo ufw enable
```

### NAT Traversal

**UPnP configuration:**
```bash
# Install miniupnpc
sudo apt install miniupnpc

# Test port forwarding
upnpc -a $(hostname -I | awk '{print $1}') 41641 41641 UDP

# Verify
upnpc -l
```

---

## DHT Bootstrap Node Deployment

### Dedicated DHT Node Configuration

```toml
[dht]
mode = "bootstrap"
bind_address = "0.0.0.0:41641"
public_address = "203.0.113.50:41641"
storage_limit = "10GB"
announce_interval = "3600s"
```

### High Availability Setup

**Load balancer configuration (HAProxy):**
```
frontend wraith_dht
    bind *:41641
    mode udp
    default_backend dht_nodes

backend dht_nodes
    mode udp
    balance roundrobin
    server dht1 10.0.1.10:41641 check
    server dht2 10.0.1.11:41641 check
    server dht3 10.0.1.12:41641 check
```

---

## Relay Server Deployment

### Relay Configuration

```toml
[relay]
mode = "server"
bind_address = "0.0.0.0:41641"
public_address = "relay1.wraith.network:41641"
tls_cert = "/etc/letsencrypt/live/relay1.wraith.network/fullchain.pem"
tls_key = "/etc/letsencrypt/live/relay1.wraith.network/privkey.pem"
max_clients = 10000
bandwidth_limit = "5Gbps"
```

### TLS Certificate Setup

```bash
# Install certbot
sudo apt install certbot

# Obtain certificate
sudo certbot certonly --standalone -d relay1.wraith.network

# Auto-renewal
sudo systemctl enable certbot.timer
sudo systemctl start certbot.timer
```

---

## Container Deployment

### Docker

**Dockerfile:**
```dockerfile
FROM rust:1.75 AS builder
WORKDIR /build
COPY . .
RUN cargo build --release --features af-xdp,io-uring

FROM ubuntu:24.04
RUN apt-get update && apt-get install -y libssl3 ca-certificates
COPY --from=builder /build/target/release/wraith-cli /usr/local/bin/
EXPOSE 41641/udp
ENTRYPOINT ["wraith-cli"]
CMD ["daemon"]
```

**docker-compose.yml:**
```yaml
version: '3.8'

services:
  wraith:
    build: .
    container_name: wraith-node
    restart: unless-stopped
    ports:
      - "41641:41641/udp"
    volumes:
      - ./config.toml:/etc/wraith/config.toml:ro
      - ./keypair.secret:/etc/wraith/keypair.secret:ro
      - wraith-data:/var/lib/wraith
    cap_add:
      - NET_RAW
      - NET_ADMIN
      - BPF
    environment:
      - RUST_LOG=info

volumes:
  wraith-data:
```

### Kubernetes

**Deployment:**
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: wraith-node
spec:
  replicas: 3
  selector:
    matchLabels:
      app: wraith
  template:
    metadata:
      labels:
        app: wraith
    spec:
      containers:
      - name: wraith
        image: wraith:latest
        ports:
        - containerPort: 41641
          protocol: UDP
        volumeMounts:
        - name: config
          mountPath: /etc/wraith
        - name: data
          mountPath: /var/lib/wraith
        securityContext:
          capabilities:
            add:
              - NET_RAW
              - NET_ADMIN
              - BPF
        resources:
          requests:
            cpu: "1"
            memory: "1Gi"
          limits:
            cpu: "4"
            memory: "4Gi"
      volumes:
      - name: config
        configMap:
          name: wraith-config
      - name: data
        persistentVolumeClaim:
          claimName: wraith-data
```

---

## Backup and Recovery

### Configuration Backup

```bash
# Backup configuration
sudo tar czf wraith-backup-$(date +%Y%m%d).tar.gz \
    /etc/wraith/config.toml \
    /etc/wraith/keypair.secret \
    /etc/wraith/groups/

# Restore
sudo tar xzf wraith-backup-20251128.tar.gz -C /
sudo chown -R wraith:wraith /etc/wraith
sudo chmod 600 /etc/wraith/keypair.secret
```

### Data Backup

```bash
# Backup file storage
sudo rsync -avz /var/lib/wraith/ backup-server:/backups/wraith/

# Automated backup (cron)
echo "0 2 * * * root rsync -avz /var/lib/wraith/ backup-server:/backups/wraith/" | \
    sudo tee -a /etc/crontab
```

---

## Performance Tuning

### System Tuning

```bash
# Increase socket buffers (critical for high throughput)
sudo sysctl -w net.core.rmem_max=26214400    # 25 MB
sudo sysctl -w net.core.wmem_max=26214400    # 25 MB
sudo sysctl -w net.core.rmem_default=1048576 # 1 MB
sudo sysctl -w net.core.wmem_default=1048576 # 1 MB

# Increase file descriptors
echo "wraith soft nofile 65536" | sudo tee -a /etc/security/limits.conf
echo "wraith hard nofile 65536" | sudo tee -a /etc/security/limits.conf

# Network optimization
sudo sysctl -w net.core.netdev_max_backlog=65536
sudo sysctl -w net.core.optmem_max=25165824

# Enable BBR congestion control (for relay servers)
sudo sysctl -w net.ipv4.tcp_congestion_control=bbr
sudo sysctl -w net.core.default_qdisc=fq

# UDP buffer optimization
sudo sysctl -w net.ipv4.udp_rmem_min=8192
sudo sysctl -w net.ipv4.udp_wmem_min=8192

# Make persistent
cat << EOF | sudo tee -a /etc/sysctl.d/99-wraith.conf
net.core.rmem_max=26214400
net.core.wmem_max=26214400
net.core.rmem_default=1048576
net.core.wmem_default=1048576
net.core.netdev_max_backlog=65536
net.core.optmem_max=25165824
net.ipv4.tcp_congestion_control=bbr
net.core.default_qdisc=fq
EOF
sudo sysctl -p /etc/sysctl.d/99-wraith.conf
```

### CPU Optimization

```bash
# Pin WRAITH process to specific cores (NUMA-aware)
sudo numactl --cpunodebind=0 --membind=0 wraith-cli daemon

# Or use taskset for specific cores
sudo taskset -c 0-3 wraith-cli daemon

# Disable CPU frequency scaling (for consistent performance)
sudo cpupower frequency-set -g performance
```

### AF_XDP Performance (Linux)

```bash
# Enable XDP support on network interface
sudo ethtool -L eth0 combined 4  # Set number of queues

# Increase UMEM (shared memory) size
# Edit config.toml:
# [transport]
# xdp_umem_size = 67108864  # 64 MB

# Grant capabilities to binary (avoid running as root)
sudo setcap cap_net_raw,cap_net_admin,cap_bpf+ep /usr/local/bin/wraith-cli
```

### io_uring Optimization

```bash
# Increase max pending I/O operations
echo 32768 | sudo tee /proc/sys/fs/aio-max-nr

# For NVMe storage, enable polling mode in config.toml:
# [files]
# io_polling = true
# ring_size = 4096
```

### Performance Benchmarks

Expected performance on modern hardware:

| Configuration | Throughput | Latency |
|---------------|------------|---------|
| UDP (default) | 300-500 Mbps | 1-5 ms |
| AF_XDP (1 core) | 1-3 Gbps | <1 ms |
| AF_XDP (4 cores) | 5-10 Gbps | <500 us |
| io_uring file I/O | >3 GB/s | <100 us |

**Run benchmarks:**
```bash
# Protocol benchmarks
cargo bench -p wraith-core

# File I/O benchmarks
cargo bench -p wraith-files

# Profiling
./scripts/profile.sh --benchmark transfer
```

---

## Security Hardening

### Access Control

```bash
# Create dedicated user (non-login)
sudo useradd -r -s /sbin/nologin wraith

# Restrict configuration directory
sudo mkdir -p /etc/wraith
sudo chmod 750 /etc/wraith
sudo chown root:wraith /etc/wraith

# Protect keypair (critical - contains private key)
sudo chmod 600 /etc/wraith/keypair.secret
sudo chown wraith:wraith /etc/wraith/keypair.secret

# Log directory
sudo mkdir -p /var/log/wraith
sudo chmod 750 /var/log/wraith
sudo chown wraith:wraith /var/log/wraith

# Data directory
sudo mkdir -p /var/lib/wraith
sudo chmod 700 /var/lib/wraith
sudo chown wraith:wraith /var/lib/wraith
```

### Cryptographic Best Practices

1. **Key Rotation:** Rotate node keypairs periodically (recommended: yearly)
   ```bash
   wraith-cli keygen --output /etc/wraith/keypair.secret.new
   # Backup old key, then rename
   ```

2. **Key Backup:** Store encrypted backup of private key
   ```bash
   gpg -c /etc/wraith/keypair.secret > ~/secure-backup/keypair.gpg
   ```

3. **Memory Zeroization:** WRAITH automatically zeroizes sensitive data on drop.
   Verify with: `cargo test --features memory-testing`

### Systemd Security

Enhanced systemd unit with maximum sandboxing:

```ini
[Unit]
Description=WRAITH Protocol Node
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=wraith
Group=wraith
ExecStart=/usr/local/bin/wraith-cli daemon --config /etc/wraith/config.toml
Restart=on-failure
RestartSec=10s

# Security Hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectKernelLogs=true
ProtectControlGroups=true
ProtectProc=invisible
ProcSubset=pid
PrivateDevices=true
PrivateUsers=true
RestrictAddressFamilies=AF_INET AF_INET6 AF_UNIX AF_NETLINK
RestrictNamespaces=true
RestrictRealtime=true
RestrictSUIDSGID=true
MemoryDenyWriteExecute=true
LockPersonality=true
SystemCallFilter=@system-service
SystemCallArchitectures=native

# Required capabilities for AF_XDP (remove if using UDP only)
CapabilityBoundingSet=CAP_NET_RAW CAP_NET_ADMIN CAP_BPF
AmbientCapabilities=CAP_NET_RAW CAP_NET_ADMIN CAP_BPF

# File system access
ReadWritePaths=/var/lib/wraith /var/log/wraith
ReadOnlyPaths=/etc/wraith

# Resource limits
LimitNOFILE=65536
LimitNPROC=512
MemoryMax=2G
TasksMax=512

[Install]
WantedBy=multi-user.target
```

### SELinux Policy

**Full SELinux policy (wraith.te):**
```te
policy_module(wraith, 1.0.0)

require {
    type init_t;
    type var_lib_t;
    type var_log_t;
    type etc_t;
}

# Define types
type wraith_t;
type wraith_exec_t;
type wraith_conf_t;
type wraith_var_lib_t;
type wraith_log_t;

# Domain transitions
init_daemon_domain(wraith_t, wraith_exec_t)
files_type(wraith_conf_t)
files_type(wraith_var_lib_t)
logging_log_file(wraith_log_t)

# Capabilities
allow wraith_t self:capability { net_raw net_admin sys_resource };
allow wraith_t self:capability2 bpf;

# Network
allow wraith_t self:udp_socket create_socket_perms;
allow wraith_t self:packet_socket create_socket_perms;
allow wraith_t self:bpf { map_create map_read map_write prog_load prog_run };

# File access
allow wraith_t wraith_conf_t:file read_file_perms;
allow wraith_t wraith_conf_t:dir search_dir_perms;
allow wraith_t wraith_var_lib_t:file manage_file_perms;
allow wraith_t wraith_var_lib_t:dir manage_dir_perms;
allow wraith_t wraith_log_t:file append_file_perms;
```

**Install SELinux policy:**
```bash
checkmodule -M -m -o wraith.mod wraith.te
semodule_package -o wraith.pp -m wraith.mod
sudo semodule -i wraith.pp
```

### AppArmor Profile

**/etc/apparmor.d/usr.local.bin.wraith-cli:**
```apparmor
#include <tunables/global>

/usr/local/bin/wraith-cli {
  #include <abstractions/base>
  #include <abstractions/nameservice>

  # Binary
  /usr/local/bin/wraith-cli mr,

  # Configuration
  /etc/wraith/ r,
  /etc/wraith/** r,
  /etc/wraith/keypair.secret r,

  # Data
  /var/lib/wraith/ rw,
  /var/lib/wraith/** rwk,

  # Logs
  /var/log/wraith/ rw,
  /var/log/wraith/** rw,

  # Network
  network inet dgram,
  network inet6 dgram,
  network netlink raw,
  network packet raw,

  # Capabilities
  capability net_raw,
  capability net_admin,

  # BPF (for AF_XDP)
  capability bpf,

  # Deny everything else
  deny /home/** rwx,
  deny /root/** rwx,
}
```

**Install AppArmor profile:**
```bash
sudo cp wraith-cli.apparmor /etc/apparmor.d/usr.local.bin.wraith-cli
sudo apparmor_parser -r /etc/apparmor.d/usr.local.bin.wraith-cli
```

### Network Security

```bash
# Firewall rules (nftables)
sudo nft add table inet wraith
sudo nft add chain inet wraith input { type filter hook input priority 0 \; }
sudo nft add chain inet wraith output { type filter hook output priority 0 \; }

# Allow WRAITH traffic
sudo nft add rule inet wraith input udp dport 41641 accept
sudo nft add rule inet wraith output udp sport 41641 accept

# Rate limiting (prevent DoS)
sudo nft add rule inet wraith input udp dport 41641 limit rate 10000/second accept
sudo nft add rule inet wraith input udp dport 41641 drop
```

### Security Audit Checklist

Before production deployment:

- [ ] **Key Management:** Private key permissions 600, owned by wraith user
- [ ] **Systemd Hardening:** All security directives enabled
- [ ] **SELinux/AppArmor:** Policy installed and enforcing
- [ ] **Firewall:** UDP 41641 allowed, rate limiting configured
- [ ] **Logging:** Audit logs enabled, rotated, secured
- [ ] **Updates:** Latest WRAITH version, all dependencies updated
- [ ] **Monitoring:** Prometheus metrics exported, alerts configured
- [ ] **Backup:** Configuration and keys backed up securely
- [ ] **Network:** TLS for relay connections, DHT over trusted bootstrap nodes
- [ ] **Testing:** Security scan passed (`cargo audit`, fuzzing)

### Security Monitoring

```bash
# Monitor for security events
journalctl -u wraith -f --grep="SECURITY|WARN|ERROR"

# Check for unusual connections
ss -ulnp | grep wraith

# Monitor capability usage (auditd)
sudo auditctl -w /usr/local/bin/wraith-cli -p x -k wraith_exec
sudo ausearch -k wraith_exec
```

---

## Logging and Auditing

### Log Configuration

```toml
# In config.toml
[logging]
level = "info"              # trace, debug, info, warn, error
format = "json"             # json or text
output = "/var/log/wraith/wraith.log"
max_size = "100MB"
max_files = 10
compress = true
```

### Log Rotation

**/etc/logrotate.d/wraith:**
```
/var/log/wraith/*.log {
    daily
    missingok
    rotate 30
    compress
    delaycompress
    notifempty
    create 640 wraith wraith
    sharedscripts
    postrotate
        systemctl reload wraith > /dev/null 2>&1 || true
    endscript
}
```

### Audit Logging

Enable comprehensive audit trail:

```toml
# In config.toml
[logging]
audit_enabled = true
audit_file = "/var/log/wraith/audit.log"
audit_events = [
    "handshake_complete",
    "transfer_start",
    "transfer_complete",
    "peer_connected",
    "peer_disconnected",
    "authentication_failure",
]
```

---

## See Also

- [Monitoring](monitoring.md)
- [Troubleshooting](troubleshooting.md)
- [Platform Support](../integration/platform-support.md)
