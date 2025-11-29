# WRAITH-Transfer Architecture

**Document Version:** 1.0.0
**Last Updated:** 2025-11-28
**Client Version:** 1.0.0

---

## Overview

WRAITH-Transfer is a high-performance file transfer utility built on the WRAITH Protocol. It provides both CLI and GUI interfaces for secure, privacy-preserving file transfers between peers.

**Design Goals:**
- Maximum throughput (10+ Gbps)
- Minimal latency (<5 ms per packet)
- Resume support for interrupted transfers
- Multi-source downloads (swarm)
- Cross-platform compatibility

---

## Architecture Diagram

```
┌──────────────────────────────────────────────┐
│           User Interface Layer               │
│  ┌────────────┐          ┌────────────┐     │
│  │ CLI (Clap) │          │ GUI (Tauri)│     │
│  └────────────┘          └────────────┘     │
└──────────────────────────────────────────────┘
                    │
┌──────────────────────────────────────────────┐
│         Application Logic Layer              │
│  ┌──────────────────────────────────────┐   │
│  │  Transfer Manager                     │   │
│  │  - Progress tracking                  │   │
│  │  - Resume logic                       │   │
│  │  - Multi-source coordination          │   │
│  └──────────────────────────────────────┘   │
└──────────────────────────────────────────────┘
                    │
┌──────────────────────────────────────────────┐
│         WRAITH Protocol Stack                │
│  (wraith-files, wraith-discovery,            │
│   wraith-transport, wraith-core,             │
│   wraith-crypto)                             │
└──────────────────────────────────────────────┘
```

---

## Components

### 1. Transfer Manager

**Responsibilities:**
- Manage concurrent file transfers
- Track progress and statistics
- Handle errors and retries
- Coordinate multi-source downloads

**Implementation:**
```rust
pub struct TransferManager {
    active_transfers: HashMap<TransferId, ActiveTransfer>,
    session_pool: Arc<SessionPool>,
    config: TransferConfig,
}

impl TransferManager {
    pub async fn send_file(&mut self, path: PathBuf, peer: PeerId) -> Result<TransferId>;
    pub async fn receive_file(&mut self, output: PathBuf) -> Result<TransferId>;
    pub fn get_progress(&self, id: TransferId) -> Option<TransferProgress>;
    pub async fn cancel(&mut self, id: TransferId) -> Result<()>;
}
```

### 2. Chunk Manager

**Responsibilities:**
- Split files into chunks
- Verify chunk integrity (BLAKE3)
- Handle chunk deduplication
- Manage chunk priority

**Chunking Strategy:**
```
File (10 GB)
├── Chunk 0 (1 MB) [BLAKE3: abc123...]
├── Chunk 1 (1 MB) [BLAKE3: def456...]
├── Chunk 2 (1 MB) [BLAKE3: ghi789...]
│   ...
└── Chunk 9999 (1 MB) [BLAKE3: xyz000...]
```

### 3. Progress Tracker

**Metrics:**
- Bytes transferred / total
- Current throughput (Mbps)
- Estimated time remaining
- Peer-specific statistics

**Update frequency:** 100ms (UI), 1s (logs)

---

## File Transfer Protocol

### Handshake Phase

```
Sender                          Receiver
  │                                │
  │── FILE_OFFER ───────────────>  │
  │   (hash, size, chunk_count)    │
  │                                │
  │<─ FILE_ACCEPT ─────────────────│
  │   (accepted chunks)            │
  │                                │
```

### Data Transfer Phase

```
Sender                          Receiver
  │                                │
  │── CHUNK[0] ─────────────────>  │
  │── CHUNK[1] ─────────────────>  │
  │── CHUNK[2] ─────────────────>  │
  │                                │
  │<─ ACK[0-2] ────────────────────│
  │                                │
  │── CHUNK[3-5] ───────────────>  │
  │<─ ACK[3-5] ────────────────────│
  │                                │
```

### Completion Phase

```
Sender                          Receiver
  │                                │
  │<─ TRANSFER_COMPLETE ───────────│
  │   (final_hash)                 │
  │                                │
  │── VERIFIED ─────────────────>  │
  │                                │
```

