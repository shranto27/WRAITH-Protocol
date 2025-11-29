# WRAITH-Transfer Implementation

**Document Version:** 1.0.0
**Last Updated:** 2025-11-28
**Client Version:** 1.0.0

---

## Overview

This document provides detailed implementation guidance for WRAITH-Transfer, including technology stack, code architecture, key algorithms, and deployment strategies.

---

## Technology Stack

### Backend (Rust)

**Core Dependencies:**
```toml
[dependencies]
# WRAITH Protocol
wraith-core = { path = "../../crates/wraith-core" }
wraith-files = { path = "../../crates/wraith-files" }
wraith-crypto = { path = "../../crates/wraith-crypto" }
wraith-transport = { path = "../../crates/wraith-transport" }
wraith-discovery = { path = "../../crates/wraith-discovery" }

# Async Runtime
tokio = { version = "1.40", features = ["full"] }
futures = "0.3"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
bincode = "1.3"

# Hashing
blake3 = "1.5"

# Compression
lz4 = "1.24"

# Database
rusqlite = { version = "0.31", features = ["bundled"] }

# Error Handling
anyhow = "1.0"
thiserror = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Tauri Integration
tauri = { version = "2.0", features = ["protocol-asset", "fs-all", "dialog-all", "notification-all"] }
tauri-plugin-fs = "2.0"
```

### Frontend (TypeScript + React)

**Core Dependencies:**
```json
{
  "dependencies": {
    "react": "^18.3.0",
    "react-dom": "^18.3.0",
    "@tanstack/react-query": "^5.0.0",
    "@tauri-apps/api": "^2.0.0",
    "lucide-react": "^0.400.0",
    "recharts": "^2.12.0",
    "qrcode": "^1.5.3",
    "jsqr": "^1.4.0"
  },
  "devDependencies": {
    "typescript": "^5.5.0",
    "vite": "^5.4.0",
    "@vitejs/plugin-react": "^4.3.0",
    "tailwindcss": "^3.4.0"
  }
}
```

---

## Code Architecture

### Project Structure

```
wraith-transfer/
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── main.rs         # Application entry point
│   │   ├── commands.rs     # Tauri commands
│   │   ├── transfer/
│   │   │   ├── mod.rs
│   │   │   ├── manager.rs  # Transfer manager
│   │   │   ├── sender.rs   # Send logic
│   │   │   ├── receiver.rs # Receive logic
│   │   │   └── progress.rs # Progress tracking
│   │   ├── peers/
│   │   │   ├── mod.rs
│   │   │   ├── manager.rs  # Peer management
│   │   │   └── discovery.rs# Peer discovery
│   │   ├── storage/
│   │   │   ├── mod.rs
│   │   │   ├── history.rs  # Transfer history
│   │   │   └── state.rs    # State persistence
│   │   └── config.rs       # Configuration
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── src/                    # React frontend
│   ├── App.tsx
│   ├── main.tsx
│   ├── components/
│   │   ├── Sidebar.tsx
│   │   ├── FileDropZone.tsx
│   │   ├── TransferProgress.tsx
│   │   ├── PeerList.tsx
│   │   ├── QRCodePairing.tsx
│   │   └── HistoryView.tsx
│   ├── hooks/
│   │   ├── useTransfer.ts
│   │   ├── usePeers.ts
│   │   └── useSettings.ts
│   ├── lib/
│   │   ├── tauri.ts       # Tauri API wrappers
│   │   └── utils.ts
│   └── views/
│       ├── SendView.tsx
│       ├── ReceiveView.tsx
│       ├── HistoryView.tsx
│       └── SettingsView.tsx
│
├── package.json
└── vite.config.ts
```

### Core Components

#### Transfer Manager

