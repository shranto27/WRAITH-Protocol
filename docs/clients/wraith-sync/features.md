# WRAITH-Sync Features

**Document Version:** 1.0.0
**Last Updated:** 2025-11-28
**Client Version:** 1.0.0

---

## Overview

WRAITH-Sync provides serverless file synchronization across all your devices with end-to-end encryption, automatic conflict resolution, and version history.

---

## Core Features

### 1. Real-Time File Sync

**Description:** Automatic file synchronization as soon as changes are detected.

**User Stories:**
- As a user, files sync within 1 second of saving
- As a user, I can work offline and changes sync when online
- As a user, large files sync in background without blocking other operations

**Change Detection:**
- File creation/modification/deletion
- Folder structure changes
- File renames and moves
- Permission changes

---

### 2. Selective Sync

**Description:** Choose which folders to sync on each device.

**User Stories:**
- As a user, I can exclude large folders from mobile devices
- As a user, I can sync only work files to my work computer
- As a user, I can configure different sync sets per device

**Configuration:**
```toml
[[sync_folders]]
local_path = "~/Documents"
remote_path = "/Documents"
enabled = true

[[sync_folders.exclude]]
patterns = ["*.tmp", "node_modules/", ".git/"]
```

---

### 3. Conflict Resolution

**Description:** Automatic and manual conflict resolution when same file modified on multiple devices.

**Resolution Strategies:**
1. **Last-Writer-Wins (default):** Most recent modification wins
2. **Keep Both:** Save both versions with different names
3. **Manual:** Prompt user to choose version

**Conflict Notification:**
```
┌─────────────────────────────────────────┐
│  Sync Conflict Detected                 │
├─────────────────────────────────────────┤
│  File: report.docx                      │
│                                         │
│  Local:  Modified 2:30 PM (1.2 MB)      │
│  Remote: Modified 2:32 PM (1.3 MB)      │
│          from Desktop                   │
│                                         │
│  [Use Local] [Use Remote] [Keep Both]  │
└─────────────────────────────────────────┘
```

---

### 4. Version History

**Description:** 30-day version history with point-in-time restore.

**Features:**
- View all versions of a file
- Restore previous version
- Compare versions (diff for text files)
- Automatic cleanup after 30 days

**Version Viewer:**
```
document.docx - Version History
┌─────────────────────────────────────────┐
│  Today at 3:45 PM    (Current)  1.5 MB  │
│  Today at 2:30 PM              1.4 MB  │
│  Today at 11:20 AM             1.3 MB  │
│  Yesterday at 4:15 PM          1.2 MB  │
│  Nov 27 at 9:30 AM             1.0 MB  │
│                                         │
│  [Restore] [Preview] [Delete]           │
└─────────────────────────────────────────┘
```

---

### 5. Bandwidth Throttling

**Description:** Limit sync speeds to avoid network congestion.

**User Stories:**
- As a user, I can set maximum upload/download speeds
- As a user, I can pause sync temporarily
- As a user, I can schedule different limits for different times

**Throttle Settings:**
```toml
[bandwidth]
max_upload = "5 MB/s"
max_download = "10 MB/s"
enable_schedule = true

[[bandwidth.schedule]]
time_range = "09:00-17:00"
max_upload = "1 MB/s"
```

---

## Advanced Features

### 1. Delta Sync

**Description:** Transfer only changed blocks instead of entire files.

**Benefits:**
- 90%+ bandwidth savings for modified files
- Faster sync for large files
- Lower data usage on mobile

**Example:**
```
Original file: 100 MB
Modified: 500 KB changed
Transferred: 500 KB (99.5% savings)
```

---

### 2. Deduplication

**Description:** Store identical blocks only once across all files.

**User Stories:**
- As a user, duplicate files use minimal additional space
- As a user, different versions of same file share common blocks

**Deduplication Rate:**
- Documents: 60-80% savings
- Photos: 10-20% savings (already compressed)
- Videos: 5-10% savings

---

