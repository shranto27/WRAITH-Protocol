# WRAITH Protocol Coding Standards

**Document Version:** 1.0.0
**Last Updated:** 2025-11-28
**Status:** Engineering Documentation

---

## Overview

This document defines coding standards, style guidelines, and best practices for the WRAITH Protocol codebase. Adherence to these standards ensures consistency, maintainability, and security across all components.

**Key Principles:**
- **Safety First:** Prefer safe Rust; justify all `unsafe` code
- **Performance Matters:** Optimize hot paths; document trade-offs
- **Privacy by Default:** Constant-time operations; zeroization
- **Clear Intent:** Self-documenting code; meaningful names
- **Test Coverage:** Unit tests for logic; integration tests for workflows

---

## Rust Style Guide

### Code Formatting

**Use `rustfmt` for all formatting:**
```bash
# Format entire workspace
cargo fmt

# Check formatting in CI
cargo fmt -- --check
```

**Configuration (rustfmt.toml):**
```toml
max_width = 100
hard_tabs = false
tab_spaces = 4
newline_style = "Unix"
use_small_heuristics = "Default"
reorder_imports = true
reorder_modules = true
remove_nested_parens = true
edition = "2021"
```

### Naming Conventions

**General Rules:**
- Use `snake_case` for variables, functions, modules
- Use `PascalCase` for types, traits, enums
- Use `SCREAMING_SNAKE_CASE` for constants, statics
- Prefix private items with `_` if intentionally unused

**Examples:**
```rust
// Constants
const MAX_PACKET_SIZE: usize = 1500;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

// Types
struct SessionState { /* ... */ }
enum FrameType { /* ... */ }
trait EncryptionProvider { /* ... */ }

// Functions and variables
fn establish_connection(peer_addr: SocketAddr) -> Result<Session> { /* ... */ }
let connection_timeout = Duration::from_secs(10);

// Modules
mod noise_handshake;
mod dht_routing;

// Intentionally unused
let _guard = lock.lock();  // Held for scope, not accessed
```

**Protocol-Specific Names:**
```rust
// Frame types: PascalCase with descriptive names
enum FrameType {
    Handshake,
    Data,
    PathChallenge,
    PathResponse,
    Ack,
}

// Cryptographic primitives: Full algorithm name
fn xchacha20_poly1305_encrypt(/* ... */) { }
fn blake3_hash(/* ... */) { }
fn elligator2_encode(/* ... */) { }

// Network components: Descriptive, not abbreviated
struct DhtNode { /* ... */ }  // Not "DNode"
struct RelayConnection { /* ... */ }  // Not "RelayConn"
```

### Documentation

**Every public item must have documentation:**
```rust
/// Establishes a Noise_XX handshake with a remote peer.
///
/// This function performs a three-message handshake pattern:
/// 1. Initiator → Responder: ephemeral key
/// 2. Responder → Initiator: ephemeral + static keys
/// 3. Initiator → Responder: static key
///
/// # Arguments
///
/// * `peer_pubkey` - The peer's long-term Ed25519 public key (if known)
/// * `socket` - The UDP socket for communication
///
/// # Returns
///
/// A `NoiseSession` containing symmetric encryption keys, or an error
/// if the handshake fails.
///
/// # Errors
///
/// - `HandshakeError::Timeout` if peer doesn't respond within 10 seconds
/// - `HandshakeError::InvalidKey` if peer's static key is invalid
/// - `HandshakeError::DecryptionFailed` if message authentication fails
///
/// # Security
///
/// This handshake provides mutual authentication and forward secrecy.
/// The static keys are encrypted after the first message, preventing
/// passive eavesdropping from learning peer identities.
///
/// # Example
///
/// ```no_run
/// # use wraith_core::noise_handshake;
/// # use std::net::UdpSocket;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let socket = UdpSocket::bind("0.0.0.0:0")?;
/// let session = noise_handshake(None, socket).await?;
/// println!("Handshake complete, session keys established");
/// # Ok(())
/// # }
/// ```
pub async fn noise_handshake(
    peer_pubkey: Option<PublicKey>,
    socket: UdpSocket,
) -> Result<NoiseSession, HandshakeError> {
    // Implementation...
}
```

**Module-level documentation:**
```rust
//! Noise protocol handshake implementation.
//!
//! This module implements the Noise_XX handshake pattern for establishing
//! secure sessions between peers. The handshake provides:
//!
//! - **Mutual Authentication:** Both peers verify each other's identity
//! - **Forward Secrecy:** Compromise of long-term keys doesn't decrypt past sessions
//! - **Identity Hiding:** Static keys encrypted after first message
//!
//! # Protocol Flow
//!
//! ```text
//! Initiator                    Responder
//!    |                            |
//!    |--- e ---------------------->|  Message 1: ephemeral key
//!    |<-- e, ee, s, es ------------|  Message 2: ephemeral + static
//!    |--- s, se ------------------->|  Message 3: static key
//!    |                            |
//!    [Established: both have symmetric keys]
//! ```
//!
//! # Security Considerations
//!
//! - All operations are constant-time to prevent timing side-channels
//! - Keys are zeroized on drop to prevent memory leakage
//! - Replay protection via monotonic counters
//!
//! # Performance
//!
//! - Handshake latency: ~1.5 RTT (three messages)
//! - CPU cost: ~0.5 ms on modern hardware
//! - Memory: ~1 KB per session
```

### Error Handling

**Use `Result<T, E>` for all fallible operations:**
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("handshake timeout after {0:?}")]
    HandshakeTimeout(Duration),

    #[error("invalid frame type: expected {expected}, got {actual}")]
    InvalidFrameType { expected: u8, actual: u8 },

    #[error("decryption failed")]
    DecryptionFailed,

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

// Usage
fn decrypt_frame(frame: &[u8]) -> Result<Vec<u8>, SessionError> {
    // ... decryption logic ...
    Ok(plaintext)
}
```