```rust
// src-tauri/src/transfer/manager.rs
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use wraith_files::FileTransfer;
use wraith_core::Session;

pub struct TransferManager {
    active_transfers: Arc<RwLock<HashMap<TransferId, TransferState>>>,
    session_pool: Arc<SessionPool>,
    config: TransferConfig,
    history: Arc<TransferHistory>,
}

#[derive(Clone, Debug)]
pub struct TransferState {
    pub id: TransferId,
    pub file_name: String,
    pub total_size: u64,
    pub transferred: u64,
    pub throughput: f64,
    pub peer_id: PeerId,
    pub status: TransferStatus,
    pub started_at: SystemTime,
}

#[derive(Clone, Debug)]
pub enum TransferStatus {
    Pending,
    Connecting,
    Transferring,
    Paused,
    Completed,
    Failed(String),
    Cancelled,
}

impl TransferManager {
    pub fn new(config: TransferConfig) -> Self {
        Self {
            active_transfers: Arc::new(RwLock::new(HashMap::new())),
            session_pool: Arc::new(SessionPool::new()),
            config,
            history: Arc::new(TransferHistory::new()),
        }
    }

    pub async fn send_file(
        &self,
        path: PathBuf,
        peer_id: PeerId,
    ) -> Result<TransferId> {
        let transfer_id = TransferId::new();
        let file_size = tokio::fs::metadata(&path).await?.len();

        // Create transfer state
        let state = TransferState {
            id: transfer_id,
            file_name: path.file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            total_size: file_size,
            transferred: 0,
            throughput: 0.0,
            peer_id: peer_id.clone(),
            status: TransferStatus::Pending,
            started_at: SystemTime::now(),
        };

        self.active_transfers.write().await.insert(transfer_id, state);

        // Spawn transfer task
        let manager = self.clone();
        tokio::spawn(async move {
            manager.run_send_task(transfer_id, path, peer_id).await
        });

        Ok(transfer_id)
    }

    async fn run_send_task(
        &self,
        id: TransferId,
        path: PathBuf,
        peer_id: PeerId,
    ) -> Result<()> {
        // Update status to connecting
        self.update_status(id, TransferStatus::Connecting).await;

        // Get or create session
        let session = self.session_pool
            .get_or_create(peer_id.clone())
            .await?;

        // Update status to transferring
        self.update_status(id, TransferStatus::Transferring).await;

        // Create file transfer
        let mut transfer = FileTransfer::new(session);

        // Send file with progress callback
        let manager_clone = self.clone();
        transfer.send_file(path, move |progress| {
            let manager = manager_clone.clone();
            tokio::spawn(async move {
                manager.update_progress(id, progress).await;
            });
        }).await?;

        // Update status to completed
        self.update_status(id, TransferStatus::Completed).await;

        // Record in history
        self.history.record_transfer(id, true).await;

        Ok(())
    }

    pub async fn receive_file(
        &self,
        output_path: PathBuf,
    ) -> Result<TransferId> {
        let transfer_id = TransferId::new();

        let state = TransferState {
            id: transfer_id,
            file_name: "Incoming...".to_string(),
            total_size: 0,
            transferred: 0,
            throughput: 0.0,
            peer_id: PeerId::unknown(),
            status: TransferStatus::Pending,
            started_at: SystemTime::now(),
        };

        self.active_transfers.write().await.insert(transfer_id, state);

        let manager = self.clone();
        tokio::spawn(async move {
            manager.run_receive_task(transfer_id, output_path).await
        });

        Ok(transfer_id)
    }

    async fn run_receive_task(
        &self,
        id: TransferId,
        output_path: PathBuf,
    ) -> Result<()> {
        // Listen for incoming transfer
        self.update_status(id, TransferStatus::Connecting).await;

        // Accept incoming connection
        let session = self.session_pool.accept_incoming().await?;

        self.update_status(id, TransferStatus::Transferring).await;

        // Receive file
        let mut transfer = FileTransfer::new(session);

        let manager_clone = self.clone();
        transfer.receive_file(output_path, move |progress| {
            let manager = manager_clone.clone();
            tokio::spawn(async move {
                manager.update_progress(id, progress).await;
            });
        }).await?;

        self.update_status(id, TransferStatus::Completed).await;
        self.history.record_transfer(id, true).await;

        Ok(())
    }

    async fn update_progress(&self, id: TransferId, progress: TransferProgress) {
        let mut transfers = self.active_transfers.write().await;
        if let Some(state) = transfers.get_mut(&id) {
            state.transferred = progress.bytes_transferred;
            state.throughput = progress.throughput;
        }
    }

    async fn update_status(&self, id: TransferId, status: TransferStatus) {
        let mut transfers = self.active_transfers.write().await;
        if let Some(state) = transfers.get_mut(&id) {
            state.status = status;
        }
    }

    pub async fn get_progress(&self, id: TransferId) -> Option<TransferState> {
        self.active_transfers.read().await.get(&id).cloned()
    }

    pub async fn pause(&self, id: TransferId) -> Result<()> {
        self.update_status(id, TransferStatus::Paused).await;
        // Implementation: Signal transfer task to pause
        Ok(())
    }

    pub async fn resume(&self, id: TransferId) -> Result<()> {
        self.update_status(id, TransferStatus::Transferring).await;
        // Implementation: Signal transfer task to resume
        Ok(())
    }

    pub async fn cancel(&self, id: TransferId) -> Result<()> {
        self.update_status(id, TransferStatus::Cancelled).await;
        self.active_transfers.write().await.remove(&id);
        Ok(())
    }
}
```