### 3. Compression

**Description:** Automatic compression with zstandard algorithm.

**Compression Ratios:**
- Text files: 3-5x
- Documents: 2-3x
- Images: 1.1-1.2x (already compressed)

**Configuration:**
```toml
[compression]
enabled = true
level = 3  # 1-19 (higher = better compression, slower)
min_file_size = 1048576  # 1 MB
```

---

### 4. Encryption

**Description:** All synced files encrypted end-to-end.

**Encryption:**
- Algorithm: XChaCha20-Poly1305
- Key derivation: BLAKE3
- Per-device encryption keys
- Zero-knowledge architecture (server can't decrypt)

---

## Platform-Specific Features

### Desktop (Windows/macOS/Linux)

**Features:**
- System tray icon with sync status
- Context menu integration ("View on other devices")
- Selective sync folder configuration
- Unlimited folder sync

### Mobile (iOS/Android)

**Features:**
- Camera upload (photos/videos)
- Download on demand (save storage)
- Selective folder sync
- Background sync with battery optimization

---

## User Interface

### Main Window

```
┌─────────────────────────────────────────┐
│  WRAITH Sync                    ─ □ ×   │
├─────────────────────────────────────────┤
│  Folders  │  Devices  │  Settings       │
├─────────────────────────────────────────┤
│                                         │
│  Synced Folders                         │
│  ┌───────────────────────────────────┐  │
│  │ 📁 Documents          ✓ Synced   │  │
│  │    ~/Documents                    │  │
│  │    1,234 files • 15.2 GB          │  │
│  ├───────────────────────────────────┤  │
│  │ 📁 Photos             ⏸ Paused   │  │
│  │    ~/Pictures                     │  │
│  │    5,678 files • 125 GB           │  │
│  └───────────────────────────────────┘  │
│                                         │
│  [+ Add Folder]                         │
│                                         │
│  Sync Status: Idle                      │
│  Last sync: 2 minutes ago               │
│                                         │
└─────────────────────────────────────────┘
```

### Sync Status Indicator

```
System Tray:
  🟢 Synced (all up to date)
  🔵 Syncing... (in progress)
  ⏸  Paused
  🔴 Error (click for details)
  ⚠️  Conflict (needs resolution)
```

---

## Configuration Options

### Sync Settings

```toml
[sync]
# Enable sync on startup
auto_start = true

# Check for changes interval (seconds)
watch_interval = 1

# Conflict resolution strategy
conflict_resolution = "last_writer_wins"  # last_writer_wins, keep_both, manual

# Enable version history
version_history = true
version_retention_days = 30
```

### Network Settings

```toml
[network]
# Bandwidth limits
max_upload_speed = "unlimited"  # or "5 MB/s"
max_download_speed = "unlimited"

# Timeout settings
connection_timeout = 30  # seconds
transfer_timeout = 300  # seconds

# Retry settings
max_retries = 3
retry_delay = 5  # seconds
```

### Storage Settings

```toml
[storage]
# Cache settings
cache_size = 1073741824  # 1 GB
cache_expiry = 86400  # 24 hours

# Metadata
metadata_path = "~/.local/share/wraith-sync"

# Version storage
version_storage_path = "~/.local/share/wraith-sync/versions"
max_version_storage = 10737418240  # 10 GB
```

---

## Command-Line Interface

```bash
# Initialize sync folder
wraith-sync init ~/Documents --group <group-secret>

# Start sync daemon
wraith-sync daemon

# Check sync status
wraith-sync status

# List synced folders
wraith-sync folders list

# Add folder
wraith-sync folders add ~/Projects

# Remove folder
wraith-sync folders remove ~/Projects

# Pause/resume sync
wraith-sync pause
wraith-sync resume

# View conflicts
wraith-sync conflicts list

# Resolve conflict
wraith-sync conflicts resolve <file> --strategy local
```

---

## See Also

- [Architecture](architecture.md)
- [Implementation](implementation.md)
- [Client Overview](../overview.md)