---

## Multi-Source Downloads

### Peer Selection

```rust
fn select_best_peers(available: &[PeerInfo], needed: usize) -> Vec<PeerId> {
    available.sort_by(|a, b| {
        // Prioritize by:
        // 1. Latency (lower better)
        // 2. Throughput (higher better)
        // 3. Chunk availability (rarest first)
        (a.latency, -a.throughput, a.rarest_chunk)
            .cmp(&(b.latency, -b.throughput, b.rarest_chunk))
    });

    available.iter().take(needed).map(|p| p.id).collect()
}
```

### Chunk Distribution

```
Peer A: Chunks [0-99]
Peer B: Chunks [100-199]
Peer C: Chunks [200-299]
  ↓
Download in parallel, merge on completion
```

---

## Resume Support

### State Persistence

**Location:** `~/.cache/wraith-transfer/state/<transfer_id>.json`

**Format:**
```json
{
  "transfer_id": "abc123...",
  "file_hash": "def456...",
  "total_size": 10737418240,
  "chunk_size": 1048576,
  "completed_chunks": [0, 1, 2, 5, 6, 7],
  "peers": [
    {
      "id": "peer1",
      "address": "192.0.2.10:41641",
      "chunks": [0, 1, 2, 3, 4, 5]
    }
  ],
  "started_at": "2025-11-28T10:00:00Z",
  "last_updated": "2025-11-28T10:05:23Z"
}
```

### Resume Logic

```rust
pub async fn resume_transfer(id: TransferId) -> Result<()> {
    // 1. Load state
    let state = load_state(id)?;

    // 2. Reconnect to peers
    let sessions = reconnect_peers(&state.peers).await?;

    // 3. Resume from last checkpoint
    let remaining_chunks = calculate_remaining(&state.completed_chunks);

    // 4. Continue transfer
    download_chunks(sessions, remaining_chunks).await?;

    Ok(())
}
```

---

## Performance Optimizations

### 1. Parallel I/O

**io_uring integration:**
```rust
async fn write_chunks_parallel(chunks: Vec<Chunk>) -> Result<()> {
    let ring = io_uring::IoUring::new(256)?;

    for chunk in chunks {
        ring.prep_write(fd, chunk.offset, &chunk.data)?;
    }

    ring.submit_and_wait_all()?;
    Ok(())
}
```

### 2. Memory-Mapped I/O

```rust
use memmap2::MmapMut;

fn write_chunk_mmap(file: &File, offset: u64, data: &[u8]) -> Result<()> {
    let mut mmap = unsafe {
        MmapMut::map_mut(file)?
    };

    mmap[offset as usize..(offset + data.len() as u64) as usize]
        .copy_from_slice(data);

    mmap.flush()?;
    Ok(())
}
```

### 3. Zero-Copy Transmission

**AF_XDP:**
```rust
#[cfg(target_os = "linux")]
fn send_chunk_zerocopy(xdp: &mut XdpTransport, chunk: &[u8]) -> Result<()> {
    // Direct NIC buffer access
    let frame = xdp.allocate_tx_frame()?;
    frame.copy_from_slice(chunk);
    xdp.transmit(frame)?;
    Ok(())
}
```

---

## Error Handling

### Transient Errors

**Automatic retry:**
- Network timeouts → Retry 3x with exponential backoff
- Peer disconnect → Find alternative peer
- Chunk verification failure → Re-download from different peer

### Permanent Errors

**User notification:**
- File not found → Error, no retry
- Permission denied → Error, no retry
- Disk full → Error, pause transfer

---

## Configuration

**Default settings:**
```toml
[transfer]
chunk_size = 1048576  # 1 MB
max_parallel_chunks = 16
max_parallel_peers = 10
retry_attempts = 3
retry_delay = "5s"
verify_chunks = true
compression = false  # Optional LZ4
```

---

## See Also

- [Features](features.md)
- [Implementation](implementation.md)
- [Client Overview](../overview.md)
