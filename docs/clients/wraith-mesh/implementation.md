# WRAITH-Mesh Implementation

**Document Version:** 1.0.0
**Last Updated:** 2025-11-28
**Client Version:** 1.0.0

---

## Overview

Implementation details for WRAITH-Mesh, including D3.js graph rendering, network data collection, and performance optimization.

---

## Technology Stack

```json
{
  "dependencies": {
    "react": "^18.3.0",
    "d3": "^7.8.0",
    "three": "^0.160.0",
    "@tauri-apps/api": "^2.0.0"
  }
}
```

---

## Force-Directed Graph

```tsx
// src/components/NetworkGraph.tsx
import React, { useEffect, useRef } from 'react';
import * as d3 from 'd3';

export function NetworkGraph({ nodes, links }: Props) {
  const svgRef = useRef<SVGSVGElement>(null);

  useEffect(() => {
    if (!svgRef.current) return;

    const width = 1200;
    const height = 800;

    const svg = d3.select(svgRef.current)
      .attr('width', width)
      .attr('height', height);

    const simulation = d3.forceSimulation(nodes)
      .force('link', d3.forceLink(links)
        .id(d => d.id)
        .distance(d => 100 + d.latency)
      )
      .force('charge', d3.forceManyBody().strength(-300))
      .force('center', d3.forceCenter(width / 2, height / 2));

    const link = svg.append('g')
      .selectAll('line')
      .data(links)
      .join('line')
      .attr('stroke', '#999')
      .attr('stroke-width', d => Math.sqrt(d.bandwidth));

    const node = svg.append('g')
      .selectAll('circle')
      .data(nodes)
      .join('circle')
      .attr('r', 8)
      .attr('fill', d => getNodeColor(d.type));

    simulation.on('tick', () => {
      link
        .attr('x1', d => d.source.x)
        .attr('y1', d => d.source.y)
        .attr('x2', d => d.target.x)
        .attr('y2', d => d.target.y);

      node
        .attr('cx', d => d.x)
        .attr('cy', d => d.y);
    });

    return () => simulation.stop();
  }, [nodes, links]);

  return <svg ref={svgRef} />;
}
```

---

## Network Data Collection

```rust
// src/network/collector.rs
pub struct NetworkCollector {
    wraith: Arc<WraithClient>,
}

impl NetworkCollector {
    pub async fn collect(&self) -> Result<NetworkState> {
        let peers = self.wraith.get_connected_peers().await?;

        let mut nodes = Vec::new();
        let mut connections = Vec::new();

        for peer in peers {
            let info = self.wraith.get_peer_info(peer.id).await?;

            nodes.push(NodeInfo {
                id: peer.id,
                address: info.address,
                latency: info.latency,
                bandwidth: info.bandwidth,
            });

            connections.push(Connection {
                source: self.wraith.local_peer_id(),
                target: peer.id,
                latency: info.latency,
            });
        }

        Ok(NetworkState { nodes, connections })
    }
}
```

---

## Build and Deployment

```bash
npm run tauri build
```

---

## See Also

- [Architecture](architecture.md)
- [Features](features.md)
- [Client Overview](../overview.md)
