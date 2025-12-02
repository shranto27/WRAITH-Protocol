# WRAITH Protocol Troubleshooting Guide

**Version:** 1.0
**Last Updated:** 2025-12-01
**Maintainer:** WRAITH Operations Team

---

## Overview

This guide provides diagnostic procedures and solutions for common issues in WRAITH Protocol deployments.

---

## Quick Diagnostics

### Health Check Script

```bash
#!/bin/bash
# wraith-health-check.sh

echo "=== WRAITH Health Check ==="
echo

# 1. Service status
echo "1. Service Status:"
systemctl is-active wraith && echo "✓ Service running" || echo "✗ Service stopped"

# 2. Process check
echo "2. Process:"
pgrep -a wraith || echo "✗ No wraith process"

# 3. Port binding
echo "3. Port Binding:"
ss -tulnp | grep 5000 || echo "✗ Port 5000 not bound"

# 4. Config validation
echo "4. Configuration:"
wraith config --validate /etc/wraith/config.toml && echo "✓ Config valid" || echo "✗ Config invalid"

# 5. Connectivity
echo "5. Network Connectivity:"
timeout 5 wraith peers &>/dev/null && echo "✓ Connected to peers" || echo "✗ No peer connections"

# 6. Disk space
echo "6. Disk Space:"
df -h /var/lib/wraith | awk 'NR==2 {print $5 " used (" $4 " free)"}'

# 7. Memory usage
echo "7. Memory:"
ps aux | awk '/wraith/ && !/awk/ {print $6/1024 " MB"}'

# 8. Recent errors
echo "8. Recent Errors:"
journalctl -u wraith --since "1 hour ago" -p err --no-pager | wc -l
```

---

## Service Issues

### Service Won't Start

**Symptoms:**
- `systemctl start wraith` fails
- Service exits immediately

**Diagnosis:**

```bash
# Check service status
sudo systemctl status wraith -l

# Check recent logs
sudo journalctl -u wraith -n 100 --no-pager

# Test binary directly
sudo -u wraith /usr/local/bin/wraith daemon --config /etc/wraith/config.toml
```

**Common Causes:**

1. **Config File Error**
   ```bash
   # Validate config
   wraith config --validate /etc/wraith/config.toml

   # Fix: Edit config and correct syntax errors
   sudo nano /etc/wraith/config.toml
   ```

2. **Port Already in Use**
   ```bash
   # Check port
   sudo ss -tulnp | grep 5000

   # Fix: Kill conflicting process or change port
   sudo kill <PID>
   # Or edit config to use different port
   ```

3. **Permission Issues**
   ```bash
   # Check file ownership
   ls -la /var/lib/wraith/keys/

   # Fix: Set correct ownership
   sudo chown -R wraith:wraith /var/lib/wraith
   sudo chmod 700 /var/lib/wraith/keys
   sudo chmod 600 /var/lib/wraith/keys/*.enc
   ```

4. **Missing Key File**
   ```bash
   # Check key exists
   ls -la /var/lib/wraith/keys/node_key.enc

   # Fix: Generate new key or restore from backup
   wraith keygen -o /var/lib/wraith/keys/node_key.enc
   ```

### Service Crashes/Restarts

**Symptoms:**
- Service keeps restarting
- Frequent crashes in logs

**Diagnosis:**

```bash
# Check crash logs
sudo journalctl -u wraith --since "1 hour ago" | grep -i "panic\|segfault\|crash"

# Check core dumps
coredumpctl list wraith

# Analyze core dump
coredumpctl debug wraith
```

**Solutions:**

1. **Memory Limit Exceeded**
   ```bash
   # Check memory usage
   systemctl show wraith | grep MemoryCurrent

   # Fix: Increase memory limit
   sudo systemctl edit wraith
   # Add: [Service]
   #      MemoryMax=8G
   sudo systemctl daemon-reload
   sudo systemctl restart wraith
   ```

2. **Corrupted State**
   ```bash
   # Backup and reset state
   sudo systemctl stop wraith
   sudo mv /var/lib/wraith/data /var/lib/wraith/data.backup
   sudo mkdir /var/lib/wraith/data
   sudo chown wraith:wraith /var/lib/wraith/data
   sudo systemctl start wraith
   ```

---

## Connectivity Issues

### No Peer Connections

**Symptoms:**
- `wraith peers` shows 0 peers
- Cannot send/receive files

**Diagnosis:**

```bash
# Check listening ports
sudo ss -tulnp | grep wraith

# Check firewall
sudo iptables -L -n -v | grep 5000

# Test connectivity to bootstrap nodes
nc -zv bootstrap1.wraith.example 5000
```

**Solutions:**