**Never use `unwrap()` in production code:**
```rust
// BAD: Panics on error
let data = file.read_to_end(&mut buffer).unwrap();

// GOOD: Handle error properly
let data = file.read_to_end(&mut buffer)
    .map_err(|e| SessionError::Io(e))?;

// ACCEPTABLE: Only in tests or when invariant is guaranteed
let config = CONFIG.get().expect("config must be initialized in main()");
```

**Use `expect()` with descriptive messages for invariants:**
```rust
// BAD: Unclear why this can't fail
let session = sessions.get(&peer_id).expect("failed");

// GOOD: Clear invariant
let session = sessions.get(&peer_id)
    .expect("session must exist after successful handshake");
```

### Type Safety

**Use newtype pattern for domain-specific values:**
```rust
// BAD: Easy to confuse different u64 values
fn set_session_timeout(session_id: u64, timeout: u64) { }

// GOOD: Type-safe, impossible to mix up
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionId(u64);

#[derive(Debug, Clone, Copy)]
pub struct Timeout(Duration);

fn set_session_timeout(session_id: SessionId, timeout: Timeout) { }
```

**Use enums for state machines:**
```rust
pub enum SessionState {
    Idle,
    Handshaking {
        started_at: Instant,
        message_count: u8,
    },
    Established {
        keys: SymmetricKeys,
        last_activity: Instant,
    },
    Terminated {
        reason: TerminationReason,
    },
}

impl SessionState {
    pub fn transition(&mut self, event: SessionEvent) -> Result<()> {
        match (self, event) {
            (SessionState::Idle, SessionEvent::InitiateHandshake) => {
                *self = SessionState::Handshaking {
                    started_at: Instant::now(),
                    message_count: 0,
                };
                Ok(())
            }
            // ... other transitions ...
            (current, event) => {
                Err(SessionError::InvalidTransition {
                    from: current.name(),
                    event: event.name(),
                })
            }
        }
    }
}
```

---

## Security Coding Practices

### Constant-Time Operations

**Use constant-time comparisons for secrets:**
```rust
use subtle::ConstantTimeEq;

// BAD: Timing leak
if auth_tag == computed_tag {
    // ...
}

// GOOD: Constant-time comparison
use subtle::ConstantTimeEq;
if auth_tag.ct_eq(&computed_tag).into() {
    // ...
}
```

**Avoid conditional branches on secret data:**
```rust
// BAD: Branch depends on secret bit
fn conditional_negate(x: u32, condition: bool) -> u32 {
    if condition {
        x.wrapping_neg()
    } else {
        x
    }
}

// GOOD: Constant-time using bitwise operations
fn conditional_negate(x: u32, condition: bool) -> u32 {
    let mask = (condition as u32).wrapping_neg();  // 0x00000000 or 0xFFFFFFFF
    (x ^ mask).wrapping_add(mask & 1)
}
```

### Zeroization

**Zero sensitive data on drop:**
```rust
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SymmetricKeys {
    send_key: [u8; 32],
    recv_key: [u8; 32],
    nonce_send: u64,
    nonce_recv: u64,
}

// Keys automatically zeroized when dropped
```

**Manually zeroize when needed:**
```rust
fn derive_keys(shared_secret: &[u8; 32]) -> SymmetricKeys {
    let mut ikm = *shared_secret;  // Copy for HKDF
    let keys = hkdf_extract_expand(&ikm);

    // Zeroize intermediate key material
    ikm.zeroize();

    keys
}
```

### Input Validation

