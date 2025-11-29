# WRAITH Protocol API Reference

**Document Version:** 1.0.0
**Last Updated:** 2025-11-28
**Status:** Engineering Documentation

---

## Overview

This document provides a comprehensive reference for the WRAITH Protocol's public APIs. The protocol is implemented as a Rust workspace with multiple crates, each exposing specific functionality.

**Crate Organization:**
- **wraith-core:** Core session management, framing
- **wraith-crypto:** Cryptographic primitives
- **wraith-transport:** Network transport layer
- **wraith-discovery:** DHT and peer discovery
- **wraith-files:** File transfer operations
- **wraith-cli:** Command-line interface

---

## wraith-core

### Session Management

#### `Session`

Represents an established secure session with a peer.

```rust
pub struct Session {
    // Private fields
}

impl Session {
    /// Creates a new session after successful handshake.
    ///
    /// # Arguments
    ///
    /// * `keys` - Symmetric encryption keys derived from handshake
    /// * `peer_id` - Peer's long-term public key
    /// * `socket` - UDP socket for communication
    ///
    /// # Returns
    ///
    /// A new `Session` instance.
    pub fn new(
        keys: SymmetricKeys,
        peer_id: PublicKey,
        socket: Arc<UdpSocket>,
    ) -> Self;

    /// Sends a data frame to the peer.
    ///
    /// # Arguments
    ///
    /// * `payload` - Plaintext data to send (max 1200 bytes recommended)
    ///
    /// # Returns
    ///
    /// Ok(()) on success, or SessionError on failure.
    ///
    /// # Errors
    ///
    /// - `SessionError::PayloadTooLarge` if payload exceeds max size
    /// - `SessionError::EncryptionFailed` if encryption fails
    /// - `SessionError::NetworkError` if socket send fails
    pub async fn send(&mut self, payload: &[u8]) -> Result<(), SessionError>;

    /// Receives the next data frame from the peer.
    ///
    /// # Returns
    ///
    /// The decrypted payload bytes, or SessionError on failure.
    ///
    /// # Errors
    ///
    /// - `SessionError::Timeout` if no frame received within timeout
    /// - `SessionError::DecryptionFailed` if frame authentication fails
    /// - `SessionError::ReplayAttack` if nonce is reused
    pub async fn recv(&mut self) -> Result<Vec<u8>, SessionError>;

    /// Closes the session gracefully.
    ///
    /// Sends a TERMINATE frame to the peer and cleans up resources.
    ///
    /// # Returns
    ///
    /// Ok(()) if session closed successfully.
    pub async fn close(self) -> Result<(), SessionError>;

    /// Returns the peer's public key.
    pub fn peer_id(&self) -> &PublicKey;

    /// Returns session statistics.
    pub fn stats(&self) -> SessionStats;
}
```

#### `SessionStats`

Statistics about an active session.

```rust
#[derive(Debug, Clone)]
pub struct SessionStats {
    /// Bytes sent
    pub bytes_sent: u64,
    /// Bytes received
    pub bytes_received: u64,
    /// Packets sent
    pub packets_sent: u64,
    /// Packets received
    pub packets_received: u64,
    /// Round-trip time estimate
    pub rtt: Duration,
    /// Session establishment time
    pub established_at: Instant,
}
```

### Frame Types

#### `Frame`

Represents a protocol frame.

```rust
#[derive(Debug, Clone)]
pub struct Frame {
    pub frame_type: FrameType,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameType {
    Handshake = 0x00,
    Data = 0x01,
    Ack = 0x02,
    PathChallenge = 0x03,
    PathResponse = 0x04,
    Terminate = 0x05,
}

impl Frame {
    /// Parses a frame from raw bytes.
    ///
    /// # Arguments
    ///
    /// * `data` - Raw frame bytes (minimum 3 bytes)
    ///
    /// # Returns
    ///
    /// Parsed `Frame` or `FrameError`.
    ///
    /// # Errors
    ///
    /// - `FrameError::TooShort` if data < 3 bytes
    /// - `FrameError::InvalidType` if frame type is unknown
    /// - `FrameError::InvalidLength` if length field doesn't match data
    pub fn parse(data: &[u8]) -> Result<Self, FrameError>;

    /// Serializes frame to bytes.
    ///
    /// # Returns
    ///
    /// Frame encoded as bytes (3-byte header + payload).
    pub fn to_bytes(&self) -> Vec<u8>;
}
```

