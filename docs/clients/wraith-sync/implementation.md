# WRAITH-Sync Implementation

**Document Version:** 1.0.0
**Last Updated:** 2025-11-28
**Client Version:** 1.0.0

---

## Overview

Implementation details for WRAITH-Sync, including sync algorithms, delta sync implementation, and conflict resolution strategies.

---

## Technology Stack

```toml
[dependencies]
# WRAITH Protocol
wraith-core = { path = "../../crates/wraith-core" }
wraith-files = { path = "../../crates/wraith-files" }

# File watching
notify = "6.1"  # Cross-platform file watcher

# Hashing
blake3 = "1.5"

# Compression
zstd = "0.13"

# Database
rusqlite = { version = "0.31", features = ["bundled"] }

# Delta sync
rdiff = "0.2"  # rsync algorithm

# Async runtime
tokio = { version = "1.40", features = ["full"] }
```

---

## Sync Engine Implementation

```rust
// src/sync/engine.rs
use blake3;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;

pub struct SyncEngine {
    db: Arc<Database>,
    wraith: Arc<WraithClient>,
    file_watcher: Arc<FileWatcher>,
    sync_queue: Arc<RwLock<VecDeque<SyncOperation>>>,
}

#[derive(Clone)]
pub struct FileState {
    pub path: PathBuf,
    pub hash: [u8; 32],
    pub size: u64,
    pub modified_at: SystemTime,
    pub is_directory: bool,
}

pub enum SyncOperation {
    Upload(PathBuf),
    Download(PathBuf),
    Delete(PathBuf),
    Conflict(PathBuf, ConflictInfo),
}

impl SyncEngine {
    pub async fn sync_folder(&self, folder_id: u64) -> Result<()> {
        // Get local file states
        let local_files = self.scan_folder(folder_id).await?;

        // Get remote file states from all devices
        let devices = self.db.get_devices().await?;
        let mut remote_states = HashMap::new();

        for device in devices {
            let states = self.query_device_state(device.id, folder_id).await?;
            for state in states {
                remote_states
                    .entry(state.path.clone())
                    .or_insert_with(Vec::new)
                    .push((device.id.clone(), state));
            }
        }

        // Determine sync operations
        for (path, local_state) in &local_files {
            if let Some(remote) = remote_states.get(path) {
                let op = self.resolve_sync_operation(
                    local_state,
                    remote,
                ).await?;

                if let Some(op) = op {
                    self.sync_queue.write().await.push_back(op);
                }
            } else {
                // File only exists locally - upload
                self.sync_queue.write().await.push_back(
                    SyncOperation::Upload(path.clone())
                );
            }
        }

        // Process sync queue
        self.process_queue().await?;

        Ok(())
    }

    async fn scan_folder(&self, folder_id: u64) -> Result<HashMap<PathBuf, FileState>> {
        let folder = self.db.get_folder(folder_id).await?;
        let mut states = HashMap::new();

        let mut entries = fs::read_dir(&folder.local_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let metadata = entry.metadata().await?;

            if metadata.is_file() {
                let hash = compute_file_hash(&path).await?;

                states.insert(path.clone(), FileState {
                    path,
                    hash,
                    size: metadata.len(),
                    modified_at: metadata.modified()?,
                    is_directory: false,
                });
            }
        }

        Ok(states)
    }

    async fn resolve_sync_operation(
        &self,
        local: &FileState,
        remote: &[(String, FileState)],
    ) -> Result<Option<SyncOperation>> {
        // Find most recent remote version
        let latest_remote = remote.iter()
            .max_by_key(|(_, state)| state.modified_at)
            .unwrap();

        // Compare hashes
        if local.hash == latest_remote.1.hash {
            // Files identical - no sync needed
            return Ok(None);
        }

        // Compare modification times
        match local.modified_at.cmp(&latest_remote.1.modified_at) {
            std::cmp::Ordering::Greater => {
                // Local is newer - upload
                Ok(Some(SyncOperation::Upload(local.path.clone())))
            }
            std::cmp::Ordering::Less => {
                // Remote is newer - download
                Ok(Some(SyncOperation::Download(local.path.clone())))
            }
            std::cmp::Ordering::Equal => {
                // Same timestamp but different content - conflict
                Ok(Some(SyncOperation::Conflict(
                    local.path.clone(),
                    ConflictInfo {
                        local_hash: local.hash,
                        remote_hash: latest_remote.1.hash,
                        local_modified: local.modified_at,
                        remote_modified: latest_remote.1.modified_at,
                    }
                )))
            }
        }
    }

    async fn process_queue(&self) -> Result<()> {
        while let Some(op) = self.sync_queue.write().await.pop_front() {
            match op {
                SyncOperation::Upload(path) => self.upload_file(&path).await?,
                SyncOperation::Download(path) => self.download_file(&path).await?,
                SyncOperation::Delete(path) => self.delete_file(&path).await?,
                SyncOperation::Conflict(path, info) => {
                    self.handle_conflict(&path, info).await?
                }
            }
        }

        Ok(())
    }

    async fn upload_file(&self, path: &Path) -> Result<()> {
        // Read file
        let data = fs::read(path).await?;

        // Compress
        let compressed = zstd::encode_all(&data[..], 3)?;

        // Calculate hash
        let hash = blake3::hash(&compressed);

        // Send to all devices
        for device in self.db.get_devices().await? {
            self.wraith.send_file(
                device.peer_id,
                &compressed,
                hash.as_bytes(),
            ).await?;
        }

        // Update database
        self.db.update_file_state(path, &hash.as_bytes(), data.len() as u64).await?;

        Ok(())
    }
}

async fn compute_file_hash(path: &Path) -> Result<[u8; 32]> {
    let mut hasher = blake3::Hasher::new();
    let mut file = fs::File::open(path).await?;

    let mut buffer = vec![0u8; 65536]; // 64 KB buffer

    loop {
        let n = file.read(&mut buffer).await?;
        if n == 0 { break; }

        hasher.update(&buffer[..n]);
    }

    Ok(*hasher.finalize().as_bytes())
}
```