**Validate all external input:**
```rust
pub fn parse_frame(data: &[u8]) -> Result<Frame, FrameError> {
    // Validate minimum size
    if data.len() < FRAME_HEADER_SIZE {
        return Err(FrameError::TooShort {
            expected: FRAME_HEADER_SIZE,
            actual: data.len(),
        });
    }

    // Validate frame type
    let frame_type = data[0];
    if frame_type > MAX_FRAME_TYPE {
        return Err(FrameError::InvalidType(frame_type));
    }

    // Validate length field
    let length = u16::from_le_bytes([data[1], data[2]]) as usize;
    if length > MAX_FRAME_PAYLOAD {
        return Err(FrameError::PayloadTooLarge {
            max: MAX_FRAME_PAYLOAD,
            actual: length,
        });
    }

    // ... parse rest of frame ...
}
```

### Safe Unsafe Code

**Minimize `unsafe` usage; document invariants:**
```rust
/// # Safety
///
/// The caller must ensure:
/// 1. `ptr` is valid for reads of `len` bytes
/// 2. `ptr` is properly aligned for `T`
/// 3. The memory `ptr` points to is initialized
/// 4. No mutable references to the same memory exist
unsafe fn read_unaligned<T: Copy>(ptr: *const u8, len: usize) -> T {
    debug_assert!(len == std::mem::size_of::<T>());
    debug_assert!(!ptr.is_null());

    std::ptr::read_unaligned(ptr as *const T)
}
```

**Encapsulate unsafe in safe abstractions:**
```rust
// GOOD: Unsafe contained in safe wrapper
pub struct AlignedBuffer {
    ptr: *mut u8,
    len: usize,
    capacity: usize,
}

impl AlignedBuffer {
    pub fn new(capacity: usize) -> Self {
        // Unsafe allocation encapsulated
        let layout = Layout::from_size_align(capacity, 4096)
            .expect("invalid layout");
        let ptr = unsafe { std::alloc::alloc(layout) };

        Self {
            ptr,
            len: 0,
            capacity,
        }
    }

    // Safe public interface
    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        if self.len + data.len() > self.capacity {
            return Err(BufferError::InsufficientSpace);
        }

        // Unsafe contained here
        unsafe {
            std::ptr::copy_nonoverlapping(
                data.as_ptr(),
                self.ptr.add(self.len),
                data.len(),
            );
        }
        self.len += data.len();
        Ok(())
    }
}

unsafe impl Send for AlignedBuffer {}
unsafe impl Sync for AlignedBuffer {}

impl Drop for AlignedBuffer {
    fn drop(&mut self) {
        unsafe {
            let layout = Layout::from_size_align_unchecked(self.capacity, 4096);
            std::alloc::dealloc(self.ptr, layout);
        }
    }
}
```

---

## Performance Guidelines

### Hot Path Optimization

**Profile before optimizing:**
```bash
# Generate flamegraph
cargo flamegraph --bin wraith-cli -- transfer large_file.bin

# CPU profiling
perf record -g target/release/wraith-cli transfer large_file.bin
perf report
```

**Inline critical functions:**
```rust
// Cold path: Don't inline
pub fn initialize_logging() { /* ... */ }

// Hot path: Inline hint
#[inline]
pub fn encrypt_chunk(key: &[u8; 32], data: &[u8]) -> Vec<u8> {
    // ... fast path encryption ...
}

// Very hot path: Force inline
#[inline(always)]
pub fn xor_block(a: &mut [u8; 16], b: &[u8; 16]) {
    for i in 0..16 {
        a[i] ^= b[i];
    }
}
```

**Use SIMD when beneficial:**
```rust
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn xor_blocks_simd(a: &mut [u8], b: &[u8]) {
    assert_eq!(a.len(), b.len());
    assert_eq!(a.len() % 32, 0);

    for i in (0..a.len()).step_by(32) {
        let va = _mm256_loadu_si256(a.as_ptr().add(i) as *const __m256i);
        let vb = _mm256_loadu_si256(b.as_ptr().add(i) as *const __m256i);
        let vr = _mm256_xor_si256(va, vb);
        _mm256_storeu_si256(a.as_mut_ptr().add(i) as *mut __m256i, vr);
    }
}
```

### Memory Management

**Preallocate when size is known:**
```rust
// BAD: Repeated allocations
let mut buffer = Vec::new();
for chunk in chunks {
    buffer.extend_from_slice(chunk);
}

// GOOD: Preallocate
let total_size: usize = chunks.iter().map(|c| c.len()).sum();
let mut buffer = Vec::with_capacity(total_size);
for chunk in chunks {
    buffer.extend_from_slice(chunk);
}
```

