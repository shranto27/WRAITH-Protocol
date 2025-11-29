# WRAITH-Vault Implementation

**Document Version:** 1.0.0
**Last Updated:** 2025-11-28
**Client Version:** 1.0.0

---

## Overview

Implementation details for WRAITH-Vault, including content-defined chunking, Reed-Solomon erasure coding, and deduplication.

---

## Technology Stack

```toml
[dependencies]
# WRAITH Protocol
wraith-core = { path = "../../crates/wraith-core" }
wraith-files = { path = "../../crates/wraith-files" }

# Chunking and hashing
blake3 = "1.5"

# Compression
zstd = "0.13"

# Erasure coding
reed-solomon-erasure = "6.0"

# Database
rusqlite = { version = "0.31", features = ["bundled"] }

# Encryption
chacha20poly1305 = "0.10"

# Async runtime
tokio = { version = "1.40", features = ["full"] }
```

---

## Content-Defined Chunking

```rust
// src/vault/chunker.rs
use blake3;

const MIN_CHUNK_SIZE: usize = 256 * 1024;   // 256 KB
const AVG_CHUNK_SIZE: usize = 1024 * 1024;  // 1 MB
const MAX_CHUNK_SIZE: usize = 8 * 1024 * 1024; // 8 MB

pub struct Chunker {
    rolling_hash: u32,
    window: Vec<u8>,
}

impl Chunker {
    pub fn new() -> Self {
        Self {
            rolling_hash: 0,
            window: Vec::with_capacity(64),
        }
    }

    pub fn chunk_file<R: Read>(&mut self, reader: &mut R) -> Result<Vec<Chunk>> {
        let mut chunks = Vec::new();
        let mut buffer = vec![0u8; MAX_CHUNK_SIZE];
        let mut chunk_start = 0;
        let mut offset = 0;

        loop {
            let n = reader.read(&mut buffer[offset..])?;
            if n == 0 { break; }

            offset += n;

            for i in chunk_start..offset {
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

            if chunk_start > 0 {
                buffer.copy_within(chunk_start..offset, 0);
                offset -= chunk_start;
                chunk_start = 0;
            }
        }

        if offset > 0 {
            let chunk_data = &buffer[..offset];
            let chunk_hash = blake3::hash(chunk_data);

            chunks.push(Chunk {
                hash: chunk_hash.into(),
                size: chunk_data.len(),
                data: chunk_data.to_vec(),
            });
        }

        Ok(chunks)
    }

    fn update_rolling_hash(&mut self, byte: u8) {
        if self.window.len() >= 64 {
            self.window.remove(0);
        }
        self.window.push(byte);

        // Rabin fingerprint
        self.rolling_hash = self.rolling_hash.rotate_left(1) ^ (byte as u32);
    }
}

pub struct Chunk {
    pub hash: [u8; 32],
    pub size: usize,
    pub data: Vec<u8>,
}
```

---

## Reed-Solomon Erasure Coding

```rust
// src/vault/erasure.rs
use reed_solomon_erasure::ReedSolomon;

pub struct ErasureCoder {
    encoder: ReedSolomon,
}

impl ErasureCoder {
    pub fn new() -> Self {
        // 16 data shards + 4 parity shards
        Self {
            encoder: ReedSolomon::new(16, 4).unwrap(),
        }
    }

    pub fn encode(&self, chunk: &[u8]) -> Result<Vec<Vec<u8>>> {
        let shard_size = (chunk.len() + 15) / 16;
        let mut shards = vec![vec![0u8; shard_size]; 20];

        // Split into data shards
        for (i, chunk_slice) in chunk.chunks(shard_size).enumerate() {
            shards[i][..chunk_slice.len()].copy_from_slice(chunk_slice);
        }

        // Generate parity shards
        self.encoder.encode(&mut shards)?;

        Ok(shards)
    }

    pub fn decode(&self, shards: Vec<Option<Vec<u8>>>) -> Result<Vec<u8>> {
        let mut shards_clone = shards.clone();

        // Reconstruct missing shards
        self.encoder.reconstruct(&mut shards_clone)?;

        // Concatenate data shards
        let mut data = Vec::new();
        for i in 0..16 {
            if let Some(shard) = &shards_clone[i] {
                data.extend_from_slice(shard);
            }
        }

        Ok(data)
    }
}
```

---

## Deduplication Index

