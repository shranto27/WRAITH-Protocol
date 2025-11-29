# WRAITH-Transfer Features

**Document Version:** 1.0.0
**Last Updated:** 2025-11-28
**Client Version:** 1.0.0

---

## Overview

WRAITH-Transfer provides an intuitive interface for secure, high-speed file transfers between peers. This document details all features available in both CLI and GUI interfaces.

---

## Core Features

### 1. Drag-and-Drop File Transfer

**Description:** Simply drag files or folders onto the application window to initiate transfer.

**User Stories:**
- As a user, I can drag a file from my desktop onto WRAITH-Transfer to send it
- As a user, I can drag multiple files simultaneously for batch transfer
- As a user, I can see a visual indicator when hovering files over the drop zone

**Implementation:**
- Native file drag-and-drop API integration
- Folder recursion with progress tracking
- File type filtering (optional)

**Platform Support:**
- Windows: Full support via Windows Shell API
- macOS: Full support via Cocoa drag-and-drop
- Linux: Full support via GTK/Qt drag-and-drop

---

### 2. QR Code Pairing

**Description:** Establish peer connection by scanning QR code or displaying code for scanning.

**User Stories:**
- As a sender, I can generate a QR code containing my connection details
- As a receiver, I can scan a QR code with my device camera to connect
- As a user, I can verify peer identity via safety numbers after pairing

**QR Code Contents:**
```json
{
  "version": 1,
  "peer_id": "wraith_abc123...",
  "public_key": "base64_encoded_ed25519_key",
  "endpoints": [
    "192.168.1.100:41641",
    "relay.wraith.network:443"
  ],
  "fingerprint": "SHA256:abc123..."
}
```

**Security:**
- QR codes expire after 5 minutes
- One-time use codes prevent replay attacks
- Visual confirmation of peer name/fingerprint before transfer

---

### 3. Multi-File Batch Transfer

**Description:** Send multiple files or entire folders in a single transfer session.

**User Stories:**
- As a user, I can select 10,000+ files for batch transfer
- As a user, I see individual progress for each file
- As a user, I can pause/resume the entire batch

**Features:**
- Parallel file transfers (configurable, default 4 concurrent)
- Per-file integrity verification (BLAKE3)
- Automatic retry for failed files
- Summary report on completion

**Performance:**
- Small files (<1 MB): Bundled into archives for efficiency
- Large files (>100 MB): Streamed with chunking
- Mixed sizes: Adaptive strategy based on file distribution

---

### 4. Resume Support

**Description:** Automatically resume interrupted transfers from the last checkpoint.

**User Stories:**
- As a user, I can close the application and resume transfers later
- As a user, network interruptions don't restart transfers from zero
- As a user, I can manually pause and resume transfers

**Implementation:**
- State persistence in `~/.cache/wraith-transfer/state/`
- Chunk-level tracking (1 MB chunks)
- Automatic reconnection with exponential backoff
- Multi-peer resume (continue from different peer if original offline)

**Resume Scenarios:**
- Application crash → Auto-resume on restart
- Network disconnect → Auto-resume when network available
- Manual pause → Resume on user action
- Peer disconnect → Find alternative peer from DHT

---

### 5. Progress Tracking

**Description:** Real-time visualization of transfer progress and statistics.

**Metrics Displayed:**
- Bytes transferred / Total size
- Current throughput (MB/s)
- Average throughput
- Estimated time remaining (ETA)
- Peer connection status
- Number of chunks completed / total

**Update Frequency:**
- GUI: 100ms refresh rate
- CLI: 500ms refresh rate
- Logs: 1 second interval

**Visual Indicators:**
- Progress bar (percentage)
- Throughput graph (real-time chart)
- Chunk completion grid
- Peer latency heatmap

---

### 6. Integrity Verification

**Description:** Automatic verification of file integrity using BLAKE3 hashing.

**Verification Levels:**
1. **Chunk-level:** Each 1 MB chunk verified on receipt
2. **File-level:** Entire file hash verified on completion
3. **Transfer-level:** All files verified in batch transfers

**Hash Algorithm:**
- BLAKE3 (faster than SHA256, cryptographically secure)
- Merkle tree construction for efficient partial verification
- Hash stored in transfer metadata