### Error Types

```rust
#[derive(Debug, Error)]
pub enum SessionError {
    #[error("handshake timeout after {0:?}")]
    HandshakeTimeout(Duration),

    #[error("payload too large: {size} bytes (max {max})")]
    PayloadTooLarge { size: usize, max: usize },

    #[error("decryption failed")]
    DecryptionFailed,

    #[error("replay attack detected")]
    ReplayAttack,

    #[error("network error: {0}")]
    NetworkError(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum FrameError {
    #[error("frame too short: expected {expected}, got {actual}")]
    TooShort { expected: usize, actual: usize },

    #[error("invalid frame type: {0}")]
    InvalidType(u8),

    #[error("invalid length field")]
    InvalidLength,
}
```

---

## wraith-crypto

### Noise Protocol

#### `NoiseHandshake`

Performs Noise_XX handshake pattern.

```rust
pub struct NoiseHandshake {
    // Private fields
}

impl NoiseHandshake {
    /// Creates a new handshake initiator.
    ///
    /// # Arguments
    ///
    /// * `static_keypair` - Local long-term Ed25519 keypair
    ///
    /// # Returns
    ///
    /// A new `NoiseHandshake` in initiator mode.
    pub fn new_initiator(static_keypair: Keypair) -> Self;

    /// Creates a new handshake responder.
    ///
    /// # Arguments
    ///
    /// * `static_keypair` - Local long-term Ed25519 keypair
    ///
    /// # Returns
    ///
    /// A new `NoiseHandshake` in responder mode.
    pub fn new_responder(static_keypair: Keypair) -> Self;

    /// Generates the first handshake message (initiator only).
    ///
    /// # Returns
    ///
    /// Handshake message 1 (ephemeral key, ~32 bytes).
    pub fn write_message_1(&mut self) -> Result<Vec<u8>, NoiseError>;

    /// Processes handshake message 1 (responder only).
    ///
    /// # Arguments
    ///
    /// * `message` - Message 1 from initiator
    ///
    /// # Returns
    ///
    /// Ok(()) if message is valid.
    pub fn read_message_1(&mut self, message: &[u8]) -> Result<(), NoiseError>;

    /// Generates handshake message 2 (responder only).
    ///
    /// # Returns
    ///
    /// Handshake message 2 (ephemeral + static keys, ~96 bytes).
    pub fn write_message_2(&mut self) -> Result<Vec<u8>, NoiseError>;

    /// Processes handshake message 2 (initiator only).
    ///
    /// # Arguments
    ///
    /// * `message` - Message 2 from responder
    ///
    /// # Returns
    ///
    /// Ok(()) if message is valid.
    pub fn read_message_2(&mut self, message: &[u8]) -> Result<(), NoiseError>;

    /// Generates handshake message 3 (initiator only).
    ///
    /// # Returns
    ///
    /// Handshake message 3 (static key, ~64 bytes).
    pub fn write_message_3(&mut self) -> Result<Vec<u8>, NoiseError>;

    /// Processes handshake message 3 (responder only).
    ///
    /// # Arguments
    ///
    /// * `message` - Message 3 from initiator
    ///
    /// # Returns
    ///
    /// Ok(()) if message is valid.
    pub fn read_message_3(&mut self, message: &[u8]) -> Result<(), NoiseError>;

    /// Finalizes handshake and derives session keys.
    ///
    /// # Returns
    ///
    /// Symmetric encryption keys for the session.
    pub fn finalize(self) -> Result<SymmetricKeys, NoiseError>;
}
```

### Symmetric Encryption

#### `SymmetricKeys`

Session encryption keys.

```rust
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SymmetricKeys {
    // Private fields (zeroized on drop)
}

impl SymmetricKeys {
    /// Encrypts plaintext using XChaCha20-Poly1305 AEAD.
    ///
    /// # Arguments
    ///
    /// * `plaintext` - Data to encrypt
    ///
    /// # Returns
    ///
    /// Ciphertext with 16-byte authentication tag appended.
    pub fn encrypt(&mut self, plaintext: &[u8]) -> Vec<u8>;

    /// Decrypts ciphertext using XChaCha20-Poly1305 AEAD.
    ///
    /// # Arguments
    ///
    /// * `ciphertext` - Encrypted data with auth tag
    ///
    /// # Returns
    ///
    /// Plaintext if authentication succeeds, or CryptoError.
    ///
    /// # Errors
    ///
    /// - `CryptoError::AuthenticationFailed` if tag is invalid
    /// - `CryptoError::ReplayAttack` if nonce is reused
    pub fn decrypt(&mut self, ciphertext: &[u8]) -> Result<Vec<u8>, CryptoError>;
}
```

