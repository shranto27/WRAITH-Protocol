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
# Increase socket buffers
sudo sysctl -w net.core.rmem_max=26214400
sudo sysctl -w net.core.wmem_max=26214400

# Increase file descriptors
echo "wraith soft nofile 65536" | sudo tee -a /etc/security/limits.conf
echo "wraith hard nofile 65536" | sudo tee -a /etc/security/limits.conf

# Enable BBR congestion control
sudo sysctl -w net.ipv4.tcp_congestion_control=bbr

# Make persistent
echo "net.core.rmem_max=26214400" | sudo tee -a /etc/sysctl.conf
echo "net.core.wmem_max=26214400" | sudo tee -a /etc/sysctl.conf
sudo sysctl -p
```

---

## Security Hardening

### Access Control

```bash
# Restrict configuration directory
sudo chmod 750 /etc/wraith
sudo chown root:wraith /etc/wraith

# Protect keypair
sudo chmod 600 /etc/wraith/keypair.secret
sudo chown wraith:wraith /etc/wraith/keypair.secret
```

### SELinux/AppArmor

**SELinux policy (wraith.te):**
```
policy_module(wraith, 1.0.0)

type wraith_t;
type wraith_exec_t;
init_daemon_domain(wraith_t, wraith_exec_t)

allow wraith_t self:udp_socket create_socket_perms;
allow wraith_t self:capability { net_raw net_admin };
```

---

## See Also

- [Monitoring](monitoring.md)
- [Troubleshooting](troubleshooting.md)
- [Platform Support](../integration/platform-support.md)
