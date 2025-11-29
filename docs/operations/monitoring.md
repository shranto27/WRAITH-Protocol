# WRAITH Protocol Monitoring

**Document Version:** 1.0.0
**Last Updated:** 2025-11-28
**Status:** Operations Documentation

---

## Overview

This document describes monitoring strategies, metrics collection, and observability for WRAITH Protocol deployments.

**Monitoring Objectives:**
- Service health and availability
- Performance metrics (throughput, latency)
- Resource utilization (CPU, memory, network)
- Security events and anomalies
- Error rates and debugging information

---

## Metrics Collection

### Prometheus Integration

**Configuration:**
```toml
# config.toml
[monitoring]
enabled = true
prometheus_port = 9090
metrics_path = "/metrics"
update_interval = "15s"
```

**Exposed Metrics:**
```
# Session metrics
wraith_sessions_active{state="established"} 42
wraith_sessions_total{result="success"} 1523
wraith_sessions_total{result="failure"} 8
wraith_handshake_duration_seconds{quantile="0.5"} 0.0015
wraith_handshake_duration_seconds{quantile="0.99"} 0.0042

# Transfer metrics
wraith_bytes_sent_total 15680000000
wraith_bytes_received_total 8430000000
wraith_transfers_active 12
wraith_transfer_throughput_bps{direction="send"} 850000000

# DHT metrics
wraith_dht_lookups_total{result="success"} 2341
wraith_dht_lookup_duration_seconds{quantile="0.95"} 0.245
wraith_dht_stored_values 156

# System metrics
wraith_cpu_usage_percent 32.5
wraith_memory_usage_bytes 125829120
wraith_file_descriptors_open 87

# Error metrics
wraith_errors_total{type="network"} 12
wraith_errors_total{type="crypto"} 0
wraith_errors_total{type="timeout"} 5
```

**Prometheus scrape config:**
```yaml
scrape_configs:
  - job_name: 'wraith'
    static_configs:
      - targets: ['localhost:9090']
    scrape_interval: 15s
    scrape_timeout: 10s
```

### Grafana Dashboards

**Dashboard JSON:**
```json
{
  "dashboard": {
    "title": "WRAITH Protocol",
    "panels": [
      {
        "title": "Active Sessions",
        "targets": [
          {
            "expr": "wraith_sessions_active"
          }
        ],
        "type": "graph"
      },
      {
        "title": "Transfer Throughput",
        "targets": [
          {
            "expr": "rate(wraith_bytes_sent_total[5m]) * 8"
          },
          {
            "expr": "rate(wraith_bytes_received_total[5m]) * 8"
          }
        ],
        "type": "graph"
      }
    ]
  }
}
```

---

## Logging

### Structured Logging

**Configuration:**
```toml
[logging]
level = "info"
format = "json"
output = "/var/log/wraith/wraith.log"
max_size = "100MB"
max_backups = 10
compress = true
```

**Log Levels:**
- `error`: Critical failures (service disruption)
- `warn`: Degraded performance (timeouts, retries)
- `info`: Normal operations (session established, file transferred)
- `debug`: Detailed diagnostics (packet dumps, state transitions)
- `trace`: Verbose debugging (crypto operations, DHT queries)

**Log Format:**
```json
{
  "timestamp": "2025-11-28T10:15:30.123Z",
  "level": "info",
  "message": "Session established",
  "fields": {
    "peer_id": "ed25519:abc123...",
    "remote_addr": "192.0.2.10:41641",
    "handshake_duration_ms": 1.5
  }
}
```

### Log Aggregation

**ELK Stack (Elasticsearch + Logstash + Kibana):**

**Logstash config:**
```
input {
  file {
    path => "/var/log/wraith/wraith.log"
    codec => json
  }
}

filter {
  if [level] == "error" {
    mutate {
      add_tag => ["alert"]
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

**Kibana query:**
```
level:error AND fields.error_type:network
```

---

## Health Checks

### Liveness Probe

```bash
#!/bin/bash
# /usr/local/bin/wraith-healthcheck

# Check if process running
if ! pgrep -f wraith-cli >/dev/null; then
    echo "CRITICAL: WRAITH process not running"
    exit 2
fi

# Check if port listening
if ! ss -lun | grep -q ":41641"; then
    echo "CRITICAL: Port 41641 not listening"
    exit 2
fi

echo "OK: WRAITH healthy"
exit 0
```

### Readiness Probe

```bash
#!/bin/bash
# Check DHT connectivity

DHT_PEERS=$(wraith-cli dht peers | wc -l)

if [ "$DHT_PEERS" -lt 3 ]; then
    echo "WARNING: Insufficient DHT peers ($DHT_PEERS)"
    exit 1
fi