---

## Delta Sync Implementation

```rust
// src/sync/delta.rs
use rdiff;

pub struct DeltaSync {
    block_size: usize,
}

impl DeltaSync {
    pub fn new() -> Self {
        Self {
            block_size: 4096,  // 4 KB blocks
        }
    }

    pub async fn create_signature(&self, path: &Path) -> Result<Vec<BlockSignature>> {
        let file = fs::File::open(path).await?;
        let file_size = file.metadata().await?.len();
        let num_blocks = (file_size + self.block_size as u64 - 1) / self.block_size as u64;

        let mut signatures = Vec::with_capacity(num_blocks as usize);
        let mut buffer = vec![0u8; self.block_size];
        let mut offset = 0u64;

        loop {
            let n = file.read(&mut buffer).await?;
            if n == 0 { break; }

            let block_data = &buffer[..n];

            signatures.push(BlockSignature {
                offset,
                size: n,
                weak_hash: weak_checksum(block_data),
                strong_hash: blake3::hash(block_data).into(),
            });

            offset += n as u64;
        }

        Ok(signatures)
    }

    pub async fn compute_delta(
        &self,
        local_path: &Path,
        remote_signatures: &[BlockSignature],
    ) -> Result<Vec<DeltaOperation>> {
        let mut delta = Vec::new();
        let mut file = fs::File::open(local_path).await?;
        let mut buffer = vec![0u8; self.block_size];

        let sig_map: HashMap<u32, &BlockSignature> = remote_signatures
            .iter()
            .map(|s| (s.weak_hash, s))
            .collect();

        let mut offset = 0u64;

        loop {
            let n = file.read(&mut buffer).await?;
            if n == 0 { break; }

            let block_data = &buffer[..n];
            let weak = weak_checksum(block_data);

            // Check if block matches remote
            if let Some(sig) = sig_map.get(&weak) {
                let strong = blake3::hash(block_data);

                if strong.as_bytes() == &sig.strong_hash {
                    // Block unchanged - reference remote
                    delta.push(DeltaOperation::Copy {
                        offset: sig.offset,
                        size: sig.size,
                    });
                } else {
                    // Hash collision - send literal data
                    delta.push(DeltaOperation::Data {
                        data: block_data.to_vec(),
                    });
                }
            } else {
                // New block - send literal data
                delta.push(DeltaOperation::Data {
                    data: block_data.to_vec(),
                });
            }

            offset += n as u64;
        }

        Ok(delta)
    }

    pub async fn apply_delta(
        &self,
        base_path: &Path,
        delta: &[DeltaOperation],
        output_path: &Path,
    ) -> Result<()> {
        let mut base_file = fs::File::open(base_path).await?;
        let mut output_file = fs::File::create(output_path).await?;

        for op in delta {
            match op {
                DeltaOperation::Copy { offset, size } => {
                    // Copy block from base file
                    base_file.seek(SeekFrom::Start(*offset)).await?;
                    let mut buf = vec![0u8; *size];
                    base_file.read_exact(&mut buf).await?;
                    output_file.write_all(&buf).await?;
                }
                DeltaOperation::Data { data } => {
                    // Write literal data
                    output_file.write_all(data).await?;
                }
            }
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct BlockSignature {
    pub offset: u64,
    pub size: usize,
    pub weak_hash: u32,
    pub strong_hash: [u8; 32],
}

pub enum DeltaOperation {
    Copy { offset: u64, size: usize },
    Data { data: Vec<u8> },
}

fn weak_checksum(data: &[u8]) -> u32 {
    // Adler-32 checksum
    let mut a = 1u32;
    let mut b = 0u32;

    for &byte in data {
        a = (a + byte as u32) % 65521;
        b = (b + a) % 65521;
    }

    (b << 16) | a
}
```

---

## Conflict Resolution

```rust
// src/sync/conflict.rs
pub enum ConflictStrategy {
    LastWriterWins,
    KeepBoth,
    Manual,
}

impl SyncEngine {
    async fn handle_conflict(
        &self,
        path: &Path,
        info: ConflictInfo,
    ) -> Result<()> {
        let strategy = self.config.conflict_strategy;

        match strategy {
            ConflictStrategy::LastWriterWins => {
                // Most recent modification wins
                if info.local_modified > info.remote_modified {
                    self.upload_file(path).await?;
                } else {
                    self.download_file(path).await?;
                }
            }
            ConflictStrategy::KeepBoth => {
                // Save both versions
                let conflict_path = self.generate_conflict_path(path);

                // Rename local file
                fs::rename(path, &conflict_path).await?;

                // Download remote version
                self.download_file(path).await?;
            }
            ConflictStrategy::Manual => {
                // Store conflict in database for user resolution
                self.db.insert_conflict(path, &info).await?;

                // Notify UI
                self.notify_conflict(path, info).await?;
            }
        }

        Ok(())
    }

    fn generate_conflict_path(&self, path: &Path) -> PathBuf {
        let file_name = path.file_stem().unwrap().to_str().unwrap();
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");

        path.with_file_name(format!(
            "{} (conflict {}).{}",
            file_name,
            timestamp,
            extension
        ))
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

# Platform-specific packages
cargo tauri build
```

---

## See Also

- [Architecture](architecture.md)
- [Features](features.md)
- [Client Overview](../overview.md)