```rust
// src/vault/dedup.rs
use std::collections::HashMap;

pub struct ChunkStore {
    chunk_index: HashMap<[u8; 32], ChunkInfo>,
    db: Arc<Database>,
}

pub struct ChunkInfo {
    pub hash: [u8; 32],
    pub size: usize,
    pub ref_count: usize,
    pub storage_peers: Vec<PeerId>,
}

impl ChunkStore {
    pub async fn add_chunk(&mut self, chunk: &Chunk) -> Result<bool> {
        if let Some(info) = self.chunk_index.get_mut(&chunk.hash) {
            // Chunk already exists - increment ref count
            info.ref_count += 1;
            self.db.update_chunk_refcount(&chunk.hash, info.ref_count).await?;

            Ok(false) // Not a new chunk
        } else {
            // New chunk - store in network
            let peers = self.distribute_chunk(chunk).await?;

            self.chunk_index.insert(chunk.hash, ChunkInfo {
                hash: chunk.hash,
                size: chunk.size,
                ref_count: 1,
                storage_peers: peers.clone(),
            });

            self.db.insert_chunk(&chunk.hash, chunk.size, &peers).await?;

            Ok(true) // New chunk
        }
    }

    async fn distribute_chunk(&self, chunk: &Chunk) -> Result<Vec<PeerId>> {
        // Compress
        let compressed = zstd::encode_all(&chunk.data[..], 3)?;

        // Erasure encode
        let coder = ErasureCoder::new();
        let shards = coder.encode(&compressed)?;

        // Encrypt each shard
        let mut storage_peers = Vec::new();

        for (i, shard) in shards.iter().enumerate() {
            let encrypted = encrypt_shard(shard, &chunk.hash, i)?;

            // Find peer to store this shard
            let peer = self.select_storage_peer().await?;

            // Send shard
            self.wraith.store_chunk(peer.id, &encrypted).await?;

            storage_peers.push(peer.id);
        }

        Ok(storage_peers)
    }
}

fn encrypt_shard(shard: &[u8], chunk_hash: &[u8; 32], shard_index: usize) -> Result<Vec<u8>> {
    let key = derive_shard_key(chunk_hash, shard_index);
    let nonce = [0u8; 24]; // Derived deterministically

    let cipher = XChaCha20Poly1305::new(&key.into());
    let encrypted = cipher.encrypt(&nonce.into(), shard)?;

    Ok(encrypted)
}
```

---

## Backup Scheduler

```rust
// src/vault/scheduler.rs
use tokio::time::{interval, Duration};

pub struct BackupScheduler {
    backups: Vec<ScheduledBackup>,
}

pub struct ScheduledBackup {
    pub id: BackupId,
    pub source_path: PathBuf,
    pub schedule: Schedule,
    pub last_run: Option<SystemTime>,
}

pub enum Schedule {
    Realtime,
    Hourly,
    Daily { hour: u8 },
    Weekly { day: u8, hour: u8 },
    Manual,
}

impl BackupScheduler {
    pub async fn run(&mut self) {
        let mut ticker = interval(Duration::from_secs(60)); // Check every minute

        loop {
            ticker.tick().await;

            for backup in &mut self.backups {
                if self.should_run(backup) {
                    let _ = self.run_backup(backup).await;
                    backup.last_run = Some(SystemTime::now());
                }
            }
        }
    }

    fn should_run(&self, backup: &ScheduledBackup) -> bool {
        match backup.schedule {
            Schedule::Realtime => true,
            Schedule::Hourly => {
                backup.last_run
                    .map(|last| last.elapsed().unwrap().as_secs() >= 3600)
                    .unwrap_or(true)
            }
            Schedule::Daily { hour } => {
                let now = chrono::Local::now();
                now.hour() == hour as u32
                    && backup.last_run
                        .map(|last| last.elapsed().unwrap().as_secs() >= 86400)
                        .unwrap_or(true)
            }
            Schedule::Manual => false,
        }
    }

    async fn run_backup(&self, backup: &ScheduledBackup) -> Result<()> {
        // Implementation of backup process
        Ok(())
    }
}
```

---

## Build and Deployment

```bash
# Development
cargo run

# Production
cargo build --release

# Platform packages
cargo tauri build

# NAS packages (requires Docker)
docker build -t wraith-vault-synology -f Dockerfile.synology .
docker build -t wraith-vault-qnap -f Dockerfile.qnap .
```

---

## See Also

- [Architecture](architecture.md)
- [Features](features.md)
- [Client Overview](../overview.md)
