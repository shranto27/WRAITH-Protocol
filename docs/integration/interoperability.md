# WRAITH Protocol Interoperability

**Document Version:** 1.0.0
**Last Updated:** 2025-11-28
**Status:** Integration Documentation

---

## Overview

WRAITH Protocol is designed to interoperate with existing systems and protocols while maintaining its privacy and security guarantees. This document describes integration points, compatibility considerations, and bridging strategies.

---

## Protocol Version Compatibility

### Version Negotiation

WRAITH implements forward and backward compatibility through protocol version negotiation.

**Version Format:**
```
WRAITH/<major>.<minor>

Example: WRAITH/1.0
```

**Compatibility Matrix:**

| Client Version | Server Version | Compatible? | Notes |
|---------------|----------------|-------------|-------|
| 1.0 | 1.0 | ✓ | Perfect match |
| 1.1 | 1.0 | ✓ | Backward compatible |
| 1.0 | 1.1 | ✓ | Forward compatible (degraded features) |
| 2.0 | 1.x | ✗ | Major version incompatibility |

**Negotiation Process:**
```rust
// Handshake includes version announcement
struct HandshakeInit {
    version: Version,
    supported_versions: Vec<Version>,
    // ... other fields
}

// Server responds with chosen version
struct HandshakeResponse {
    version: Version,  // Must be in client's supported_versions
    // ... other fields
}
```

---

## File System Integration

### Virtual File System (FUSE)

WRAITH can be exposed as a FUSE filesystem for seamless OS integration.

**Mount WRAITH as filesystem:**
```rust
use fuse::*;

struct WraithFS {
    dht: DhtNode,
    group_secret: [u8; 32],
}

impl Filesystem for WraithFS {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        // Lookup file in DHT by name
        let file_hash = blake3_hash(name.as_bytes());
        match self.dht.find_peers(&self.group_secret, &file_hash).await {
            Ok(peers) if !peers.is_empty() => {
                reply.entry(&TTL, &file_attr, 0);
            }
            _ => reply.error(ENOENT),
        }
    }

    fn read(&mut self, _req: &Request, ino: u64, fh: u64, offset: i64, size: u32, reply: ReplyData) {
        // Download chunk from WRAITH peers
        let chunk = download_chunk(ino, offset, size).await;
        reply.data(&chunk);
    }
}
```

**Usage:**
```bash
# Mount WRAITH group as directory
wraith-fuse mount --group-secret <secret> /mnt/wraith

# Access files normally
ls /mnt/wraith
cat /mnt/wraith/document.pdf
```

### S3-Compatible API

WRAITH can expose an S3-compatible API for cloud storage integration.

**Example implementation:**
```rust
use rusty_s3::{S3Action, Bucket};
use actix_web::{web, App, HttpServer};

async fn s3_get_object(
    bucket: web::Path<String>,
    key: web::Path<String>,
) -> Result<Vec<u8>> {
    // Map S3 object key to WRAITH file hash
    let file_hash = blake3_hash(key.as_bytes());

    // Retrieve from WRAITH network
    let peers = dht.find_peers(&group_secret, &file_hash).await?;
    let session = Session::connect(keypair, peers[0], transport).await?;
    let mut transfer = FileTransfer::new(session, Default::default());

    // Stream to S3 response
    transfer.recv_file_to_stream().await
}
```

---

## Network Protocol Bridges

### HTTP/HTTPS Gateway

Bridge WRAITH to HTTP for web browser access.

**Gateway server:**
```rust
use actix_web::{get, web, HttpResponse};

#[get("/download/{file_hash}")]
async fn download(
    file_hash: web::Path<String>,
    data: web::Data<AppState>,
) -> HttpResponse {
    let hash = Blake3Hash::from_hex(&file_hash).unwrap();

    // Find WRAITH peers
    let peers = data.dht.find_peers(&data.group_secret, &hash).await.unwrap();

    // Download via WRAITH
    let session = Session::connect(data.keypair.clone(), peers[0], transport).await.unwrap();
    let mut transfer = FileTransfer::new(session, Default::default());
    let file_data = transfer.recv_file_to_memory().await.unwrap();

    HttpResponse::Ok()
        .content_type("application/octet-stream")
        .body(file_data)
}
```

**Browser usage:**
```html
<!-- Download file via WRAITH gateway -->
<a href="https://gateway.example.com/download/abc123...">Download File</a>
```

### WebRTC Bridge

Enable browser-based WRAITH clients via WebRTC.

