# WRAITH Protocol Monitoring & Observability Guide

**Version:** 1.0
**Last Updated:** 2025-12-01
**Maintainer:** WRAITH Operations Team

---

## Overview

This guide provides comprehensive monitoring strategies for WRAITH Protocol deployments, including metrics collection, alerting, logging, and observability best practices.

---

## Monitoring Stack

**Recommended Stack:**
- **Metrics**: Prometheus + Grafana
- **Logs**: ELK Stack (Elasticsearch, Logstash, Kibana)  or Loki
- **Tracing**: Jaeger (future)
- **Alerting**: Alertmanager + PagerDuty

---

## Metrics Collection

### Enable Metrics Endpoint

Edit `/etc/wraith/config.toml`:

```toml
[metrics]
enabled = true
listen_addr = "127.0.0.1:9090"
path = "/metrics"
format = "prometheus"  # Options: prometheus, json
```

Restart service:

```bash
sudo systemctl restart wraith
```

Test endpoint:

```bash
curl http://localhost:9090/metrics
```

### Key Metrics

#### System Metrics

**CPU & Memory:**
```prometheus
# CPU usage (percentage)
wraith_cpu_usage_percent

# Memory usage (bytes)
wraith_memory_usage_bytes

# Open file descriptors
wraith_open_fds
```

**Network:**
```prometheus
# Bytes sent/received
wraith_network_bytes_sent_total
wraith_network_bytes_received_total

# Packets sent/received
wraith_network_packets_sent_total
wraith_network_packets_received_total

# Network errors
wraith_network_errors_total
```

#### Protocol Metrics

**Connections:**
```prometheus
# Active connections
wraith_connections_active

# Total connections
wraith_connections_total

# Failed connections
wraith_connections_failed_total

# Connection duration (seconds)
wraith_connection_duration_seconds_bucket
```

**Sessions:**
```prometheus
# Active sessions
wraith_sessions_active

# Session state (Handshaking, Established, etc.)
wraith_session_state{state="established"}

# Handshake duration
wraith_handshake_duration_seconds_bucket

# Handshake failures
wraith_handshake_failures_total
```

**Transfers:**
```prometheus
# Active transfers
wraith_transfers_active

# Total transfers
wraith_transfers_total{direction="send|receive"}

# Transfer bytes
wraith_transfer_bytes_total{direction="send|receive"}

# Transfer speed (bytes/sec)
wraith_transfer_speed_bytes_per_second

# Transfer failures
wraith_transfer_failures_total{reason="timeout|error|cancelled"}
```

**Cryptography:**
```prometheus
# Encryption operations
wraith_crypto_encrypt_total
wraith_crypto_decrypt_total

# Encryption failures
wraith_crypto_failures_total{operation="encrypt|decrypt|sign|verify"}

# Key ratchets
wraith_ratchet_total{type="symmetric|dh"}

# BLAKE3 hash operations
wraith_hash_operations_total
```

**Obfuscation:**
```prometheus
# Padding operations
wraith_padding_operations_total{mode="power_of_two|statistical"}

# Timing delays added (milliseconds)
wraith_timing_delay_milliseconds_total

# Protocol mimicry
wraith_mimicry_operations_total{protocol="tls|websocket|doh"}
```

**Discovery:**
```prometheus
# DHT nodes known
wraith_dht_nodes_total

# DHT lookups
wraith_dht_lookups_total{result="success|failure"}

# Relay connections
wraith_relay_connections_active

# NAT type
wraith_nat_type{type="full_cone|restricted|symmetric"}
```

---

## Prometheus Configuration

### Prometheus Scrape Config

Add to `prometheus.yml`:

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'wraith'
    static_configs:
      - targets: ['localhost:9090']
        labels:
          instance: 'node-01'
          environment: 'production'

  # For multi-node deployment
  - job_name: 'wraith-cluster'
    static_configs:
      - targets:
        - 'node-01.wraith.example:9090'
        - 'node-02.wraith.example:9090'
        - 'node-03.wraith.example:9090'
