# WRAITH-Sync Architecture

**Document Version:** 1.0.0
**Last Updated:** 2025-11-28
**Client Version:** 1.0.0

---

## Overview

WRAITH-Sync provides Dropbox-like file synchronization without central servers. Files are synced peer-to-peer across all user devices using the WRAITH protocol with end-to-end encryption and conflict resolution.

**Design Goals:**
- Real-time file synchronization across unlimited devices
- Delta sync (only changed blocks transmitted)
- Automatic conflict resolution with version history
- Selective sync (choose folders per device)
- Offline-first operation with sync queue

---

## Architecture Diagram

```
┌──────────────────────────────────────────────────────┐
│               User Interface Layer                    │
│  ┌────────────────┐  ┌──────────────────────────┐   │
│  │  Desktop GUI   │  │     Mobile App            │   │
│  │  (Tauri)       │  │   (React Native)          │   │
│  └────────────────┘  └──────────────────────────┘   │
└──────────────────────────────────────────────────────┘
                         │
┌──────────────────────────────────────────────────────┐
│            Sync Engine Layer                         │
│  ┌──────────────────────────────────────────────┐   │
│  │  Sync Coordinator                            │   │
│  │  - Detect file changes                       │   │
│  │  - Compare remote/local state                │   │
│  │  - Resolve conflicts                         │   │
│  └──────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────┐   │
│  │  Delta Sync Engine                           │   │
│  │  - rsync-style diff algorithm                │   │
│  │  - Block-level deduplication                 │   │
│  │  - Compression (zstd)                        │   │
│  └──────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────┐   │
│  │  Version Manager                             │   │
│  │  - Track file versions                       │   │
│  │  - 30-day history                            │   │
│  │  - Point-in-time restore                     │   │
│  └──────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────┘
                         │
┌──────────────────────────────────────────────────────┐
│           File System Watcher                        │
│  (chokidar on desktop, native APIs on mobile)        │
└──────────────────────────────────────────────────────┘
                         │
┌──────────────────────────────────────────────────────┐
│         Sync Metadata Database (SQLite)              │
│  - File hashes, modification times                   │
│  - Device sync state                                 │
│  - Version history                                   │
└──────────────────────────────────────────────────────┘
                         │
┌──────────────────────────────────────────────────────┐
│         WRAITH Protocol Stack                        │
│  (file transfer with encryption)                     │
└──────────────────────────────────────────────────────┘
```

---

## Components

### 1. File System Watcher

**Purpose:** Detect file and folder changes in real-time.

**Events:**
- File created/modified/deleted
- Folder created/deleted
- File moved/renamed

**Implementation:**
```typescript
import chokidar from 'chokidar';

export class FileWatcher extends EventEmitter {
  private watcher: chokidar.FSWatcher | null = null;

  addPath(path: string): void {
    if (!this.watcher) {
      this.watcher = chokidar.watch(path, {
        persistent: true,
        ignoreInitial: false,
        awaitWriteFinish: {
          stabilityThreshold: 2000,
          pollInterval: 100,
        },
        ignored: [
          /(^|[\/\\])\../,  // Hidden files
          '**/.wraith-sync/**',  // Metadata
          '**/node_modules/**',
        ],
      });

      this.watcher
        .on('add', (path, stats) => this.emit('change', { type: 'add', path, stats }))
        .on('change', (path, stats) => this.emit('change', { type: 'change', path, stats }))
        .on('unlink', path => this.emit('change', { type: 'unlink', path }));
    } else {
      this.watcher.add(path);
    }
  }
}
```

---

### 2. Sync Coordinator

**Responsibilities:**
- Maintain sync state for each folder
- Detect conflicts (same file modified on multiple devices)
- Coordinate sync operations across devices
- Manage sync queue for offline changes

**Sync Algorithm:**
```
For each file in watched folder:
  1. Calculate BLAKE3 hash
  2. Compare with local database:
     - If hash matches: No action
     - If hash differs: File modified locally
  3. Query peer devices for file state
  4. Determine sync action:
     - Upload: Local newer than all peers
     - Download: Peer has newer version
     - Conflict: Multiple devices modified simultaneously
  5. Execute sync operation
  6. Update local database
```

**Conflict Resolution:**
```
Last-Writer-Wins (default):
  - File with most recent modification time wins
  - Losing version saved as "file (conflict).ext"

Manual Resolution:
  - User prompted to choose version
  - Both versions kept until resolved
```

---

### 3. Delta Sync Engine

**Purpose:** Minimize bandwidth by transferring only changed blocks.