**Signaling server:**
```rust
use tokio_tungstenite::WebSocketStream;

async fn webrtc_signaling(ws: WebSocketStream<TcpStream>) {
    // Exchange ICE candidates between browser and WRAITH peer
    let offer = receive_webrtc_offer(ws).await;

    // Establish WRAITH session
    let session = Session::connect(keypair, peer_addr, transport).await.unwrap();

    // Bridge WebRTC data channel to WRAITH session
    tokio::spawn(bridge_webrtc_to_wraith(webrtc_channel, session));
}
```

---

## Storage Backends

### Local Storage

**File-based storage:**
```rust
use std::path::PathBuf;

struct LocalStorage {
    base_path: PathBuf,
}

impl LocalStorage {
    async fn store(&self, file_hash: &Blake3Hash, data: Vec<u8>) -> Result<()> {
        let path = self.base_path.join(file_hash.to_hex());
        tokio::fs::write(path, data).await?;
        Ok(())
    }

    async fn retrieve(&self, file_hash: &Blake3Hash) -> Result<Vec<u8>> {
        let path = self.base_path.join(file_hash.to_hex());
        tokio::fs::read(path).await.map_err(Into::into)
    }
}
```

### IPFS Integration

Bridge WRAITH to IPFS for content-addressed storage.

**IPFS adapter:**
```rust
use ipfs_api::IpfsClient;

async fn upload_to_ipfs(file_data: Vec<u8>) -> Result<String> {
    let client = IpfsClient::default();
    let response = client.add(file_data).await?;
    Ok(response.hash)
}

async fn wraith_to_ipfs_bridge(
    file_hash: Blake3Hash,
    group_secret: [u8; 32],
) -> Result<String> {
    // Download from WRAITH
    let peers = dht.find_peers(&group_secret, &file_hash).await?;
    let session = Session::connect(keypair, peers[0], transport).await?;
    let mut transfer = FileTransfer::new(session, Default::default());
    let file_data = transfer.recv_file_to_memory().await?;

    // Upload to IPFS
    upload_to_ipfs(file_data).await
}
```

### Database Backends

**PostgreSQL metadata storage:**
```rust
use sqlx::PgPool;

struct MetadataStore {
    pool: PgPool,
}

impl MetadataStore {
    async fn store_file_metadata(
        &self,
        file_hash: &Blake3Hash,
        filename: &str,
        size: u64,
    ) -> Result<()> {
        sqlx::query!(
            "INSERT INTO files (hash, filename, size, created_at) VALUES ($1, $2, $3, NOW())",
            file_hash.as_bytes(),
            filename,
            size as i64,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn list_files(&self) -> Result<Vec<FileMetadata>> {
        let rows = sqlx::query!("SELECT hash, filename, size FROM files")
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(|r| FileMetadata {
            hash: Blake3Hash::from_bytes(&r.hash),
            filename: r.filename,
            size: r.size as u64,
        }).collect())
    }
}
```

---

## Authentication Integration

### OAuth 2.0

Integrate WRAITH with OAuth 2.0 for user authentication.

**OAuth flow:**
```rust
use oauth2::{AuthorizationCode, TokenResponse};

async fn authenticate_user(code: String) -> Result<Keypair> {
    // Exchange authorization code for access token
    let token = oauth_client.exchange_code(AuthorizationCode::new(code))
        .request_async(async_http_client)
        .await?;

    // Derive WRAITH keypair from user ID
    let user_id = fetch_user_id(&token).await?;
    let seed = derive_seed_from_user_id(&user_id);

    Ok(Keypair::from_seed(&seed))
}
```

### JWT Integration

Use JWT tokens for group access control.

**JWT-based group secrets:**
```rust
use jsonwebtoken::{decode, Validation};

async fn verify_group_access(jwt: &str) -> Result<[u8; 32]> {
    let token = decode::<GroupClaims>(
        jwt,
        &DecodingKey::from_secret(JWT_SECRET.as_ref()),
        &Validation::default(),
    )?;

    // Group secret embedded in JWT claims
    Ok(token.claims.group_secret)
}

#[derive(Deserialize)]
struct GroupClaims {
    group_id: String,
    group_secret: [u8; 32],
    exp: usize,
}
```

---

## Message Queue Integration

### Kafka Producer/Consumer

Integrate WRAITH with Kafka for event streaming.