```

### Recording Rules

Create `wraith_rules.yml`:

```yaml
groups:
  - name: wraith_recording_rules
    interval: 1m
    rules:
      # Transfer throughput (5m average)
      - record: wraith_transfer_throughput_5m
        expr: rate(wraith_transfer_bytes_total[5m])

      # Connection success rate
      - record: wraith_connection_success_rate
        expr: |
          1 - (
            rate(wraith_connections_failed_total[5m]) /
            rate(wraith_connections_total[5m])
          )

      # Handshake latency (p95)
      - record: wraith_handshake_latency_p95
        expr: histogram_quantile(0.95, rate(wraith_handshake_duration_seconds_bucket[5m]))

      # Crypto operations per second
      - record: wraith_crypto_ops_per_second
        expr: |
          rate(wraith_crypto_encrypt_total[1m]) +
          rate(wraith_crypto_decrypt_total[1m])
```

---

## Alerting Rules

### Alertmanager Configuration

Create `wraith_alerts.yml`:

```yaml
groups:
  - name: wraith_alerts
    rules:
      # Critical: Service down
      - alert: WraithServiceDown
        expr: up{job="wraith"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "WRAITH service down on {{ $labels.instance }}"
          description: "Service has been down for 1 minute"

      # Critical: High error rate
      - alert: WraithHighErrorRate
        expr: |
          rate(wraith_connections_failed_total[5m]) /
          rate(wraith_connections_total[5m]) > 0.1
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High connection error rate on {{ $labels.instance }}"
          description: "Connection failure rate is {{ $value | humanizePercentage }}"

      # Warning: High memory usage
      - alert: WraithHighMemory
        expr: wraith_memory_usage_bytes > 8e9  # 8 GB
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage on {{ $labels.instance }}"
          description: "Memory usage is {{ $value | humanize }}B"

      # Warning: High CPU usage
      - alert: WraithHighCPU
        expr: wraith_cpu_usage_percent > 80
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "High CPU usage on {{ $labels.instance }}"
          description: "CPU usage is {{ $value }}%"

      # Warning: Slow transfers
      - alert: WraithSlowTransfers
        expr: wraith_transfer_speed_bytes_per_second < 1e6  # < 1 MB/s
        for: 15m
        labels:
          severity: warning
        annotations:
          summary: "Slow transfer speeds on {{ $labels.instance }}"
          description: "Transfer speed is {{ $value | humanize }}B/s"

      # Warning: Few peers
      - alert: WraithFewPeers
        expr: wraith_dht_nodes_total < 5
        for: 15m
        labels:
          severity: warning
        annotations:
          summary: "Low peer count on {{ $labels.instance }}"
          description: "Only {{ $value }} peers connected"

      # Info: High transfer volume
      - alert: WraithHighTransferVolume
        expr: rate(wraith_transfer_bytes_total[1h]) > 1e11  # > 100 GB/hr
        for: 1h
        labels:
          severity: info
        annotations:
          summary: "High transfer volume on {{ $labels.instance }}"
          description: "Transfer rate is {{ $value | humanize }}B/s"
```

---

## Grafana Dashboards

### Create WRAITH Dashboard

Import JSON template or create panels:

**Panel 1: Service Health**
```promql
up{job="wraith"}
```

**Panel 2: Active Connections**
```promql
wraith_connections_active
```

**Panel 3: Transfer Throughput**
```promql
rate(wraith_transfer_bytes_total[5m])
```

**Panel 4: Handshake Latency (p50, p95, p99)**
```promql
histogram_quantile(0.50, rate(wraith_handshake_duration_seconds_bucket[5m]))
histogram_quantile(0.95, rate(wraith_handshake_duration_seconds_bucket[5m]))
histogram_quantile(0.99, rate(wraith_handshake_duration_seconds_bucket[5m]))
```

**Panel 5: Error Rate**
```promql
rate(wraith_connections_failed_total[5m]) / rate(wraith_connections_total[5m])
```

**Panel 6: CPU & Memory**
```promql
wraith_cpu_usage_percent
wraith_memory_usage_bytes / 1e9  # Convert to GB
```

**Panel 7: Network I/O**
```promql
rate(wraith_network_bytes_sent_total[5m])
rate(wraith_network_bytes_received_total[5m])
```

**Panel 8: DHT Peers**
```promql
wraith_dht_nodes_total
```

---

## Logging

### Configure Structured Logging

Edit `/etc/wraith/config.toml`:

```toml
[logging]
level = "info"  # Options: trace, debug, info, warn, error
format = "json"  # Options: json, plain
file = "/var/lib/wraith/logs/wraith.log"
max_size = "100MB"
max_age = "30d"
max_backups = 10
compress = true
```

### Log Levels

**Trace**: Very detailed, for deep debugging
**Debug**: Detailed operational info
**Info**: Normal operational messages
**Warn**: Warning conditions
**Error**: Error conditions

**Recommendation:**
- Production: `info` or `warn`
- Staging: `debug`
- Development: `trace`

### Log Aggregation (ELK)

#### Logstash Configuration

Create `/etc/logstash/conf.d/wraith.conf`:

```conf
input {
  file {
    path => "/var/lib/wraith/logs/wraith.log"
    type => "wraith"
    codec => "json"
  }
}

filter {
  if [type] == "wraith" {
    # Parse timestamp
    date {
      match => [ "timestamp", "ISO8601" ]
      target => "@timestamp"
    }

    # Extract log level
    mutate {
      uppercase => [ "level" ]
    }

    # Add host info
    mutate {
      add_field => {
        "host" => "%{host.name}"
      }
    }

    # Extract IP addresses
    grok {
      match => { "message" => "%{IP:src_ip}.*%{IP:dst_ip}" }
      tag_on_failure => []
    }
  }
}

output {
  elasticsearch {
    hosts => ["localhost:9200"]
    index => "wraith-%{+YYYY.MM.dd}"
  }
}
```

#### Kibana Visualizations

**Common Searches:**

```
# Errors in last hour
level:ERROR AND @timestamp:[now-1h TO now]

# Failed handshakes
message:"handshake failed"

# Slow transfers
message:"slow transfer" OR message:"timeout"

# By IP address
src_ip:"192.168.1.100"

# By peer ID
peer_id:"abc123..."
```

---

## Health Checks

### HTTP Health Endpoint

Enable in config:

```toml
[health]
enabled = true
listen_addr = "127.0.0.1:8080"
path = "/health"
```

Response format:

```json
{
  "status": "healthy",
  "version": "0.8.0",
  "uptime_seconds": 3600,
  "connections": 42,
  "transfers_active": 5,
  "peers": 128,
  "checks": {
    "dht": "ok",
    "storage": "ok",
    "network": "ok"
  }
}
```

### Kubernetes Liveness/Readiness Probes

```yaml
livenessProbe:
  httpGet:
    path: /health
    port: 8080
  initialDelaySeconds: 30
  periodSeconds: 10
  timeoutSeconds: 5
  failureThreshold: 3

readinessProbe:
  httpGet:
    path: /ready
    port: 8080
  initialDelaySeconds: 10
  periodSeconds: 5
  timeoutSeconds: 3
  failureThreshold: 3
```

---

## Tracing (Future)

### OpenTelemetry Integration

```toml
[tracing]
enabled = true
exporter = "otlp"  # or "jaeger", "zipkin"
endpoint = "localhost:4317"
sample_rate = 0.1  # Sample 10% of requests
```

Trace spans:
- Handshake execution
- File transfer operations
- Crypto operations
- DHT lookups

---

## Performance Monitoring

### Benchmarking

```bash
# Built-in benchmarks
cargo bench --workspace

# Specific benchmark
cargo bench -p wraith-core frame_parsing

# Compare results
cargo bench --workspace -- --save-baseline main
# Make changes
cargo bench --workspace -- --baseline main
```

### Continuous Performance Monitoring

Set up regression alerts:

```yaml
# In prometheus
- alert: WraithPerformanceRegression
  expr: |
    wraith_transfer_throughput_5m <
    0.9 * wraith_transfer_throughput_5m offset 7d
  for: 1h
  annotations:
    summary: "Performance regression detected"
```

---

## Capacity Planning

### Key Metrics to Track

1. **Transfer Capacity**
   - Current: X GB/day
   - Growth rate: Y% per month
   - Projected capacity: Z GB/day in 6 months

2. **Storage Growth**
   - Current usage: X GB
   - Growth rate: Y GB/month
   - Capacity planning: Expand at 80% utilization

3. **Connection Scaling**
   - Current: X connections
   - Max capacity: Y connections (based on benchmarks)
   - Scale out at: 80% of max capacity

### Scaling Thresholds

| Metric | Warn | Critical | Action |
|--------|------|----------|--------|
| CPU Usage | 70% | 85% | Scale up/out |
| Memory Usage | 70% | 85% | Scale up |
| Disk Usage | 70% | 85% | Add storage |
| Transfer Speed | <50 Mbps | <10 Mbps | Investigate |
| Error Rate | >5% | >10% | Incident |
| Peer Count | <10 | <5 | Check DHT |

---

## SLA Monitoring

### Define SLOs (Service Level Objectives)

**Example SLOs:**
- **Availability**: 99.9% uptime (43 min downtime/month)
- **Latency**: p95 handshake <500ms
- **Throughput**: Avg >100 Mbps
- **Error Rate**: <1% failed transfers

### SLI (Service Level Indicators)

```promql
# Availability (last 30 days)
avg_over_time(up{job="wraith"}[30d]) * 100

# Latency p95
histogram_quantile(0.95, rate(wraith_handshake_duration_seconds_bucket[30d]))

# Throughput
avg_over_time(rate(wraith_transfer_bytes_total[30d])[30d:1d])

# Error rate
sum(rate(wraith_transfer_failures_total[30d])) /
sum(rate(wraith_transfers_total[30d]))
```

---

## Compliance Monitoring

### Audit Logging

Enable audit logs:

```toml
[audit]
enabled = true
file = "/var/lib/wraith/logs/audit.log"
events = ["key_rotation", "config_change", "transfer_start", "transfer_complete"]
```

Audit log format:

```json
{
  "timestamp": "2025-12-01T10:00:00Z",
  "event": "key_rotation",
  "user": "admin@example.com",
  "result": "success",
  "details": {
    "old_key_id": "abc123",
    "new_key_id": "def456"
  }
}
```

---

## Best Practices

1. **Retention Policies**
   - Metrics: 30 days high-res, 1 year aggregated
   - Logs: 90 days active, 1 year archived
   - Traces: 7 days

2. **Alert Fatigue**
   - Tune thresholds to reduce false positives
   - Use tiered severity (critical, warning, info)
   - Implement alert suppression during maintenance

3. **Dashboard Design**
   - Top-level: Service health overview
   - Second-level: Component-specific metrics
   - Third-level: Deep dive / troubleshooting

4. **Regular Reviews**
   - Weekly: Review alerts and incidents
   - Monthly: Capacity planning
   - Quarterly: SLO compliance review

---

## Monitoring Checklist

- [ ] Prometheus scraping WRAITH metrics
- [ ] Grafana dashboards created
- [ ] Alertmanager configured
- [ ] PagerDuty/OpsGenie integration
- [ ] Log aggregation (ELK/Loki)
- [ ] Health checks enabled
- [ ] SLO baselines defined
- [ ] Runbooks linked to alerts
- [ ] On-call rotation defined
- [ ] Escalation procedures documented