#### Tauri Commands

```rust
// src-tauri/src/commands.rs
use tauri::State;
use std::path::PathBuf;

#[tauri::command]
pub async fn send_file(
    path: PathBuf,
    peer_id: String,
    state: State<'_, TransferManager>,
) -> Result<String, String> {
    let peer = PeerId::from_string(&peer_id)
        .map_err(|e| e.to_string())?;

    let transfer_id = state.send_file(path, peer)
        .await
        .map_err(|e| e.to_string())?;

    Ok(transfer_id.to_string())
}

#[tauri::command]
pub async fn receive_file(
    output_path: PathBuf,
    state: State<'_, TransferManager>,
) -> Result<String, String> {
    let transfer_id = state.receive_file(output_path)
        .await
        .map_err(|e| e.to_string())?;

    Ok(transfer_id.to_string())
}

#[tauri::command]
pub async fn get_transfer_progress(
    transfer_id: String,
    state: State<'_, TransferManager>,
) -> Result<Option<TransferState>, String> {
    let id = TransferId::from_string(&transfer_id)
        .map_err(|e| e.to_string())?;

    Ok(state.get_progress(id).await)
}

#[tauri::command]
pub async fn pause_transfer(
    transfer_id: String,
    state: State<'_, TransferManager>,
) -> Result<(), String> {
    let id = TransferId::from_string(&transfer_id)
        .map_err(|e| e.to_string())?;

    state.pause(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cancel_transfer(
    transfer_id: String,
    state: State<'_, TransferManager>,
) -> Result<(), String> {
    let id = TransferId::from_string(&transfer_id)
        .map_err(|e| e.to_string())?;

    state.cancel(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_peers(
    state: State<'_, PeerManager>,
) -> Result<Vec<PeerInfo>, String> {
    state.list_peers().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_peer(
    name: String,
    address: String,
    state: State<'_, PeerManager>,
) -> Result<(), String> {
    state.add_peer(name, address)
        .await
        .map_err(|e| e.to_string())
}
```

#### React Frontend Integration

```typescript
// src/hooks/useTransfer.ts
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';

export interface TransferState {
  id: string;
  fileName: string;
  totalSize: number;
  transferred: number;
  throughput: number;
  peerId: string;
  status: 'Pending' | 'Connecting' | 'Transferring' | 'Paused' | 'Completed' | 'Failed' | 'Cancelled';
  startedAt: number;
}

export function useTransfer(transferId: string | null) {
  const [state, setState] = useState<TransferState | null>(null);

  useEffect(() => {
    if (!transferId) return;

    const interval = setInterval(async () => {
      const progress = await invoke<TransferState | null>(
        'get_transfer_progress',
        { transferId }
      );
      setState(progress);
    }, 100); // Update every 100ms

    return () => clearInterval(interval);
  }, [transferId]);

  const pause = async () => {
    if (!transferId) return;
    await invoke('pause_transfer', { transferId });
  };

  const resume = async () => {
    if (!transferId) return;
    await invoke('resume_transfer', { transferId });
  };

  const cancel = async () => {
    if (!transferId) return;
    await invoke('cancel_transfer', { transferId });
  };

  return { state, pause, resume, cancel };
}

export function useSendFile() {
  const send = async (path: string, peerId: string) => {
    return await invoke<string>('send_file', { path, peerId });
  };

  return { send };
}

export function useReceiveFile() {
  const receive = async (outputPath: string) => {
    return await invoke<string>('receive_file', { outputPath });
  };

  return { receive };
}
```