**rsync Algorithm:**
```
Sender                          Receiver
  │                                │
  │<─── Request file ──────────────│
  │  (file path, hash)             │
  │                                │
  │──── Send block signatures ────>│
  │  [{block: 0, hash: abc...},    │
  │   {block: 1, hash: def...}]    │
  │                                │
  │<─── Request changed blocks ────│
  │  [0, 5, 7, 9]                  │
  │                                │
  │──── Send changed blocks ──────>│
  │  [{block: 0, data: ...},       │
  │   {block: 5, data: ...}]       │
  │                                │
  │<─── File reconstructed ────────│
```

**Block Size:** 4 KB (configurable)

**Compression:** Zstandard (level 3) on each block before transmission

---

### 4. Version Manager

**Purpose:** Maintain file version history for 30 days.

**Storage:**
```
~/.local/share/wraith-sync/versions/
├── <folder_id>/
│   ├── <file_hash_v1>
│   ├── <file_hash_v2>
│   └── <file_hash_v3>
```

**Retention Policy:**
- Keep all versions for 30 days
- After 30 days, keep only daily snapshots
- After 90 days, delete all versions

**Version Metadata:**
```sql
CREATE TABLE file_versions (
    id INTEGER PRIMARY KEY,
    file_id INTEGER NOT NULL,
    version INTEGER NOT NULL,
    hash BLOB NOT NULL,
    size INTEGER NOT NULL,
    modified_at INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (file_id) REFERENCES file_metadata(id)
);
```

---

## Data Flow

### File Change Detection Flow

```
1. User modifies file (e.g., saves document.pdf)
2. FileWatcher detects change after 2-second stability
3. Sync Coordinator calculates BLAKE3 hash
4. Hash compared with database:
   - Different → File changed
5. File added to sync queue
6. Sync Coordinator contacts peer devices
7. Determines action (upload/download/conflict)
8. Delta sync calculates changed blocks
9. Changed blocks transferred via WRAITH
10. Peer receives blocks and reconstructs file
11. Database updated on both sides
```

---

## Database Schema

```sql
CREATE TABLE sync_folders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    local_path TEXT UNIQUE NOT NULL,
    remote_path TEXT NOT NULL,
    enabled INTEGER DEFAULT 1,
    created_at INTEGER NOT NULL
);

CREATE TABLE file_metadata (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    folder_id INTEGER NOT NULL,
    relative_path TEXT NOT NULL,
    size INTEGER NOT NULL,
    modified_at INTEGER NOT NULL,
    hash BLOB NOT NULL,
    is_directory INTEGER DEFAULT 0,
    synced INTEGER DEFAULT 0,
    deleted INTEGER DEFAULT 0,
    FOREIGN KEY (folder_id) REFERENCES sync_folders(id) ON DELETE CASCADE,
    UNIQUE (folder_id, relative_path)
);

CREATE TABLE devices (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    device_id TEXT UNIQUE NOT NULL,
    device_name TEXT NOT NULL,
    last_seen INTEGER NOT NULL,
    public_key BLOB NOT NULL
);

CREATE TABLE conflicts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_id INTEGER NOT NULL,
    local_hash BLOB NOT NULL,
    remote_hash BLOB NOT NULL,
    local_modified_at INTEGER NOT NULL,
    remote_modified_at INTEGER NOT NULL,
    device_id TEXT NOT NULL,
    resolved INTEGER DEFAULT 0,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (file_id) REFERENCES file_metadata(id)
);

CREATE INDEX idx_file_metadata_folder ON file_metadata(folder_id);
CREATE INDEX idx_file_metadata_path ON file_metadata(relative_path);
```

---

## Protocol Integration

### Sync Messages

**File State Request:**
```json
{
  "type": "file_state_request",
  "folder_id": "abc123",
  "paths": ["file1.txt", "file2.jpg"]
}
```

**File State Response:**
```json
{
  "type": "file_state_response",
  "states": [
    {
      "path": "file1.txt",
      "hash": "blake3_hash",
      "size": 1024,
      "modified_at": 1700000000
    }
  ]
}
```

**Sync Request:**
```json
{
  "type": "sync_request",
  "file_path": "file1.txt",
  "blocks_needed": [0, 5, 7]
}
```

---

## Performance Characteristics

**Sync Performance:**
- Initial sync: 100 GB in <2 hours (1 Gbps network)
- Incremental sync: <1 second for typical file change
- Delta sync: 90%+ bandwidth reduction for modified files

**Memory Usage:**
- Baseline: 50 MB
- + 1 KB per tracked file
- + 10 MB per active sync operation

**Disk Usage:**
- Metadata database: ~100 bytes per file
- Version history: Depends on file change frequency

---

## See Also

- [Features](features.md)
- [Implementation](implementation.md)
- [Client Overview](../overview.md)