1. **Firewall Blocking**
   ```bash
   # Allow WRAITH ports
   sudo iptables -A INPUT -p udp --dport 5000 -j ACCEPT
   sudo iptables -A INPUT -p tcp --dport 5000 -j ACCEPT
   ```

2. **NAT Issues**
   ```bash
   # Check NAT type
   wraith nat-detect

   # Enable STUN
   # Edit /etc/wraith/config.toml:
   # [discovery]
   # stun_servers = ["stun.l.google.com:19302"]
   ```

3. **Bootstrap Nodes Unreachable**
   ```bash
   # Test bootstrap nodes
   dig +short bootstrap1.wraith.example
   ping -c 3 bootstrap1.wraith.example

   # Fix: Update bootstrap nodes in config
   ```

### High Latency

**Symptoms:**
- Slow file transfers
- High ping times

**Diagnosis:**

```bash
# Check network latency
wraith ping <peer_id>

# Check for packet loss
sudo tcpdump -i eth0 -c 1000 port 5000 | grep -c "dropped"

# Check BBR congestion control
sysctl net.ipv4.tcp_congestion_control
```

**Solutions:**

1. **Enable BBR**
   ```bash
   sudo sysctl -w net.ipv4.tcp_congestion_control=bbr
   sudo sysctl -w net.core.default_qdisc=fq
   echo "net.ipv4.tcp_congestion_control=bbr" | sudo tee -a /etc/sysctl.conf
   ```

2. **Increase Buffer Sizes**
   ```bash
   sudo sysctl -w net.core.rmem_max=134217728
   sudo sysctl -w net.core.wmem_max=134217728
   ```

---

## Performance Issues

### High CPU Usage

**Symptoms:**
- CPU usage consistently >80%
- System slowdown

**Diagnosis:**

```bash
# Check CPU usage
top -p $(pgrep wraith)

# Profile CPU usage
sudo perf record -p $(pgrep wraith) -g -- sleep 10
sudo perf report
```

**Solutions:**

1. **Obfuscation Overhead**
   ```bash
   # Reduce obfuscation (testing only!)
   # Edit /etc/wraith/config.toml:
   # [obfuscation]
   # mode = "none"  # Or "tls" instead of "statistical"
   ```

2. **Too Many Connections**
   ```bash
   # Reduce max connections
   # Edit /etc/wraith/config.toml:
   # [network]
   # max_connections = 500  # Down from 1000
   ```

3. **Enable SIMD**
   ```bash
   # Rebuild with SIMD acceleration
   cd WRAITH-Protocol
   cargo build --release --features simd-avx2
   sudo install -m 755 target/release/wraith /usr/local/bin/
   sudo systemctl restart wraith
   ```

### Slow Transfer Speeds

**Symptoms:**
- Transfer speeds <10 Mbps
- Lower than network capacity

**Diagnosis:**

```bash
# Check transfer stats
wraith stats

# Check network throughput
iftop -i eth0

# Check BBR state
ss -ti | grep bbr
```

**Solutions:**

1. **Increase Chunk Size**
   ```bash
   # Edit /etc/wraith/config.toml:
   # [transfer]
   # chunk_size = 524288  # 512 KiB instead of 256 KiB
   ```

2. **Enable AF_XDP (Linux only)**
   ```bash
   # Rebuild with AF_XDP support
   cargo build --release --features af_xdp

   # Update config:
   # [network]
   # use_af_xdp = true
   ```

3. **Use Multiple Paths**
   ```bash
   # Enable multi-path transfer
   # [discovery]
   # relay_enabled = true  # Use both direct and relayed paths
   ```

### High Memory Usage

**Symptoms:**
- Memory usage >2 GB
- OOM killer triggered

**Diagnosis:**

```bash
# Check memory usage
ps aux | grep wraith | awk '{print $6/1024 " MB"}'

# Check for memory leaks
valgrind --leak-check=full wraith daemon --config /etc/wraith/config.toml
```

**Solutions:**

1. **Reduce Concurrent Transfers**
   ```bash
   # Edit /etc/wraith/config.toml:
   # [transfer]
   # max_concurrent_transfers = 50  # Down from 100
   ```

2. **Reduce Chunk Size**
   ```bash
   # [transfer]
   # chunk_size = 131072  # 128 KiB instead of 256 KiB
   ```

---

## Cryptographic Issues

### Key Decryption Failed

**Symptoms:**
- "Failed to decrypt key file" error
- Service won't start

**Diagnosis:**

```bash
# Verify key file
wraith key info /var/lib/wraith/keys/node_key.enc

# Check file permissions
ls -la /var/lib/wraith/keys/node_key.enc
```

**Solutions:**

1. **Wrong Password**
   ```bash
   # Restore from backup
   sudo cp /backup/wraith/keys/node_key.enc.backup \
     /var/lib/wraith/keys/node_key.enc
   ```