### Hashing

#### `Blake3Hash`

BLAKE3 hash function.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Blake3Hash([u8; 32]);

impl Blake3Hash {
    /// Computes BLAKE3 hash of data.
    ///
    /// # Arguments
    ///
    /// * `data` - Data to hash
    ///
    /// # Returns
    ///
    /// 32-byte BLAKE3 hash.
    pub fn hash(data: &[u8]) -> Self;

    /// Computes keyed BLAKE3 hash (for HMAC-like uses).
    ///
    /// # Arguments
    ///
    /// * `key` - 32-byte key
    /// * `data` - Data to hash
    ///
    /// # Returns
    ///
    /// 32-byte keyed hash.
    pub fn keyed_hash(key: &[u8; 32], data: &[u8]) -> Self;

    /// Derives key material using BLAKE3 in KDF mode.
    ///
    /// # Arguments
    ///
    /// * `context` - KDF context string
    /// * `ikm` - Input key material
    /// * `length` - Output length in bytes
    ///
    /// # Returns
    ///
    /// Derived key material of specified length.
    pub fn derive_key(context: &str, ikm: &[u8], length: usize) -> Vec<u8>;

    /// Returns hash as byte array.
    pub fn as_bytes(&self) -> &[u8; 32];

    /// Returns hash as hex string.
    pub fn to_hex(&self) -> String;
}
```

### Key Generation

#### `Keypair`

Ed25519 keypair.

```rust
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct Keypair {
    // Private fields (zeroized on drop)
}

impl Keypair {
    /// Generates a new random Ed25519 keypair.
    ///
    /// # Returns
    ///
    /// A new `Keypair` with cryptographically secure random keys.
    pub fn generate() -> Self;

    /// Creates a keypair from a seed.
    ///
    /// # Arguments
    ///
    /// * `seed` - 32-byte seed
    ///
    /// # Returns
    ///
    /// Deterministic `Keypair` derived from seed.
    pub fn from_seed(seed: &[u8; 32]) -> Self;

    /// Returns the public key.
    pub fn public(&self) -> &PublicKey;

    /// Signs a message.
    ///
    /// # Arguments
    ///
    /// * `message` - Message to sign
    ///
    /// # Returns
    ///
    /// 64-byte Ed25519 signature.
    pub fn sign(&self, message: &[u8]) -> Signature;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PublicKey([u8; 32]);

impl PublicKey {
    /// Verifies a signature.
    ///
    /// # Arguments
    ///
    /// * `message` - Original message
    /// * `signature` - 64-byte signature
    ///
    /// # Returns
    ///
    /// true if signature is valid, false otherwise.
    pub fn verify(&self, message: &[u8], signature: &Signature) -> bool;

    /// Converts public key to bytes.
    pub fn as_bytes(&self) -> &[u8; 32];

    /// Creates public key from bytes.
    pub fn from_bytes(bytes: [u8; 32]) -> Self;
}
```

### Elligator2

#### `Elligator2`

Elligator2 encoding for Curve25519 points.

```rust
pub struct Elligator2;

impl Elligator2 {
    /// Encodes a Curve25519 public key to a random-looking representative.
    ///
    /// # Arguments
    ///
    /// * `pubkey` - Curve25519 public key
    ///
    /// # Returns
    ///
    /// 32-byte representative indistinguishable from random,
    /// or None if point is not encodable (~50% of points).
    pub fn encode(pubkey: &[u8; 32]) -> Option<[u8; 32]>;