```tsx
// src/components/TransferProgress.tsx
import React from 'react';
import { useTransfer } from '../hooks/useTransfer';
import { Pause, Play, X } from 'lucide-react';

interface TransferProgressProps {
  transferId: string;
}

export function TransferProgress({ transferId }: TransferProgressProps) {
  const { state, pause, resume, cancel } = useTransfer(transferId);

  if (!state) return <div>Loading...</div>;

  const percentage = (state.transferred / state.totalSize) * 100;
  const speedMBps = (state.throughput / 1024 / 1024).toFixed(2);
  const remainingBytes = state.totalSize - state.transferred;
  const etaSeconds = state.throughput > 0
    ? Math.ceil(remainingBytes / state.throughput)
    : 0;

  return (
    <div className="transfer-progress">
      <div className="header">
        <h3>{state.fileName}</h3>
        <span className="status">{state.status}</span>
      </div>

      <div className="progress-bar">
        <div
          className="progress-fill"
          style={{ width: `${percentage}%` }}
        />
      </div>

      <div className="stats">
        <span>{percentage.toFixed(1)}%</span>
        <span>{speedMBps} MB/s</span>
        <span>ETA: {etaSeconds}s</span>
      </div>

      <div className="controls">
        {state.status === 'Transferring' && (
          <button onClick={pause}>
            <Pause size={20} /> Pause
          </button>
        )}
        {state.status === 'Paused' && (
          <button onClick={resume}>
            <Play size={20} /> Resume
          </button>
        )}
        <button onClick={cancel} className="danger">
          <X size={20} /> Cancel
        </button>
      </div>
    </div>
  );
}
```

---

## Key Algorithms

### Chunking Algorithm

```rust
const CHUNK_SIZE: usize = 1024 * 1024; // 1 MB

pub fn chunk_file(path: &Path) -> Result<Vec<Chunk>> {
    let file = File::open(path)?;
    let file_size = file.metadata()?.len();
    let num_chunks = (file_size + CHUNK_SIZE as u64 - 1) / CHUNK_SIZE as u64;

    let mut chunks = Vec::with_capacity(num_chunks as usize);
    let mut reader = BufReader::new(file);
    let mut buffer = vec![0u8; CHUNK_SIZE];

    for i in 0..num_chunks {
        let n = reader.read(&mut buffer)?;
        if n == 0 { break; }

        let chunk_data = &buffer[..n];
        let hash = blake3::hash(chunk_data);

        chunks.push(Chunk {
            index: i,
            size: n,
            hash: hash.into(),
            data: chunk_data.to_vec(),
        });
    }

    Ok(chunks)
}
```

### Multi-Peer Download Algorithm

```rust
pub async fn download_from_multiple_peers(
    file_hash: Hash,
    peers: Vec<PeerId>,
    output_path: PathBuf,
) -> Result<()> {
    // Get file metadata from first peer
    let metadata = get_file_metadata(&peers[0], file_hash).await?;
    let num_chunks = metadata.chunk_count;

    // Distribute chunks across peers
    let chunks_per_peer = (num_chunks + peers.len() - 1) / peers.len();
    let mut tasks = Vec::new();

    for (i, peer) in peers.iter().enumerate() {
        let start_chunk = i * chunks_per_peer;
        let end_chunk = ((i + 1) * chunks_per_peer).min(num_chunks);

        let peer = peer.clone();
        let file_hash = file_hash.clone();

        let task = tokio::spawn(async move {
            download_chunks(peer, file_hash, start_chunk..end_chunk).await
        });

        tasks.push(task);
    }

    // Wait for all downloads to complete
    let chunk_sets = futures::future::try_join_all(tasks).await?;

    // Merge chunks and write to file
    let mut chunks: Vec<_> = chunk_sets.into_iter().flatten().collect();
    chunks.sort_by_key(|c| c.index);

    let mut file = File::create(output_path)?;
    for chunk in chunks {
        file.write_all(&chunk.data)?;
    }

    Ok(())
}
```

---

## Build and Deployment

### Development Build

```bash
# Install dependencies
cd wraith-transfer
npm install

# Run development server
npm run tauri dev
```

### Production Build

```bash
# Build for current platform
npm run tauri build

# Outputs:
# - Windows: src-tauri/target/release/wraith-transfer.exe
# - macOS: src-tauri/target/release/bundle/macos/WRAITH Transfer.app
# - Linux: src-tauri/target/release/wraith-transfer
```

