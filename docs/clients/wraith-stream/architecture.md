# WRAITH-Stream Architecture

**Document Version:** 1.0.0
**Last Updated:** 2025-11-28
**Client Version:** 1.0.0

---

## Overview

WRAITH-Stream enables encrypted peer-to-peer media streaming with adaptive bitrate, supporting both live and video-on-demand content distribution.

**Design Goals:**
- Support 100+ concurrent viewers per stream
- Adaptive bitrate (240p to 4K)
- Sub-3-second startup latency
- Live streaming with <5s latency
- Encrypted HLS/DASH segments

---

## Architecture Diagram

```
┌──────────────────────────────────────────────────────┐
│            Streaming Client                          │
│  ┌────────────────┐  ┌──────────────────────────┐   │
│  │  Player UI     │  │   Broadcaster UI         │   │
│  │ (video.js/HLS) │  │   (OBS/FFmpeg)           │   │
│  └────────────────┘  └──────────────────────────┘   │
└──────────────────────────────────────────────────────┘
                         │
┌──────────────────────────────────────────────────────┐
│         Transcoding Engine (FFmpeg)                  │
│  - Multi-bitrate transcoding                         │
│  - HLS/DASH segmentation                             │
│  - AV1/H.264 encoding                                │
└──────────────────────────────────────────────────────┘
                         │
┌──────────────────────────────────────────────────────┐
│         Segment Storage (Encrypted DHT)              │
│  - XChaCha20-Poly1305 encryption                     │
│  - Content-addressed storage                         │
│  - Distributed replication                           │
└──────────────────────────────────────────────────────┘
                         │
┌──────────────────────────────────────────────────────┐
│         WRAITH Protocol Stack                        │
│  (encrypted chunk transfer)                          │
└──────────────────────────────────────────────────────┘
```

---

## Components

### 1. Transcoding Pipeline

**Input:** Raw video/audio or RTMP stream

**Output:** Multi-bitrate HLS segments

**Profiles:**
```
240p:  426x240   @ 400 kbps
480p:  854x480   @ 1000 kbps
720p:  1280x720  @ 2500 kbps
1080p: 1920x1080 @ 5000 kbps
4K:    3840x2160 @ 15000 kbps
```

---

### 2. Segment Encryption

**Per-Segment Encryption:**
```rust
pub fn encrypt_segment(
    segment: &[u8],
    stream_key: &[u8; 32],
    segment_id: &str,
) -> Vec<u8> {
    let nonce = derive_nonce(stream_key, segment_id);
    let cipher = XChaCha20Poly1305::new(stream_key.into());

    cipher.encrypt(&nonce.into(), segment).unwrap()
}
```

---

### 3. Adaptive Bitrate Logic

**Bandwidth Estimation:** Measure download speed over last 10 segments

**Quality Selection:**
```
Bandwidth > 10 Mbps → 4K
5-10 Mbps → 1080p
2-5 Mbps → 720p
1-2 Mbps → 480p
<1 Mbps → 240p
```

---

## Performance Characteristics

**Latency:**
- VOD startup: <3 seconds
- Live: <5 seconds
- Quality switch: <500ms

**Throughput:**
- 4K: 15 Mbps
- 1080p: 5 Mbps
- 720p: 2.5 Mbps

---

## See Also

- [Features](features.md)
- [Implementation](implementation.md)
- [Client Overview](../overview.md)