**Verification Flow:**
```
Sender                          Receiver
  │                                │
  │── CHUNK[0] + HASH[0] ────────> │ ✓ Verify HASH[0]
  │── CHUNK[1] + HASH[1] ────────> │ ✓ Verify HASH[1]
  │                                │
  │<─ REQUEST_RESEND[0] ─────────── │ ✗ HASH[0] mismatch
  │── CHUNK[0] + HASH[0] ────────> │ ✓ Verify HASH[0]
  │                                │
  │<─ TRANSFER_COMPLETE ─────────── │ ✓ All chunks verified
```

---

### 7. Compression (Optional)

**Description:** Optional LZ4 compression for faster transfers over slow networks.

**User Stories:**
- As a user, I can enable compression for text-heavy files
- As a user, I can see compression ratio in transfer statistics
- As a user, compression is disabled for already-compressed formats (zip, jpg, mp4)

**Compression Algorithm:**
- LZ4 (high-speed compression/decompression)
- Compression ratio: 2-4x for text, 1.1-1.5x for mixed content
- Automatic bypass for incompressible data

**Configuration:**
```toml
[transfer.compression]
enabled = true
level = "fast"  # fast, balanced, high
min_file_size = 1048576  # 1 MB
exclude_extensions = ["zip", "gz", "jpg", "png", "mp4", "mkv"]
```

---

### 8. System Tray Integration

**Description:** Minimal UI in system tray for background operation.

**Features:**
- Tray icon with notification badge (active transfers)
- Quick actions menu (Send, Receive, Settings, Quit)
- Toast notifications for transfer events
- Start minimized to tray option

**Notifications:**
- Transfer initiated
- Transfer completed
- Transfer failed
- Peer connected/disconnected
- Application updates available

**Platform-Specific:**
- Windows: System tray with context menu
- macOS: Menu bar with status item
- Linux: Appindicator or StatusNotifier

---

### 9. Transfer History

**Description:** Searchable log of all past transfers.

**Data Stored:**
- File name and size
- Peer ID and display name
- Transfer start/end time
- Transfer status (completed, failed, cancelled)
- Throughput statistics
- File hash (for re-verification)

**Features:**
- Full-text search
- Filter by date range, peer, status
- Export to CSV
- Clear history (older than N days)

**Storage:**
- SQLite database: `~/.local/share/wraith-transfer/history.db`
- Automatic cleanup: Keep last 1000 transfers or 90 days
- Privacy mode: Disable history logging

---

### 10. Dark/Light Theme

**Description:** Automatic or manual theme selection.

**Modes:**
- Light theme (default)
- Dark theme
- System theme (follow OS preference)

**Customization:**
- Accent color selection
- Font size adjustment
- Compact/comfortable display density

**Accessibility:**
- High contrast mode
- Respect OS accessibility settings
- Keyboard navigation support

---

## Advanced Features

### Multi-Source Downloads

**Description:** Download file chunks from multiple peers simultaneously.

**Benefits:**
- 2-10x faster transfers for popular files
- Redundancy if one peer disconnects
- Load balancing across peers

**Algorithm:**
- Discover all peers with file from DHT
- Request different chunks from each peer
- Merge chunks on completion
- Verify assembled file hash

**Configuration:**
```toml
[transfer.multi_source]
enabled = true
max_peers = 10
chunk_size = 1048576  # 1 MB
prefer_fast_peers = true
```

---

### Bandwidth Throttling

**Description:** Limit upload/download speeds to avoid network congestion.

**User Stories:**
- As a user, I can set maximum upload/download speeds
- As a user, I can schedule different limits for different times
- As a user, I can pause/resume to instantly free bandwidth

**Configuration:**
```toml
[transfer.bandwidth]
max_upload = "10 MB/s"
max_download = "50 MB/s"
enable_schedule = true

[[transfer.bandwidth.schedule]]
time_range = "09:00-17:00"
max_upload = "1 MB/s"  # Limit during work hours
max_download = "10 MB/s"
```

---

### Password Protection

**Description:** Require password to access received files.

**User Stories:**
- As a sender, I can set a password for sensitive transfers
- As a receiver, I must enter the password before accessing files
- As a user, passwords are never sent over the network

**Implementation:**
- Password-based key derivation (Argon2)
- Additional encryption layer on top of Noise_XX
- Password hint (optional, not transmitted)
- Brute-force protection (rate limiting)

