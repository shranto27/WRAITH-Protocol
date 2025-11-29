# WRAITH Protocol Embedding Guide

**Document Version:** 1.0.0
**Last Updated:** 2025-11-28
**Status:** Integration Documentation

---

## Overview

This guide explains how to embed the WRAITH Protocol into your applications. WRAITH is designed to be integrated as a library, providing secure, privacy-preserving file transfer capabilities to any Rust application.

**Use Cases:**
- Desktop applications (file sharing, backup, sync)
- Mobile applications (secure messaging, file transfer)
- IoT devices (firmware updates, sensor data collection)
- Enterprise systems (secure document exchange)
- Web services (backend file storage and distribution)

---

## Integration Methods

### As a Rust Library

**Add WRAITH to your Cargo.toml:**
```toml
[dependencies]
wraith-core = { path = "../wraith-protocol/crates/wraith-core" }
wraith-crypto = { path = "../wraith-protocol/crates/wraith-crypto" }
wraith-transport = { path = "../wraith-protocol/crates/wraith-transport" }
wraith-files = { path = "../wraith-protocol/crates/wraith-files" }

# Or from crates.io (when published)
wraith-core = "0.1"
wraith-crypto = "0.1"
wraith-transport = "0.1"
wraith-files = "0.1"
```

### As a C Library (FFI)

WRAITH can be compiled as a C-compatible library for integration with non-Rust applications.

**Build shared library:**
```bash
cargo build --release --crate-type cdylib
```

**Header generation:**
```bash
cbindgen --config cbindgen.toml --crate wraith-ffi --output wraith.h
```

---

## Basic Integration

### Minimal Example

**Simple file transfer:**
```rust
use wraith_core::{Session, Keypair};
use wraith_transport::UdpTransport;
use wraith_files::FileTransfer;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Generate or load keypair
    let keypair = Keypair::generate();

    // 2. Create transport
    let transport = UdpTransport::bind("0.0.0.0:0".parse()?).await?;

    // 3. Connect to peer
    let peer_addr: SocketAddr = "192.0.2.10:41641".parse()?;
    let session = Session::connect(
        keypair,
        peer_addr,
        transport,
    ).await?;

    // 4. Transfer file
    let mut transfer = FileTransfer::new(session, Default::default());
    transfer.send_file("document.pdf", None).await?;

    println!("File transferred successfully!");
    Ok(())
}
```

### With Progress Tracking

```rust
use indicatif::{ProgressBar, ProgressStyle};
use wraith_files::TransferProgress;

async fn transfer_with_progress() -> Result<()> {
    let session = /* ... create session ... */;
    let mut transfer = FileTransfer::new(session, Default::default());

    // Create progress bar
    let pb = ProgressBar::new(0);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes} ({eta})")?
            .progress_chars("#>-")
    );

    // Transfer with progress callback
    transfer.send_file("large_file.bin", Some(|progress: TransferProgress| {
        pb.set_length(progress.total_bytes);
        pb.set_position(progress.bytes_transferred);
    })).await?;

    pb.finish_with_message("Transfer complete!");
    Ok(())
}
```

---

## Advanced Integration

### Custom Configuration

```rust
use wraith_core::SessionConfig;
use wraith_files::TransferConfig;
use std::time::Duration;

let session_config = SessionConfig {
    handshake_timeout: Duration::from_secs(10),
    idle_timeout: Duration::from_secs(30),
    max_packet_size: 1200,
    enable_migration: true,
};

let transfer_config = TransferConfig {
    chunk_size: 1024 * 1024,  // 1 MB chunks
    max_parallel_chunks: 16,
    verify_chunks: true,
    compression: Some(CompressionType::Lz4),
};

let session = Session::connect_with_config(
    keypair,
    peer_addr,
    transport,
    session_config,
).await?;

let mut transfer = FileTransfer::new(session, transfer_config);
```

### Multi-Peer Transfer

