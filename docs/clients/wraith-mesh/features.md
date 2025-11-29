# WRAITH-Mesh Features

**Document Version:** 1.0.0
**Last Updated:** 2025-11-28
**Client Version:** 1.0.0

---

## Overview

WRAITH-Mesh provides comprehensive network visualization and diagnostics tools for understanding WRAITH network topology and performance.

---

## Core Features

### 1. Network Graph Visualization

**User Stories:**
- As a network operator, I can see all connected peers
- As a user, I can identify relay servers and direct connections
- As an operator, I can spot network bottlenecks visually

**Visualization Modes:**
- 2D Force-Directed Graph (D3.js)
- 3D Network Globe (Three.js)
- Tree View (hierarchical)
- Matrix View (adjacency matrix)

---

### 2. Node Inspection

**User Stories:**
- As a user, I can click any node to see details
- As an operator, I can view connection history
- As an operator, I can test connection to specific node

**Node Details:**
- Peer ID
- IP address:port
- Connection type (direct/relay)
- Latency
- Bandwidth
- Uptime
- Software version

---

### 3. Connection Metrics

**Real-Time Metrics:**
- Latency (ping time)
- Bandwidth (upload/download)
- Packet loss
- Jitter
- Connection quality score

**Historical Data:**
- 24-hour graphs
- Trend analysis
- Export to CSV

---

### 4. DHT Routing Table

**User Stories:**
- As an operator, I can inspect DHT routing table
- As an operator, I can see k-bucket distribution
- As an operator, I can verify peer reachability

**Visualization:**
- K-bucket viewer
- Peer distance histogram
- Routing path tracer

---

## Advanced Features

### Network Diagnostics

**Tools:**
- Ping test (measure latency)
- Bandwidth test (measure throughput)
- Path trace (show routing path)
- Connection test (verify reachability)

### Traffic Analysis

**User Stories:**
- As an operator, I can view traffic flows
- As an operator, I can identify heavy users
- As an operator, I can detect anomalies

**Metrics:**
- Bytes sent/received per peer
- Protocol breakdown (DHT, file transfer, etc.)
- Traffic heatmap

---

## User Interface

### Main View

```
┌─────────────────────────────────────────┐
│  WRAITH Mesh                    ─ □ ×   │
├─────────────────────────────────────────┤
│  Nodes: 127 | Connections: 356         │
│  Avg Latency: 45ms | Loss: 0.1%        │
├─────────────────────────────────────────┤
│                                         │
│         [Network Graph Area]            │
│                                         │
│           ●─────●                       │
│          ╱│╲   ╱│╲                      │
│         ● │ ● ● │ ●                     │
│         │╲│╱│╲│╱│                       │
│         ● ● ● ● ●                       │
│                                         │
├─────────────────────────────────────────┤
│  Selected: Node ABC123...               │
│  Latency: 12ms | Bandwidth: 50 Mbps    │
│  [Ping] [Traceroute] [Disconnect]      │
└─────────────────────────────────────────┘
```

---

## See Also

- [Architecture](architecture.md)
- [Implementation](implementation.md)
- [Client Overview](../overview.md)
