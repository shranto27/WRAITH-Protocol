# WRAITH-Mesh Architecture

**Document Version:** 1.0.0
**Last Updated:** 2025-11-28
**Client Version:** 1.0.0

---

## Overview

WRAITH-Mesh provides real-time visualization and diagnostics for the WRAITH network topology, enabling operators and advanced users to understand network structure and performance.

**Design Goals:**
- Visualize 1,000+ node networks smoothly
- Real-time updates with <100ms latency
- 3D and 2D visualization modes
- Interactive node inspection
- Network health monitoring

---

## Architecture Diagram

```
┌──────────────────────────────────────────────────────┐
│            Visualization UI                          │
│  ┌────────────────┐  ┌──────────────────────────┐   │
│  │  2D Graph      │  │     3D Graph             │   │
│  │  (D3.js)       │  │   (Three.js)             │   │
│  └────────────────┘  └──────────────────────────┘   │
└──────────────────────────────────────────────────────┘
                         │
┌──────────────────────────────────────────────────────┐
│         Data Collection Layer                        │
│  - Peer discovery                                    │
│  - Connection monitoring                             │
│  - Traffic analysis                                  │
│  - DHT routing table inspection                      │
└──────────────────────────────────────────────────────┘
                         │
┌──────────────────────────────────────────────────────┐
│         WRAITH Protocol Stack                        │
│  (network topology queries)                          │
└──────────────────────────────────────────────────────┘
```

---

## Components

### 1. Force-Directed Graph

**Layout Algorithm:** D3 force simulation

**Node Types:**
- Self (local node)
- Direct (direct peers)
- Relay (relay servers)
- Indirect (discovered via DHT)

**Link Properties:**
- Latency (affects distance)
- Bandwidth (affects thickness)
- Packet loss (affects opacity)

---

### 2. Network Data Collector

```rust
pub struct NetworkCollector {
    wraith: Arc<WraithClient>,
    cache: Arc<RwLock<NetworkState>>,
}

pub struct NetworkState {
    pub nodes: Vec<NodeInfo>,
    pub connections: Vec<Connection>,
    pub last_updated: SystemTime,
}

impl NetworkCollector {
    pub async fn collect(&self) -> Result<NetworkState> {
        let nodes = self.discover_nodes().await?;
        let connections = self.map_connections(&nodes).await?;

        Ok(NetworkState {
            nodes,
            connections,
            last_updated: SystemTime::now(),
        })
    }
}
```

---

### 3. Metrics Dashboard

**Metrics:**
- Total nodes
- Active connections
- Average latency
- Bandwidth utilization
- Packet loss rate
- DHT routing table size

---

## Performance Characteristics

**Rendering:**
- 1,000 nodes @ 60 FPS (2D)
- 1,000 nodes @ 30 FPS (3D)

**Update Frequency:**
- Network state: 1 second
- Connection metrics: 100ms

---

## See Also

- [Features](features.md)
- [Implementation](implementation.md)
- [Client Overview](../overview.md)