```rust
use futures::stream::{self, StreamExt};

async fn download_from_multiple_peers(
    peers: Vec<SocketAddr>,
    file_hash: Blake3Hash,
) -> Result<Vec<u8>> {
    // Connect to all peers in parallel
    let sessions: Vec<Session> = stream::iter(peers)
        .map(|peer_addr| async move {
            let transport = UdpTransport::bind("0.0.0.0:0".parse()?).await?;
            Session::connect(keypair.clone(), peer_addr, transport).await
        })
        .buffer_unordered(10)
        .filter_map(|r| async { r.ok() })
        .collect()
        .await;

    // Download different chunks from different peers
    let mut file_data = vec![0u8; file_size];
    let chunk_size = 1024 * 1024;

    for (i, session) in sessions.iter().enumerate() {
        let chunk_range = (i * chunk_size)..((i + 1) * chunk_size);
        let chunk = download_chunk(session, file_hash, chunk_range).await?;
        file_data[chunk_range].copy_from_slice(&chunk);
    }

    Ok(file_data)
}
```

### Connection Pooling

```rust
use std::collections::HashMap;
use tokio::sync::Mutex;

pub struct SessionPool {
    sessions: Arc<Mutex<HashMap<PublicKey, Session>>>,
}

impl SessionPool {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_or_create(
        &self,
        peer_pubkey: PublicKey,
        peer_addr: SocketAddr,
    ) -> Result<Session> {
        let mut sessions = self.sessions.lock().await;

        if let Some(session) = sessions.get(&peer_pubkey) {
            // Reuse existing session
            Ok(session.clone())
        } else {
            // Create new session
            let transport = UdpTransport::bind("0.0.0.0:0".parse()?).await?;
            let session = Session::connect(
                keypair.clone(),
                peer_addr,
                transport,
            ).await?;

            sessions.insert(peer_pubkey, session.clone());
            Ok(session)
        }
    }

    pub async fn close_all(&self) {
        let sessions = self.sessions.lock().await;
        for (_, session) in sessions.iter() {
            let _ = session.close().await;
        }
    }
}
```

---

## DHT Integration

### Peer Discovery

```rust
use wraith_discovery::DhtNode;

async fn discover_and_transfer(
    group_secret: &[u8; 32],
    file_hash: &Blake3Hash,
) -> Result<()> {
    // 1. Initialize DHT
    let mut dht = DhtNode::new(Default::default());

    // 2. Bootstrap from known nodes
    let bootstrap = vec![
        "dht1.wraith.network:41641".parse()?,
        "dht2.wraith.network:41641".parse()?,
    ];
    dht.bootstrap(&bootstrap).await?;

    // 3. Find peers offering the file
    let peers = dht.find_peers(group_secret, file_hash).await?;
    println!("Found {} peers", peers.len());

    // 4. Connect to first peer
    let transport = UdpTransport::bind("0.0.0.0:0".parse()?).await?;
    let session = Session::connect(keypair, peers[0], transport).await?;

    // 5. Download file
    let mut transfer = FileTransfer::new(session, Default::default());
    transfer.recv_file("downloaded_file.bin", None).await?;

    Ok(())
}
```

### Announcing File Availability

```rust
async fn share_file(
    group_secret: &[u8; 32],
    file_path: &Path,
) -> Result<()> {
    // 1. Compute file hash
    let file_hash = Blake3Hash::hash_file(file_path)?;

    // 2. Get local endpoints
    let endpoints = vec![
        "192.0.2.10:41641".parse()?,  // LAN
        "203.0.113.50:41641".parse()?,  // WAN
    ];

    // 3. Announce to DHT
    let mut dht = DhtNode::new(Default::default());
    dht.bootstrap(&bootstrap_nodes).await?;
    dht.announce(group_secret, &file_hash, endpoints).await?;

    println!("File announced to DHT");

    // 4. Accept incoming connections
    let listener = UdpTransport::bind("0.0.0.0:41641".parse()?).await?;
    loop {
        let session = listener.accept().await?;
        tokio::spawn(handle_transfer(session, file_path.to_owned()));
    }
}

async fn handle_transfer(session: Session, file_path: PathBuf) -> Result<()> {
    let mut transfer = FileTransfer::new(session, Default::default());
    transfer.send_file(&file_path, None).await?;
    Ok(())
}
```

