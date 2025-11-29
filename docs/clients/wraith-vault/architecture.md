# WRAITH-Vault Architecture

**Document Version:** 1.0.0
**Last Updated:** 2025-11-28
**Client Version:** 1.0.0

---

## Overview

WRAITH-Vault provides encrypted, decentralized backup storage with geographic redundancy using erasure coding and distributed replication across the WRAITH network.

**Design Goals:**
- Military-grade encrypted backups (99.999% durability)
- No monthly cloud storage fees
- Geographic redundancy across peer network
- Automatic incremental backups
- Deduplication reduces storage by 60%+
- Point-in-time restore capabilities

---

## Architecture Diagram

```
┌──────────────────────────────────────────────────────┐
│            Backup UI                                 │
│  ┌────────────────┐  ┌──────────────────────────┐   │
│  │  Desktop GUI   │  │     NAS Web UI           │   │
│  │  (Tauri)       │  │   (React)                │   │
│  └────────────────┘  └──────────────────────────┘   │
└──────────────────────────────────────────────────────┘
                         │
┌──────────────────────────────────────────────────────┐
│         Backup Engine                                │
│  ┌──────────────────────────────────────────────┐   │
│  │  Chunker (Content-Defined Chunking)          │   │
│  │  - Variable-size chunks (256KB-8MB)          │   │
│  │  - BLAKE3 deduplication                      │   │
│  └──────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────┐   │
│  │  Compressor (Zstandard)                      │   │
│  │  - Level 3 compression                       │   │
│  │  - 2-3x reduction for documents              │   │
│  └──────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────┐   │
│  │  Erasure Coder (Reed-Solomon)                │   │
│  │  - 16 data + 4 parity blocks                 │   │
│  │  - Survives 4 peer failures                  │   │
│  └──────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────┘
                         │
┌──────────────────────────────────────────────────────┐
│         Storage Layer                                │
│  - Chunk deduplication index                         │
│  - Backup catalog (SQLite)                           │
│  - Version snapshots (daily for 30 days)             │
└──────────────────────────────────────────────────────┘
                         │
┌──────────────────────────────────────────────────────┐
│         WRAITH Protocol Stack                        │
│  (encrypted chunk distribution + DHT)                │
└──────────────────────────────────────────────────────┘
```

---

## Components

### 1. Content-Defined Chunker

**Purpose:** Split files into variable-size chunks for efficient deduplication.

**Algorithm:**
```rust
const MIN_CHUNK_SIZE: usize = 256 * 1024;  // 256 KB
const AVG_CHUNK_SIZE: usize = 1024 * 1024; // 1 MB
const MAX_CHUNK_SIZE: usize = 8 * 1024 * 1024; // 8 MB

pub struct Chunker {
    rolling_hash: u32,
}

impl Chunker {
    pub fn chunk_file<R: Read>(&mut self, reader: &mut R) -> Vec<Chunk> {
        let mut chunks = Vec::new();
        let mut buffer = vec![0u8; MAX_CHUNK_SIZE];
        let mut chunk_start = 0;

        loop {
            let n = reader.read(&mut buffer[chunk_start..]).unwrap();
            if n == 0 { break; }

            for i in chunk_start..chunk_start + n {
                self.update_rolling_hash(buffer[i]);

                let should_split = (self.rolling_hash % AVG_CHUNK_SIZE as u32) == 0
                    && (i - chunk_start) >= MIN_CHUNK_SIZE;

                if should_split || (i - chunk_start) >= MAX_CHUNK_SIZE {
                    let chunk_data = &buffer[chunk_start..i];
                    let chunk_hash = blake3::hash(chunk_data);

                    chunks.push(Chunk {
                        hash: chunk_hash.into(),
                        size: chunk_data.len(),
                        data: chunk_data.to_vec(),
                    });

                    chunk_start = i;
                }
            }
        }

        chunks
    }
}
```

---

### 2. Reed-Solomon Erasure Coding

**Purpose:** Provide fault tolerance by distributing parity blocks across peers.

