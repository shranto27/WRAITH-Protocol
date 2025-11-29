# WRAITH-Stream Features

**Document Version:** 1.0.0
**Last Updated:** 2025-11-28
**Client Version:** 1.0.0

---

## Overview

WRAITH-Stream provides encrypted peer-to-peer media streaming for both live broadcasts and video-on-demand content.

---

## Core Features

### 1. Live Streaming

**User Stories:**
- As a broadcaster, I can stream live video to peers
- As a viewer, I can watch streams with low latency
- As a broadcaster, I can see viewer count in real-time

**Supported Input:**
- RTMP from OBS Studio
- Webcam/screen capture
- Pre-recorded files

**Latency:** <5 seconds glass-to-glass

---

### 2. Video-on-Demand (VOD)

**User Stories:**
- As a user, I can upload video files for streaming
- As a viewer, I can seek to any position instantly
- As a user, I can download videos for offline viewing

**Supported Formats:**
- Input: MP4, MOV, MKV, AVI, WebM
- Output: HLS (.m3u8 + .ts segments)

---

### 3. Adaptive Bitrate Streaming

**User Stories:**
- As a viewer, video quality adjusts to my bandwidth
- As a viewer, I can manually select quality
- As a viewer, quality switches seamlessly without buffering

**Quality Levels:**
- Auto (recommended)
- 240p, 480p, 720p, 1080p, 4K

---

### 4. Subtitle Support

**User Stories:**
- As a content creator, I can add subtitles to videos
- As a viewer, I can enable/disable subtitles
- As a viewer, I can choose subtitle language

**Supported Formats:**
- SRT (SubRip)
- VTT (WebVTT)
- ASS/SSA

---

## Advanced Features

### Multi-Camera Streaming

**User Stories:**
- As a broadcaster, I can stream from multiple camera angles
- As a viewer, I can switch between camera angles

### DVR Functionality

**User Stories:**
- As a viewer of live streams, I can pause and rewind
- As a viewer, I can resume from where I left off

### Thumbnails and Preview

**User Stories:**
- As a viewer, I see thumbnail when hovering over seek bar
- As a viewer, I see preview images in video list

---

## User Interface

### Player

```
┌─────────────────────────────────────────┐
│                                         │
│         [Video Player Area]             │
│                                         │
├─────────────────────────────────────────┤
│  ▶️ [=========>        ] 12:34 / 45:67 │
│  🔊 ━━━━━━━━━━━━━━━━━━━━━ 🔴 LIVE     │
│  ⚙️ Quality: Auto (1080p) | CC: Off    │
└─────────────────────────────────────────┘
```

---

## See Also

- [Architecture](architecture.md)
- [Implementation](implementation.md)
- [Client Overview](../overview.md)