    /// Decodes an Elligator2 representative to a Curve25519 public key.
    ///
    /// # Arguments
    ///
    /// * `representative` - 32-byte Elligator2 representative
    ///
    /// # Returns
    ///
    /// Curve25519 public key.
    pub fn decode(representative: &[u8; 32]) -> [u8; 32];
}
```

---

## wraith-transport

### UDP Transport

#### `UdpTransport`

UDP-based transport layer.

```rust
pub struct UdpTransport {
    // Private fields
}

impl UdpTransport {
    /// Creates a new UDP transport.
    ///
    /// # Arguments
    ///
    /// * `bind_addr` - Local address to bind (e.g., "0.0.0.0:41641")
    ///
    /// # Returns
    ///
    /// New `UdpTransport` instance.
    ///
    /// # Errors
    ///
    /// - `TransportError::BindFailed` if socket bind fails
    pub async fn bind(bind_addr: SocketAddr) -> Result<Self, TransportError>;

    /// Sends a packet.
    ///
    /// # Arguments
    ///
    /// * `dest` - Destination address
    /// * `data` - Packet data
    ///
    /// # Returns
    ///
    /// Number of bytes sent.
    pub async fn send_to(
        &self,
        dest: SocketAddr,
        data: &[u8],
    ) -> Result<usize, TransportError>;

    /// Receives a packet.
    ///
    /// # Returns
    ///
    /// (data, source address) tuple.
    pub async fn recv_from(&self) -> Result<(Vec<u8>, SocketAddr), TransportError>;

    /// Returns the local socket address.
    pub fn local_addr(&self) -> SocketAddr;
}
```

### AF_XDP Transport

#### `XdpTransport`

High-performance AF_XDP transport (Linux-only).

```rust
#[cfg(target_os = "linux")]
pub struct XdpTransport {
    // Private fields
}

#[cfg(target_os = "linux")]
impl XdpTransport {
    /// Creates a new AF_XDP transport.
    ///
    /// # Arguments
    ///
    /// * `interface` - Network interface name (e.g., "eth0")
    /// * `queue_id` - RX queue ID (typically 0)
    ///
    /// # Returns
    ///
    /// New `XdpTransport` instance.
    ///
    /// # Errors
    ///
    /// - `TransportError::PermissionDenied` if missing CAP_NET_RAW
    /// - `TransportError::InterfaceNotFound` if interface doesn't exist
    /// - `TransportError::XdpNotSupported` if driver doesn't support XDP
    ///
    /// # Safety
    ///
    /// Requires CAP_NET_RAW, CAP_NET_ADMIN, CAP_BPF capabilities.
    pub fn new(interface: &str, queue_id: u32) -> Result<Self, TransportError>;

    /// Sends a packet (zero-copy if possible).
    ///
    /// # Arguments
    ///
    /// * `data` - Packet data
    ///
    /// # Returns
    ///
    /// Ok(()) if packet queued successfully.
    pub fn send(&mut self, data: &[u8]) -> Result<(), TransportError>;

    /// Receives a packet (zero-copy).
    ///
    /// # Returns
    ///
    /// Reference to received packet (valid until next recv call).
    pub fn recv(&mut self) -> Result<&[u8], TransportError>;
}
```

---

## wraith-discovery

### DHT

#### `DhtNode`

Kademlia DHT node.

```rust
pub struct DhtNode {
    // Private fields
}

impl DhtNode {
    /// Creates a new DHT node.
    ///
    /// # Arguments
    ///
    /// * `config` - DHT configuration
    ///
    /// # Returns
    ///
    /// New `DhtNode` instance.
    pub fn new(config: DhtConfig) -> Self;

    /// Bootstraps DHT by connecting to known nodes.
    ///
    /// # Arguments
    ///
    /// * `bootstrap_nodes` - List of known DHT node addresses
    ///
    /// # Returns
    ///
    /// Ok(()) when bootstrap succeeds.
    pub async fn bootstrap(&mut self, bootstrap_nodes: &[SocketAddr]) -> Result<()>;

    /// Stores a value in the DHT.
    ///
    /// # Arguments
    ///
    /// * `key` - 20-byte DHT key
    /// * `value` - Value to store (max 1024 bytes)
    ///
    /// # Returns
    ///
    /// Ok(()) when stored on k nodes.
    pub async fn put(&mut self, key: &[u8; 20], value: Vec<u8>) -> Result<()>;

    /// Retrieves a value from the DHT.
    ///
    /// # Arguments
    ///
    /// * `key` - 20-byte DHT key
    ///
    /// # Returns
    ///
    /// Value if found, or None.
    pub async fn get(&mut self, key: &[u8; 20]) -> Result<Option<Vec<u8>>>;