---

### Link Sharing

**Description:** Generate shareable links for file transfers.

**User Stories:**
- As a sender, I can generate a `wraith://` link for my file
- As a receiver, I can click the link to start download
- As a sender, I can set link expiration time

**Link Format:**
```
wraith://transfer/<file_hash>?peer=<peer_id>&relay=<relay_url>&expires=<timestamp>
```

**Security:**
- Links contain file hash (content addressing)
- Optional encryption key in URL fragment (not sent to relay)
- Expiration enforced by sender
- One-time use option

---

## Platform-Specific Features

### Windows

- Windows Explorer context menu integration ("Send with WRAITH")
- Windows Defender SmartScreen approval
- UWP notifications
- Windows Installer (MSI) with auto-update

### macOS

- Finder integration
- Touch Bar support (MacBook Pro)
- Notification Center integration
- Apple Notarization
- Sparkle auto-update

### Linux

- Desktop file integration (GNOME/KDE)
- AppImage with auto-update
- deb/rpm packages
- SystemD integration for daemon mode

---

## Configuration Options

### General Settings

```toml
[general]
auto_start = true
minimize_to_tray = true
check_updates = true
update_channel = "stable"  # stable, beta, nightly
default_download_dir = "~/Downloads"
```

### Network Settings

```toml
[network]
listen_port = 41641
enable_upnp = true
enable_nat_pmp = true
enable_relay = true
relay_servers = [
  "relay1.wraith.network:443",
  "relay2.wraith.network:443"
]
stun_servers = [
  "stun.l.google.com:19302"
]
```

### Security Settings

```toml
[security]
verify_peer_identity = true
require_password = false
allow_unsigned_transfers = false
auto_verify_hashes = true
```

### Privacy Settings

```toml
[privacy]
enable_history = true
enable_analytics = false
obfuscation_level = "high"  # none, low, medium, high
cover_traffic = true
```

---

## User Interface

### Main Window Layout

```
┌─────────────────────────────────────────────────┐
│  WRAITH Transfer                        ─ □ ×   │
├─────────────────────────────────────────────────┤
│  Send  │  Receive  │  History  │  Settings      │
├─────────────────────────────────────────────────┤
│                                                 │
│   ┌─────────────────────────────────────────┐  │
│   │  Drop files here to send                │  │
│   │  or click to browse                     │  │
│   │                                         │  │
│   │            📁                           │  │
│   │                                         │  │
│   └─────────────────────────────────────────┘  │
│                                                 │
│   Connected Peers: 3                            │
│   ┌─────────────────────────────────────────┐  │
│   │  Alice    │  192.168.1.100  │  Active   │  │
│   │  Bob      │  Relay          │  Idle     │  │
│   │  Charlie  │  192.168.1.102  │  Active   │  │
│   └─────────────────────────────────────────┘  │
│                                                 │
└─────────────────────────────────────────────────┘
```

### Transfer Progress Window

```
┌─────────────────────────────────────────────────┐
│  Transferring: project.zip                      │
├─────────────────────────────────────────────────┤
│  ████████████████████░░░░░░  65% (650 MB / 1 GB)│
│                                                 │
│  Speed: 85 MB/s                                 │
│  Time Remaining: 4 seconds                      │
│  Peer: Alice (192.168.1.100)                    │
│                                                 │
│  [Pause]  [Cancel]                              │
└─────────────────────────────────────────────────┘
```

---

## Command-Line Interface

### Send File

```bash
wraith-transfer send document.pdf --peer 192.0.2.10:41641
wraith-transfer send folder/ --peer alice --compress
wraith-transfer send *.jpg --peer bob --password
```

### Receive File

```bash
wraith-transfer receive --output ~/Downloads/
wraith-transfer receive --peer alice --auto-accept
```

### List Peers

```bash
wraith-transfer peers list
wraith-transfer peers add alice 192.0.2.10:41641
wraith-transfer peers verify alice
```

### Transfer History

```bash
wraith-transfer history
wraith-transfer history --last 10
wraith-transfer history --search "document.pdf"
```

---

## See Also

- [Architecture](architecture.md)
- [Implementation](implementation.md)
- [Client Overview](../overview.md)