### Cross-Platform Builds

**Windows (from Linux):**
```bash
# Install Windows cross-compilation tools
rustup target add x86_64-pc-windows-gnu
sudo apt install mingw-w64

# Build
npm run tauri build -- --target x86_64-pc-windows-gnu
```

**macOS (from Linux):**
```bash
# Requires macOS SDK and osxcross
# See: https://github.com/tpoechtrager/osxcross

export OSXCROSS_ROOT=/path/to/osxcross
npm run tauri build -- --target x86_64-apple-darwin
```

### Code Signing

**Windows:**
```bash
# Sign with signtool
signtool sign /f certificate.pfx /p password /t http://timestamp.digicert.com wraith-transfer.exe
```

**macOS:**
```bash
# Sign and notarize
codesign --deep --force --verify --verbose \
  --sign "Developer ID Application: Your Name" \
  "WRAITH Transfer.app"

xcrun notarytool submit "WRAITH Transfer.app.zip" \
  --apple-id your@email.com \
  --password @keychain:AC_PASSWORD \
  --team-id TEAM_ID
```

**Linux:**
```bash
# Create AppImage
# Already handled by Tauri bundler
```

---

## Testing Approach

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_file() {
        let path = PathBuf::from("test_file.bin");
        create_test_file(&path, 5 * 1024 * 1024); // 5 MB

        let chunks = chunk_file(&path).unwrap();
        assert_eq!(chunks.len(), 5); // 5 chunks of 1 MB each

        for chunk in chunks {
            assert!(chunk.size <= CHUNK_SIZE);
        }
    }

    #[tokio::test]
    async fn test_transfer_manager() {
        let manager = TransferManager::new(TransferConfig::default());
        let peer_id = PeerId::new();
        let path = PathBuf::from("test_file.bin");

        let transfer_id = manager.send_file(path, peer_id).await.unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;

        let state = manager.get_progress(transfer_id).await.unwrap();
        assert!(matches!(state.status, TransferStatus::Transferring));
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_end_to_end_transfer() {
    // Start two instances
    let sender = create_test_instance("sender").await;
    let receiver = create_test_instance("receiver").await;

    // Create test file
    let test_file = PathBuf::from("/tmp/test.bin");
    create_test_file(&test_file, 10 * 1024 * 1024); // 10 MB

    // Start send
    let sender_task = tokio::spawn(async move {
        sender.send_file(test_file, receiver_peer_id).await
    });

    // Start receive
    let receiver_task = tokio::spawn(async move {
        receiver.receive_file(PathBuf::from("/tmp/received.bin")).await
    });

    // Wait for both to complete
    let (send_result, recv_result) = tokio::join!(sender_task, receiver_task);

    assert!(send_result.is_ok());
    assert!(recv_result.is_ok());

    // Verify file integrity
    let original_hash = hash_file(&test_file).unwrap();
    let received_hash = hash_file(&PathBuf::from("/tmp/received.bin")).unwrap();
    assert_eq!(original_hash, received_hash);
}
```

---

## Performance Optimization

### Memory-Mapped I/O

```rust
use memmap2::MmapMut;

fn write_chunk_mmap(file: &File, offset: u64, data: &[u8]) -> Result<()> {
    let mut mmap = unsafe { MmapMut::map_mut(file)? };

    let start = offset as usize;
    let end = start + data.len();

    mmap[start..end].copy_from_slice(data);
    mmap.flush()?;

    Ok(())
}
```

### Zero-Copy with io_uring

```rust
#[cfg(target_os = "linux")]
async fn send_chunk_uring(ring: &IoUring, chunk: &[u8]) -> Result<()> {
    let mut sqe = ring.next_sqe().unwrap();

    unsafe {
        io_uring::opcode::Write::new(
            io_uring::types::Fd(fd),
            chunk.as_ptr(),
            chunk.len() as u32
        ).build(&mut sqe);
    }

    ring.submit_and_wait(1)?;

    Ok(())
}
```

---

## See Also

- [Architecture](architecture.md)
- [Features](features.md)
- [Client Overview](../overview.md)
- [Protocol Implementation Guide](../../ref-docs/protocol_implementation_guide.md)