2. **Corrupted Key File**
   ```bash
   # Verify checksum
   sha256sum /var/lib/wraith/keys/node_key.enc
   # Compare with backup checksum

   # If corrupted, restore from backup
   ```

### Handshake Failures

**Symptoms:**
- "Handshake failed" in logs
- Cannot establish sessions

**Diagnosis:**

```bash
# Enable debug logging
sudo sed -i 's/level = "info"/level = "debug"/' /etc/wraith/config.toml
sudo systemctl restart wraith

# Check handshake logs
sudo journalctl -u wraith | grep "handshake"
```

**Solutions:**

1. **Clock Skew**
   ```bash
   # Sync system time
   sudo ntpdate pool.ntp.org

   # Or with systemd-timesyncd
   sudo timedatectl set-ntp true
   ```

2. **Incompatible Versions**
   ```bash
   # Check protocol version
   wraith --version

   # Upgrade to compatible version
   git fetch --all
   git checkout v0.8.0
   cargo build --release
   ```

---

## File Transfer Issues

### Transfer Stuck

**Symptoms:**
- Transfer progress at 0% or stopped
- No error messages

**Diagnosis:**

```bash
# Check transfer status
wraith transfers

# Check peer connectivity
wraith peers | grep <peer_id>

# Check logs
sudo journalctl -u wraith | grep <transfer_id>
```

**Solutions:**

1. **Resume Transfer**
   ```bash
   wraith transfer <transfer_id> --resume
   ```

2. **Cancel and Retry**
   ```bash
   wraith transfer <transfer_id> --cancel
   wraith send <file> <peer_id>
   ```

### Chunk Verification Failed

**Symptoms:**
- "Chunk hash mismatch" errors
- Transfer fails during verification

**Diagnosis:**

```bash
# Check file integrity
sha256sum <file>

# Verify BLAKE3 implementation
wraith verify <file>
```

**Solutions:**

1. **Network Corruption**
   ```bash
   # Enable FEC (future feature)
   # [transfer]
   # enable_fec = true

   # For now: retry transfer
   wraith send <file> <peer_id> --retry
   ```

---

## Common Error Messages

| Error | Cause | Solution |
|-------|-------|----------|
| "Permission denied" | File permissions | `sudo chown wraith:wraith <file>` |
| "Port already in use" | Conflicting service | Change port or kill process |
| "Connection refused" | Firewall/NAT | Check firewall rules |
| "Handshake timeout" | Network latency | Increase timeout in config |
| "Out of memory" | Too many transfers | Reduce max_concurrent_transfers |
| "Invalid frame" | Corrupted data | Check network, retry transfer |
| "Key not found" | Missing key file | Generate new key or restore backup |

---

## Log Analysis

### Important Log Patterns

```bash
# Connection issues
sudo journalctl -u wraith | grep -i "connection\|refused\|timeout"

# Cryptographic errors
sudo journalctl -u wraith | grep -i "decrypt\|verify\|handshake"

# Transfer errors
sudo journalctl -u wraith | grep -i "transfer\|chunk\|hash"

# Performance issues
sudo journalctl -u wraith | grep -i "slow\|congestion\|bandwidth"
```

---

## Getting Help

**Before asking for help, collect:**

```bash
# System info
uname -a > debug.txt
wraith --version >> debug.txt

# Config (redact sensitive info)
grep -v "key\|password" /etc/wraith/config.toml >> debug.txt

# Recent logs
sudo journalctl -u wraith --since "1 hour ago" --no-pager >> debug.txt

# Network status
ss -tulnp | grep wraith >> debug.txt

# Resource usage
top -b -n 1 | grep wraith >> debug.txt
```

**Support Channels:**
- GitHub Issues: https://github.com/doublegate/WRAITH-Protocol/issues
- Community Forum: https://forum.wraith.example
- Email: support@wraith.example

---

## Advanced Debugging

### Enable Trace Logging

```bash
# Maximum verbosity (VERY noisy!)
sudo sed -i 's/level = "info"/level = "trace"/' /etc/wraith/config.toml
sudo systemctl restart wraith

# View trace logs
sudo journalctl -u wraith -f --output=short-precise
```

### Network Packet Capture

```bash
# Capture WRAITH traffic
sudo tcpdump -i eth0 -w wraith.pcap port 5000

# Analyze with Wireshark
wireshark wraith.pcap

# Or with tshark
tshark -r wraith.pcap -V
```

### Profiling

```bash
# CPU profiling
sudo perf record -F 99 -p $(pgrep wraith) -g -- sleep 30
sudo perf report

# Memory profiling
valgrind --tool=massif wraith daemon --config /etc/wraith/config.toml

# Flame graph
sudo perf record -F 99 -a -g -- sleep 60
sudo perf script | stackcollapse-perf.pl | flamegraph.pl > wraith.svg
```