echo "OK: DHT ready ($DHT_PEERS peers)"
exit 0
```

### Kubernetes Probes

```yaml
livenessProbe:
  exec:
    command:
      - /usr/local/bin/wraith-healthcheck
  initialDelaySeconds: 30
  periodSeconds: 10
  timeoutSeconds: 5
  failureThreshold: 3

readinessProbe:
  exec:
    command:
      - /usr/local/bin/wraith-readiness
  initialDelaySeconds: 10
  periodSeconds: 5
  timeoutSeconds: 3
```

---

## Alerting

### Alert Rules (Prometheus Alertmanager)

```yaml
groups:
  - name: wraith
    interval: 30s
    rules:
      - alert: WraithDown
        expr: up{job="wraith"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "WRAITH node down"
          description: "WRAITH node {{ $labels.instance }} is down"

      - alert: HighErrorRate
        expr: rate(wraith_errors_total[5m]) > 10
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High error rate detected"

      - alert: LowThroughput
        expr: rate(wraith_bytes_sent_total[5m]) < 1000000
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "Transfer throughput below 1 MB/s"

      - alert: MemoryExhaustion
        expr: wraith_memory_usage_bytes > 1000000000
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "Memory usage exceeds 1 GB"
```

### PagerDuty Integration

```yaml
receivers:
  - name: 'pagerduty'
    pagerduty_configs:
      - service_key: '<YOUR_SERVICE_KEY>'
        description: '{{ .GroupLabels.alertname }}'
```

---

## Performance Monitoring

### System Metrics

**CPU profiling:**
```bash
# Generate CPU profile
RUST_LOG=trace cargo run --release -- daemon &
PID=$!
perf record -g -p $PID sleep 60
perf report
```

**Memory profiling:**
```bash
# Heap profiling
valgrind --tool=massif --massif-out-file=massif.out wraith-cli daemon
ms_print massif.out
```

### Network Monitoring

**Packet capture:**
```bash
# Capture WRAITH traffic
sudo tcpdump -i eth0 'udp port 41641' -w wraith-traffic.pcap

# Analyze with Wireshark
wireshark wraith-traffic.pcap
```

**Bandwidth monitoring:**
```bash
# Install iftop
sudo apt install iftop

# Monitor interface
sudo iftop -i eth0 -f 'udp port 41641'
```

---

## Tracing

### OpenTelemetry

**Configuration:**
```rust
use opentelemetry::{global, trace::Tracer};
use tracing_opentelemetry::OpenTelemetryLayer;

fn setup_tracing() {
    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name("wraith-protocol")
        .install_simple()
        .unwrap();

    let telemetry = OpenTelemetryLayer::new(tracer);

    tracing_subscriber::registry()
        .with(telemetry)
        .init();
}
```

**Distributed tracing:**
```rust
#[tracing::instrument]
async fn transfer_file(file_path: PathBuf) -> Result<()> {
    // Automatically traced with span context
    let session = establish_session().await?;
    let transfer = FileTransfer::new(session, Default::default());
    transfer.send_file(&file_path, None).await?;
    Ok(())
}
```

---

## Security Monitoring

### Intrusion Detection

**Suspicious patterns:**
- High handshake failure rate (potential DoS)
- Repeated authentication failures
- Unusual traffic patterns (spikes, DDoS)
- DHT query flooding

**Alert example:**
```yaml
- alert: SuspiciousHandshakes
  expr: rate(wraith_sessions_total{result="failure"}[1m]) > 50
  for: 2m
  labels:
    severity: warning
  annotations:
    summary: "High handshake failure rate (potential attack)"
```

### Audit Logging

**Security events:**
```json
{
  "timestamp": "2025-11-28T10:20:15.456Z",
  "level": "warn",
  "message": "Authentication failed",
  "fields": {
    "remote_addr": "203.0.113.100:52341",
    "reason": "invalid_signature",
    "attempt_count": 5
  }
}
```

---

## Dashboard Examples

### CLI Dashboard

```bash
# Install wraith-mon (example monitoring CLI)
wraith-cli mon --interval 1s

Output:
┌─ WRAITH Node Status ──────────────────────────┐
│ Uptime: 5d 12h 34m                            │
│ Sessions: 42 active, 1,523 total              │
│ Throughput: ↑ 850 Mbps, ↓ 420 Mbps           │
│ DHT Peers: 156                                 │
│ Memory: 125 MB / 2 GB (6.25%)                 │
│ CPU: 32.5%                                     │
└───────────────────────────────────────────────┘

Recent Transfers:
✓ file1.bin → peer:abc123 (1.2 GB, 15s, 640 Mbps)
✓ file2.bin → peer:def456 (500 MB, 6s, 666 Mbps)
⏳ file3.bin → peer:ghi789 (2.1 GB, 45% complete)
```

---

## See Also

- [Deployment Guide](deployment-guide.md)
- [Troubleshooting](troubleshooting.md)
- [Performance Benchmarks](../testing/performance-benchmarks.md)