    /// Announces file availability to the DHT.
    ///
    /// # Arguments
    ///
    /// * `group_secret` - Group secret key
    /// * `file_hash` - BLAKE3 file hash
    /// * `endpoints` - Peer endpoints (IP:port)
    ///
    /// # Returns
    ///
    /// Ok(()) when announcement succeeds.
    pub async fn announce(
        &mut self,
        group_secret: &[u8; 32],
        file_hash: &Blake3Hash,
        endpoints: Vec<SocketAddr>,
    ) -> Result<()>;

    /// Searches for file peers in the DHT.
    ///
    /// # Arguments
    ///
    /// * `group_secret` - Group secret key
    /// * `file_hash` - BLAKE3 file hash
    ///
    /// # Returns
    ///
    /// List of peer endpoints offering the file.
    pub async fn find_peers(
        &mut self,
        group_secret: &[u8; 32],
        file_hash: &Blake3Hash,
    ) -> Result<Vec<SocketAddr>>;
}

#[derive(Debug, Clone)]
pub struct DhtConfig {
    /// Replication factor (default: 20)
    pub k: usize,
    /// Concurrent lookup requests (default: 3)
    pub alpha: usize,
    /// Node ID
    pub node_id: [u8; 20],
}
```

---

## wraith-files

### File Transfer

#### `FileTransfer`

High-level file transfer API.

```rust
pub struct FileTransfer {
    // Private fields
}

impl FileTransfer {
    /// Creates a new file transfer session.
    ///
    /// # Arguments
    ///
    /// * `session` - Established WRAITH session
    /// * `config` - Transfer configuration
    ///
    /// # Returns
    ///
    /// New `FileTransfer` instance.
    pub fn new(session: Session, config: TransferConfig) -> Self;

    /// Sends a file to the peer.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to file to send
    /// * `progress` - Optional progress callback
    ///
    /// # Returns
    ///
    /// Ok(()) when transfer completes successfully.
    ///
    /// # Errors
    ///
    /// - `TransferError::FileNotFound` if file doesn't exist
    /// - `TransferError::PermissionDenied` if can't read file
    /// - `TransferError::ConnectionLost` if peer disconnects
    pub async fn send_file<P: AsRef<Path>, F>(
        &mut self,
        file_path: P,
        progress: Option<F>,
    ) -> Result<(), TransferError>
    where
        F: Fn(TransferProgress);

    /// Receives a file from the peer.
    ///
    /// # Arguments
    ///
    /// * `output_path` - Where to save received file
    /// * `progress` - Optional progress callback
    ///
    /// # Returns
    ///
    /// File metadata when transfer completes.
    ///
    /// # Errors
    ///
    /// - `TransferError::PermissionDenied` if can't write file
    /// - `TransferError::IntegrityCheckFailed` if hash doesn't match
    pub async fn recv_file<P: AsRef<Path>, F>(
        &mut self,
        output_path: P,
        progress: Option<F>,
    ) -> Result<FileMetadata, TransferError>
    where
        F: Fn(TransferProgress);
}

#[derive(Debug, Clone)]
pub struct TransferProgress {
    pub bytes_transferred: u64,
    pub total_bytes: u64,
    pub throughput: f64,  // bytes/sec
    pub eta: Option<Duration>,
}

#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub file_hash: Blake3Hash,
    pub size: u64,
    pub chunk_count: u32,
}
```

---

## Error Handling

All errors implement `std::error::Error` and derive from `thiserror::Error`.

**Common error handling pattern:**
```rust
use wraith_core::{Session, SessionError};

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    let session = Session::new(/* ... */);

    match session.send(b"hello").await {
        Ok(()) => println!("Sent successfully"),
        Err(SessionError::NetworkError(e)) => {
            eprintln!("Network error: {}", e);
            // Retry logic
        }
        Err(e) => return Err(e.into()),
    }

    Ok(())
}
```

---

## Feature Flags

**Available features:**
```toml
[features]
default = ["af-xdp", "io-uring"]

# Kernel bypass features (Linux-only)
af-xdp = ["libbpf-sys"]
io-uring = ["io-uring"]

# Network features
tls-relay = ["rustls", "tokio-rustls"]

# Experimental features
experimental = []
```

---

## See Also

- [Development Guide](development-guide.md)
- [Coding Standards](coding-standards.md)
- [Protocol Overview](../architecture/protocol-overview.md)
- [Security Model](../architecture/security-model.md)