**Kafka producer:**
```rust
use rdkafka::producer::{FutureProducer, FutureRecord};

async fn publish_file_event(
    producer: &FutureProducer,
    file_hash: &Blake3Hash,
) -> Result<()> {
    let payload = serde_json::json!({
        "file_hash": file_hash.to_hex(),
        "timestamp": chrono::Utc::now(),
    }).to_string();

    producer.send(
        FutureRecord::to("wraith-files")
            .payload(&payload)
            .key(&file_hash.to_hex()),
        Duration::from_secs(5),
    ).await?;

    Ok(())
}
```

### RabbitMQ

Integrate with RabbitMQ for task queuing.

**RabbitMQ consumer:**
```rust
use lapin::{Channel, Consumer, options::*, types::FieldTable};

async fn consume_transfer_tasks(channel: Channel) -> Result<()> {
    let consumer: Consumer = channel
        .basic_consume(
            "wraith-transfers",
            "wraith-worker",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery?;
        let task: TransferTask = serde_json::from_slice(&delivery.data)?;

        // Process transfer
        process_transfer(task).await?;

        delivery.ack(BasicAckOptions::default()).await?;
    }

    Ok(())
}
```

---

## Logging & Monitoring

### OpenTelemetry

Export WRAITH metrics to OpenTelemetry.

**Telemetry integration:**
```rust
use opentelemetry::{global, trace::Tracer};
use tracing_opentelemetry::OpenTelemetryLayer;

fn setup_telemetry() {
    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name("wraith-protocol")
        .install_simple()
        .unwrap();

    let telemetry = OpenTelemetryLayer::new(tracer);

    tracing_subscriber::registry()
        .with(telemetry)
        .init();
}

// Trace file transfers
#[tracing::instrument]
async fn transfer_file(file_hash: Blake3Hash) -> Result<()> {
    // Transfer tracked automatically
    Ok(())
}
```

### Prometheus Metrics

Expose WRAITH metrics in Prometheus format.

**Metrics exporter:**
```rust
use prometheus::{Encoder, TextEncoder, Counter, Histogram};

lazy_static! {
    static ref TRANSFERS_TOTAL: Counter = register_counter!(
        "wraith_transfers_total",
        "Total number of file transfers"
    ).unwrap();

    static ref TRANSFER_DURATION: Histogram = register_histogram!(
        "wraith_transfer_duration_seconds",
        "File transfer duration"
    ).unwrap();
}

async fn metrics_handler() -> String {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}
```

---

## Client Compatibility

### CLI Compatibility

WRAITH CLI is compatible with standard Unix tools via stdin/stdout.

**Pipe integration:**
```bash
# Compress and send
tar czf - /path/to/data | wraith-cli send --peer 192.0.2.10:41641

# Receive and extract
wraith-cli recv --output - | tar xzf -

# Chain with other tools
wraith-cli recv | jq '.field' | wraith-cli send --peer <next-peer>
```

### Library Compatibility

WRAITH provides FFI bindings for non-Rust languages.

**C/C++ Integration:**
```c
#include "wraith.h"

int main() {
    wraith_session_t *session = wraith_connect("192.0.2.10:41641");
    if (!session) {
        fprintf(stderr, "Connection failed\n");
        return 1;
    }

    int result = wraith_send_file(session, "/path/to/file.bin");
    wraith_close(session);

    return result;
}
```

**Python Integration:**
```python
from wraith import Session, FileTransfer

session = Session.connect("192.0.2.10:41641")
transfer = FileTransfer(session)
transfer.send_file("/path/to/file.bin")
```

---

## Migration Strategies

### From BitTorrent

**Convert .torrent to WRAITH:**
```rust
async fn migrate_torrent(torrent_file: &Path) -> Result<Blake3Hash> {
    // Parse .torrent file
    let torrent = parse_torrent(torrent_file)?;

    // Download via BitTorrent
    let file_data = download_via_bittorrent(&torrent).await?;

    // Compute WRAITH hash
    let file_hash = Blake3Hash::hash(&file_data);

    // Announce to WRAITH DHT
    dht.announce(&group_secret, &file_hash, endpoints).await?;

    Ok(file_hash)
}
```

### From IPFS

**IPFS CID to WRAITH hash:**
```rust
async fn migrate_from_ipfs(cid: &str) -> Result<Blake3Hash> {
    // Download from IPFS
    let client = IpfsClient::default();
    let file_data = client.cat(cid).await?;

    // Store in WRAITH
    let file_hash = Blake3Hash::hash(&file_data);
    dht.announce(&group_secret, &file_hash, endpoints).await?;

    Ok(file_hash)
}
```

---

## See Also

- [Embedding Guide](embedding-guide.md)
- [Platform Support](platform-support.md)
- [API Reference](../engineering/api-reference.md)