**Reuse allocations:**
```rust
pub struct FrameProcessor {
    decrypt_buffer: Vec<u8>,
}

impl FrameProcessor {
    pub fn process_frame(&mut self, frame: &[u8]) -> Result<&[u8]> {
        self.decrypt_buffer.clear();  // Reuse allocation
        self.decrypt_buffer.resize(frame.len(), 0);

        decrypt_into(frame, &mut self.decrypt_buffer)?;
        Ok(&self.decrypt_buffer)
    }
}
```

### Async/Await Guidelines

**Use `async` for I/O-bound operations:**
```rust
// I/O-bound: Use async
pub async fn send_frame(socket: &UdpSocket, frame: &[u8]) -> Result<()> {
    socket.send(frame).await?;
    Ok(())
}

// CPU-bound: Use blocking + spawn_blocking
pub async fn compute_file_hash(path: &Path) -> Result<Blake3Hash> {
    let path = path.to_owned();
    tokio::task::spawn_blocking(move || {
        let mut hasher = blake3::Hasher::new();
        let mut file = File::open(&path)?;
        std::io::copy(&mut file, &mut hasher)?;
        Ok(Blake3Hash(hasher.finalize().into()))
    }).await?
}
```

**Avoid `.await` in tight loops:**
```rust
// BAD: Await in loop
for chunk in chunks {
    send_chunk(socket, chunk).await?;
}

// GOOD: Batch operations
use futures::stream::{self, StreamExt};

stream::iter(chunks)
    .for_each_concurrent(10, |chunk| async move {
        send_chunk(socket, chunk).await.unwrap();
    })
    .await;
```

---

## Testing Standards

### Unit Tests

**Test module structure:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_parsing_valid() {
        let frame_data = vec![0x01, 0x00, 0x04, 0xAA, 0xBB, 0xCC, 0xDD];
        let frame = parse_frame(&frame_data).unwrap();

        assert_eq!(frame.frame_type, FrameType::Data);
        assert_eq!(frame.payload, &[0xAA, 0xBB, 0xCC, 0xDD]);
    }

    #[test]
    fn test_frame_parsing_too_short() {
        let frame_data = vec![0x01];
        let result = parse_frame(&frame_data);

        assert!(matches!(result, Err(FrameError::TooShort { .. })));
    }

    #[test]
    fn test_frame_parsing_invalid_type() {
        let frame_data = vec![0xFF, 0x00, 0x00];
        let result = parse_frame(&frame_data);

        assert!(matches!(result, Err(FrameError::InvalidType(0xFF))));
    }
}
```

**Property-based testing:**
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_encryption_roundtrip(
        plaintext in prop::collection::vec(any::<u8>(), 0..10000)
    ) {
        let key = [0u8; 32];  // Test key
        let ciphertext = encrypt(&key, &plaintext);
        let decrypted = decrypt(&key, &ciphertext).unwrap();

        prop_assert_eq!(plaintext, decrypted);
    }
}
```

### Benchmarks

**Use `criterion` for benchmarks:**
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_blake3_hash(c: &mut Criterion) {
    let data = vec![0u8; 1024 * 1024];  // 1 MB

    c.bench_function("blake3_hash_1mb", |b| {
        b.iter(|| {
            let mut hasher = blake3::Hasher::new();
            hasher.update(black_box(&data));
            hasher.finalize()
        })
    });
}

criterion_group!(benches, bench_blake3_hash);
criterion_main!(benches);
```

---

## Code Review Checklist

**Before submitting PR:**
- [ ] Code compiles without warnings
- [ ] All tests pass (`cargo test`)
- [ ] Clippy passes (`cargo clippy -- -D warnings`)
- [ ] Code formatted (`cargo fmt`)
- [ ] Public items documented
- [ ] Unsafe code justified and documented
- [ ] Error handling complete
- [ ] No sensitive data in logs
- [ ] Performance benchmarks run (if relevant)

**Reviewer checklist:**
- [ ] Algorithm correctness
- [ ] Edge cases handled
- [ ] Error paths tested
- [ ] Security implications considered
- [ ] Performance impact acceptable
- [ ] API design coherent
- [ ] Documentation accurate and complete

---

## Commit Standards

**Commit message format:**
```
<type>(<scope>): <subject>

<body>

<footer>
```

**Examples:**
```
feat(crypto): implement Elligator2 encoding

Add constant-time Elligator2 encoding for Curve25519 points.
This enables protocol steganography by making public keys
indistinguishable from random.

Closes #42

fix(transport): handle connection reset gracefully

Previously, ECONNRESET would cause a panic. Now properly
clean up socket state and return error.

perf(files): use SIMD for chunk hashing

Use AVX2 BLAKE3 implementation for 3x speedup.

Before: 1.2 GB/s
After: 3.6 GB/s
```

---

## See Also

- [Development Guide](development-guide.md)
- [API Reference](api-reference.md)
- [Testing Strategy](../testing/testing-strategy.md)
- [Security Model](../architecture/security-model.md)
