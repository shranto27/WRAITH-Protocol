# WRAITH-Vault Features

**Document Version:** 1.0.0
**Last Updated:** 2025-11-28
**Client Version:** 1.0.0

---

## Overview

WRAITH-Vault provides military-grade encrypted backups with geographic redundancy and deduplication, all without monthly cloud storage fees.

---

## Core Features

### 1. Automatic Incremental Backups

**User Stories:**
- As a user, backups run automatically on schedule
- As a user, only changed files are backed up after initial backup
- As a user, I can set backup frequency (hourly/daily/weekly)

**Backup Schedules:**
- Real-time (on file change)
- Hourly
- Daily (at specific time)
- Weekly (specific day/time)
- Manual only

---

### 2. Deduplication

**User Stories:**
- As a user, duplicate files use minimal extra space
- As a user, multiple versions of files share common data
- As a user, I can see deduplication savings

**Deduplication Levels:**
- File-level: Identical files stored once
- Block-level: Shared blocks between files
- Across backups: All backups share chunk pool

**Typical Savings:**
- Documents: 60-80%
- Photos: 10-20%
- Videos: 5-10%
- Overall: 50-70%

---

### 3. Version History

**User Stories:**
- As a user, I can restore previous versions of files
- As a user, I can browse daily snapshots for 30 days
- As a user, I can compare versions

**Retention Policy:**
- Hourly snapshots: Keep 24 hours
- Daily snapshots: Keep 30 days
- Weekly snapshots: Keep 12 weeks
- Monthly snapshots: Keep 6 months

---

### 4. Point-in-Time Restore

**User Stories:**
- As a user, I can restore entire backup to specific date
- As a user, I can restore individual files
- As a user, I can restore to different location

**Restore Options:**
- Full restore (entire backup)
- Selective restore (specific files/folders)
- Restore to original location
- Restore to custom location

---

### 5. Data Durability

**User Stories:**
- As a user, my data survives peer failures
- As a user, I can verify backup integrity
- As a user, I receive alerts if redundancy degrades

**Durability Guarantees:**
- 99.999% durability (5 nines)
- Survives loss of 4 out of 20 peers
- Automatic re-replication if peers go offline
- Monthly integrity verification

---

## Advanced Features

### Bandwidth Throttling

**User Stories:**
- As a user, I can limit backup bandwidth
- As a user, backups don't slow down my internet
- As a user, I can schedule different limits for different times

### Encryption

**Encryption:**
- Client-side encryption (zero-knowledge)
- XChaCha20-Poly1305 AEAD
- Key derivation from passphrase (Argon2)
- Per-chunk encryption

**Key Management:**
- Master key derived from passphrase
- Chunk keys derived from master key
- Keys never leave local device

### Compression

**Compression:**
- Zstandard (level 3)
- Automatic for all chunks
- 2-3x compression for text
- Minimal overhead for media

---

## User Interface

### Main Dashboard

```
┌─────────────────────────────────────────┐
│  WRAITH Vault                   ─ □ ×   │
├─────────────────────────────────────────┤
│  Backups  │  Snapshots  │  Settings     │
├─────────────────────────────────────────┤
│                                         │
│  My Documents                           │
│  ┌───────────────────────────────────┐  │
│  │ Last backup: 10 minutes ago       │  │
│  │ Size: 50.2 GB (32.1 GB dedupe)    │  │
│  │ Files: 12,345                     │  │
│  │ Status: ✓ All chunks replicated   │  │
│  │                                   │  │
│  │ Next backup: in 50 minutes        │  │
│  │ Schedule: Hourly                  │  │
│  │                                   │  │
│  │ [Backup Now] [Restore] [Settings] │  │
│  └───────────────────────────────────┘  │
│                                         │
│  [+ Add Backup]                         │
│                                         │
└─────────────────────────────────────────┘
```

### Restore Interface

```
┌─────────────────────────────────────────┐
│  Restore Backup: My Documents           │
├─────────────────────────────────────────┤
│  Select Snapshot:                       │
│  ○ Latest (10 minutes ago)              │
│  ○ Today 9:00 AM                        │
│  ○ Yesterday 9:00 AM                    │
│  ○ Nov 27 9:00 AM                       │
│  ○ Custom date...                       │
│                                         │
│  Restore Options:                       │
│  ☑ Overwrite existing files             │
│  ☐ Restore to: ~/Documents/restored/    │
│  ☑ Preserve metadata (timestamps, etc.) │
│                                         │
│  Files to restore: 12,345 (50.2 GB)     │
│  Estimated time: 2 hours 15 minutes     │
│                                         │
│  [Cancel]  [Start Restore]              │
└─────────────────────────────────────────┘
```

---

## Platform Support

### Desktop

**Platforms:**
- Windows 10+
- macOS 11+
- Linux (Ubuntu/Fedora/Arch)

**Features:**
- System tray integration
- Scheduled backups
- Unlimited backup size

### NAS

**Platforms:**
- Synology DSM 7.0+
- QNAP QTS 5.0+
- TrueNAS Core/Scale

**Features:**
- Web UI
- Headless operation
- SSH access for advanced config

---

## See Also

- [Architecture](architecture.md)
- [Implementation](implementation.md)
- [Client Overview](../overview.md)