**Configuration:**
- 16 data blocks + 4 parity blocks
- Can recover from loss of any 4 blocks
- Overhead: 25% (4/16)

**Implementation:**
```rust
use reed_solomon::Encoder;

pub struct ErasureCoder {
    encoder: Encoder,
}

impl ErasureCoder {
    pub fn encode(&self, chunk: &[u8]) -> Vec<Vec<u8>> {
        let shard_size = (chunk.len() + 15) / 16;
        let mut shards = vec![vec![0u8; shard_size]; 20];

        // Split into 16 data shards
        for (i, chunk_slice) in chunk.chunks(shard_size).enumerate() {
            shards[i][..chunk_slice.len()].copy_from_slice(chunk_slice);
        }

        // Generate 4 parity shards
        self.encoder.encode(&mut shards).unwrap();

        shards
    }

    pub fn decode(&self, shards: &mut [Option<Vec<u8>>]) -> Vec<u8> {
        self.encoder.reconstruct(shards).unwrap();

        // Concatenate data shards
        let mut data = Vec::new();
        for i in 0..16 {
            if let Some(shard) = &shards[i] {
                data.extend_from_slice(shard);
            }
        }

        data
    }
}
```

---

### 3. Deduplication Index

**Purpose:** Track unique chunks to avoid storing duplicates.

**Schema:**
```sql
CREATE TABLE chunks (
    hash BLOB PRIMARY KEY,
    size INTEGER NOT NULL,
    ref_count INTEGER NOT NULL,
    first_seen INTEGER NOT NULL,
    storage_peers TEXT NOT NULL
);
```

**Deduplication Rate:**
- Documents/code: 60-80% savings
- Photos/media: 10-20% savings
- Overall: 50-70% savings

---

### 4. Backup Catalog

**Schema:**
```sql
CREATE TABLE backups (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    source_path TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    total_size INTEGER NOT NULL,
    compressed_size INTEGER NOT NULL,
    chunk_count INTEGER NOT NULL
);

CREATE TABLE backup_files (
    id INTEGER PRIMARY KEY,
    backup_id INTEGER NOT NULL,
    path TEXT NOT NULL,
    size INTEGER NOT NULL,
    modified_at INTEGER NOT NULL,
    chunks TEXT NOT NULL,  -- JSON array of chunk hashes
    FOREIGN KEY (backup_id) REFERENCES backups(id)
);

CREATE TABLE snapshots (
    id INTEGER PRIMARY KEY,
    backup_id INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (backup_id) REFERENCES backups(id)
);
```

---

## Data Flow

### Backup Process

```
1. User selects folder to backup
2. File scanner discovers all files
3. For each file:
   a. Chunker splits into variable-size chunks
   b. Each chunk compressed with zstd
   c. BLAKE3 hash calculated
   d. Check deduplication index:
      - If chunk exists: increment ref count
      - If new: continue to next step
   e. Erasure encode chunk (16+4 shards)
   f. Encrypt each shard with XChaCha20-Poly1305
   g. Distribute shards to 20 different peers
4. Update backup catalog
5. Create snapshot entry
```

### Restore Process

```
1. User selects backup/snapshot to restore
2. Query backup catalog for file list
3. For each file:
   a. Get chunk hashes from catalog
   b. For each chunk:
      - Query DHT for shard locations
      - Download at least 16/20 shards
      - Decrypt shards
      - Reed-Solomon decode to recover chunk
      - Decompress chunk
   c. Reassemble chunks into file
4. Verify file hash
5. Restore metadata (timestamps, permissions)
```

---

## Performance Characteristics

**Backup:**
- Initial: 100 GB in <2 hours (1 Gbps)
- Incremental: Only changed chunks
- Deduplication: 60% average savings

**Restore:**
- 100 GB in <3 hours (1 Gbps)
- Parallel chunk download
- On-the-fly decompression

**Storage Overhead:**
- Erasure coding: +25% (4/16)
- Metadata: ~100 bytes per file
- Net with deduplication: Often break-even or better

---

## See Also

- [Features](features.md)
- [Implementation](implementation.md)
- [Client Overview](../overview.md)