---

## Platform-Specific Integration

### Desktop Application (Tauri)

**Tauri command:**
```rust
// src-tauri/src/main.rs
use tauri::State;
use wraith_files::FileTransfer;

struct AppState {
    session: Arc<Mutex<Option<Session>>>,
}

#[tauri::command]
async fn send_file(
    path: String,
    peer_addr: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let session = state.session.lock().await;
    let session = session.as_ref().ok_or("Not connected")?;

    let mut transfer = FileTransfer::new(session.clone(), Default::default());
    transfer.send_file(path, None).await
        .map_err(|e| e.to_string())?;

    Ok("Transfer complete".to_string())
}

fn main() {
    tauri::Builder::default()
        .manage(AppState {
            session: Arc::new(Mutex::new(None)),
        })
        .invoke_handler(tauri::generate_handler![send_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Web Backend (Actix-Web)

```rust
use actix_web::{web, App, HttpServer, HttpResponse};
use wraith_files::FileTransfer;

async fn upload_file(
    data: web::Data<SessionPool>,
    peer_id: web::Path<String>,
    payload: web::Payload,
) -> HttpResponse {
    // Get session from pool
    let peer_pubkey: PublicKey = peer_id.parse().unwrap();
    let peer_addr: SocketAddr = resolve_peer(&peer_pubkey).await.unwrap();

    let session = data.get_or_create(peer_pubkey, peer_addr).await.unwrap();

    // Transfer file
    let mut transfer = FileTransfer::new(session, Default::default());
    match transfer.send_stream(payload).await {
        Ok(_) => HttpResponse::Ok().body("File uploaded"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = web::Data::new(SessionPool::new());

    HttpServer::new(move || {
        App::new()
            .app_data(pool.clone())
            .route("/upload/{peer_id}", web::post().to(upload_file))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

### Mobile (Flutter FFI)

**Rust FFI:**
```rust
// wraith-ffi/src/lib.rs
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn wraith_send_file(
    peer_addr: *const c_char,
    file_path: *const c_char,
) -> *mut c_char {
    let peer_addr = unsafe { CStr::from_ptr(peer_addr).to_str().unwrap() };
    let file_path = unsafe { CStr::from_ptr(file_path).to_str().unwrap() };

    match tokio::runtime::Runtime::new().unwrap().block_on(async {
        send_file_internal(peer_addr, file_path).await
    }) {
        Ok(_) => CString::new("Success").unwrap().into_raw(),
        Err(e) => CString::new(format!("Error: {}", e)).unwrap().into_raw(),
    }
}

#[no_mangle]
pub extern "C" fn wraith_free_string(s: *mut c_char) {
    unsafe {
        if !s.is_null() {
            CString::from_raw(s);
        }
    }
}
```

**Flutter Dart:**
```dart
import 'dart:ffi';
import 'package:ffi/ffi.dart';

typedef SendFileNative = Pointer<Utf8> Function(Pointer<Utf8>, Pointer<Utf8>);
typedef SendFileDart = Pointer<Utf8> Function(Pointer<Utf8>, Pointer<Utf8>);

class WraithFFI {
  late final DynamicLibrary _lib;
  late final SendFileDart _sendFile;

  WraithFFI() {
    _lib = DynamicLibrary.open('libwraith.so');
    _sendFile = _lib.lookupFunction<SendFileNative, SendFileDart>('wraith_send_file');
  }

  Future<String> sendFile(String peerAddr, String filePath) async {
    final peerAddrPtr = peerAddr.toNativeUtf8();
    final filePathPtr = filePath.toNativeUtf8();

    final resultPtr = _sendFile(peerAddrPtr, filePathPtr);
    final result = resultPtr.toDartString();

    malloc.free(peerAddrPtr);
    malloc.free(filePathPtr);

    return result;
  }
}
```

---

## Error Handling Integration

### Custom Error Types

```rust
use thiserror::Error;
use wraith_core::SessionError;
use wraith_files::TransferError;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("WRAITH session error: {0}")]
    Session(#[from] SessionError),

    #[error("WRAITH transfer error: {0}")]
    Transfer(#[from] TransferError),

    #[error("Application error: {0}")]
    App(String),
}

// Convert to HTTP response
impl actix_web::ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::Session(_) => HttpResponse::ServiceUnavailable().body(self.to_string()),
            AppError::Transfer(_) => HttpResponse::InternalServerError().body(self.to_string()),
            AppError::App(_) => HttpResponse::BadRequest().body(self.to_string()),
        }
    }
}
```

---

## Performance Tuning

### Buffer Sizing

```rust
use wraith_transport::TransportConfig;

let transport_config = TransportConfig {
    send_buffer_size: 2 * 1024 * 1024,  // 2 MB
    recv_buffer_size: 2 * 1024 * 1024,  // 2 MB
    ..Default::default()
};

let transport = UdpTransport::bind_with_config(
    "0.0.0.0:41641".parse()?,
    transport_config,
).await?;
```

### Thread Pool Configuration

```rust
// Configure Tokio runtime
let runtime = tokio::runtime::Builder::new_multi_thread()
    .worker_threads(8)
    .thread_name("wraith-worker")
    .enable_all()
    .build()?;

runtime.block_on(async {
    // Your WRAITH code here
});
```

---

## Security Considerations

### Key Management

**Secure key storage:**
```rust
use keyring::Entry;

// Store keypair in system keyring
fn save_keypair(keypair: &Keypair) -> Result<()> {
    let entry = Entry::new("wraith-app", "keypair")?;
    let seed = keypair.to_seed();
    entry.set_password(&hex::encode(seed))?;
    Ok(())
}

// Load keypair from keyring
fn load_keypair() -> Result<Keypair> {
    let entry = Entry::new("wraith-app", "keypair")?;
    let seed_hex = entry.get_password()?;
    let seed = hex::decode(seed_hex)?;
    Ok(Keypair::from_seed(&seed.try_into().unwrap()))
}
```

### Group Secret Management

```rust
use wraith_crypto::Blake3Hash;

// Derive group secret from passphrase
fn derive_group_secret(passphrase: &str) -> [u8; 32] {
    let context = "wraith-group-secret-v1";
    let derived = Blake3Hash::derive_key(context, passphrase.as_bytes(), 32);
    derived.try_into().unwrap()
}
```

---

## Testing Integration

### Mock Session for Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    struct MockSession {
        sent_data: Arc<Mutex<Vec<Vec<u8>>>>,
    }

    impl MockSession {
        fn new() -> Self {
            Self {
                sent_data: Arc::new(Mutex::new(Vec::new())),
            }
        }

        async fn send(&mut self, data: &[u8]) -> Result<()> {
            self.sent_data.lock().await.push(data.to_vec());
            Ok(())
        }

        async fn get_sent_data(&self) -> Vec<Vec<u8>> {
            self.sent_data.lock().await.clone()
        }
    }

    #[tokio::test]
    async fn test_file_transfer() {
        let mut session = MockSession::new();
        session.send(b"test data").await.unwrap();

        let sent = session.get_sent_data().await;
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0], b"test data");
    }
}
```

---

## Best Practices

1. **Connection Management:** Use connection pooling for frequent transfers
2. **Error Handling:** Always handle session errors gracefully
3. **Progress Tracking:** Provide user feedback for long transfers
4. **Resource Cleanup:** Close sessions when done
5. **Security:** Store keys securely, never hardcode secrets
6. **Performance:** Tune buffer sizes for your use case
7. **Testing:** Use mocks for unit tests, integration tests for full flow

---

## See Also

- [Platform Support](platform-support.md)
- [Interoperability](interoperability.md)
- [API Reference](../engineering/api-reference.md)
- [Security Model](../architecture/security-model.md)
