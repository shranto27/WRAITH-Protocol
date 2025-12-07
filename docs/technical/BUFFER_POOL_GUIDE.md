# Buffer Pool Integration Guide

**Version:** 1.2.1
**Date:** 2025-12-07
**Applies to:** wraith-transport, wraith-files, wraith-core

---

## Overview

The WRAITH Protocol uses a lock-free buffer pool for efficient packet receive operations. This guide documents when and how to use buffer pools, performance tuning, and monitoring best practices.

## Architecture

### Lock-Free Design

The `BufferPool` uses `crossbeam_queue::ArrayQueue` for zero-contention concurrent access:

```rust
pub struct BufferPool {
    pool: Arc<ArrayQueue<Vec<u8>>>,
    buffer_size: usize,
}
```

**Key Features:**
- Pre-allocated fixed-size buffers
- Lock-free acquire/release operations (O(1))
- Automatic fallback allocation when pool exhausted
- Security-conscious buffer clearing on release
- Thread-safe via Arc sharing

### Integration Points

```
                    ┌─────────────────────────────────┐
                    │         BufferPool              │
                    │   (crossbeam_queue::ArrayQueue) │
                    └──────────┬──────────────────────┘
                               │
        ┌──────────────────────┼──────────────────────┐
        │                      │                      │
        ▼                      ▼                      ▼
┌───────────────┐    ┌─────────────────┐    ┌──────────────────┐
│  WorkerPool   │    │   AF_XDP UMEM   │    │   File Chunker   │
│  (transport)  │    │   (zero-copy)   │    │     (files)      │
└───────────────┘    └─────────────────┘    └──────────────────┘
```

## When to Use Buffer Pools

### Use Buffer Pools When:

1. **High-throughput packet receive loops** (>10K packets/second)
2. **Worker thread pools** processing network I/O
3. **Predictable buffer sizes** (MTU-sized packets, chunks)
4. **Multi-threaded environments** requiring concurrent buffer access
5. **Low-latency requirements** where allocation overhead matters

### Don't Use Buffer Pools When:

1. **Variable-size allocations** (use standard allocation)
2. **One-time or infrequent operations** (overhead not justified)
3. **Memory-constrained environments** (pre-allocation may be wasteful)
4. **Debugging** (standard allocators have better debugging support)

## Usage Patterns

### Basic Usage

```rust
use wraith_transport::BufferPool;

// Create pool: 1500-byte buffers (MTU), 256 pre-allocated
let pool = BufferPool::new(1500, 256);

// Acquire buffer for packet receive
let mut buffer = pool.acquire();
// ... recv_from(&mut buffer) ...

// Return buffer to pool
pool.release(buffer);
```

### Worker Pool Integration

```rust
use wraith_transport::worker::{WorkerPool, WorkerConfig};

// Create worker pool with integrated buffer pool
let config = WorkerConfig::with_buffer_pool(1500, 1024);
let pool = WorkerPool::new(config);

// Acquire via worker pool interface
let buffer = pool.acquire_buffer(1500);
// ... use buffer ...
pool.release_buffer(buffer);
```

### Shared Pool Across Threads

```rust
use std::sync::Arc;
use wraith_transport::BufferPool;
use std::thread;

let pool = Arc::new(BufferPool::new(1500, 256));

let handles: Vec<_> = (0..4).map(|_| {
    let pool = Arc::clone(&pool);
    thread::spawn(move || {
        for _ in 0..1000 {
            let buffer = pool.acquire();
            // ... process packet ...
            pool.release(buffer);
        }
    })
}).collect();

for h in handles {
    h.join().unwrap();
}
```

## Sizing Guidelines

### Buffer Size Selection

| Use Case | Recommended Buffer Size | Notes |
|----------|------------------------|-------|
| UDP packets | 1500 (MTU) | Standard Ethernet |
| Jumbo frames | 9000 | Requires NIC support |
| AF_XDP | 2048 | UMEM frame size |
| File chunks | 262144 (256 KiB) | Default chunk size |
| Protocol frames | 9000 | Max WRAITH frame |

### Pool Size Selection

| Throughput | Pool Size | Memory |
|------------|-----------|--------|
| <1K pps | 64-128 | ~96-192 KB |
| 1K-10K pps | 256-512 | ~384-768 KB |
| 10K-100K pps | 1024-2048 | ~1.5-3 MB |
| >100K pps | 4096+ | ~6+ MB |

**Formula:**
```
pool_size = (packets_per_second * avg_processing_time_ms) / 1000 * 2
```

The 2x multiplier provides headroom for burst traffic.

## Performance Tuning

### Monitoring Pool Utilization

```rust
let pool = BufferPool::new(1500, 256);

// Check pool status
let available = pool.available();
let capacity = pool.capacity();
let utilization = 1.0 - (available as f64 / capacity as f64);

if utilization > 0.8 {
    tracing::warn!(
        "Buffer pool high utilization: {:.1}%",
        utilization * 100.0
    );
}
```

### Detecting Pool Exhaustion

Pool exhaustion occurs when all pre-allocated buffers are in use, triggering fallback allocation:

```rust
// Monitor for pool exhaustion
let initial_available = pool.available();

// After processing
if pool.available() == 0 {
    // Pool exhausted - consider increasing pool_size
    metrics.record_pool_exhaustion();
}
```

### Performance Metrics

Expected performance improvements from buffer pool usage:

| Metric | Improvement | Notes |
|--------|-------------|-------|
| Allocations/sec | -100K+ | Eliminated in hot path |
| GC pressure | -80%+ | Less heap fragmentation |
| Packet receive latency | -20-30% | O(1) vs O(n) allocation |
| Lock contention | Zero | Lock-free design |

## Security Considerations

### Buffer Clearing

Buffers are cleared on release to prevent information leakage:

```rust
pub fn release(&self, mut buffer: Vec<u8>) {
    // Clear buffer content for security
    buffer.clear();
    buffer.resize(self.buffer_size, 0);

    let _ = self.pool.push(buffer);
}
```

This ensures:
- Previous packet data is zeroed
- No information leakage between sessions
- Defense against memory inspection attacks

### Memory Locking (AF_XDP)

For AF_XDP UMEM, buffers are memory-locked to prevent swapping:

```rust
// UMEM creation with mlock
let ret = unsafe { libc::mlock(buffer, config.size) };
```

## Benchmarking

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench --workspace

# Run specific transport benchmarks
cargo bench -p wraith-transport

# Benchmark with specific features
cargo bench --features simd
```

### SIMD Benchmarking Methodology

To validate the ~2-3x SIMD speedup claim:

1. **Build without SIMD:**
   ```bash
   cargo bench -p wraith-core --no-default-features
   ```

2. **Build with SIMD:**
   ```bash
   cargo bench -p wraith-core --features simd
   ```

3. **Compare results:**
   ```
   frame_parsing/scalar: [X.XX µs Y.YY µs Z.ZZ µs]
   frame_parsing/simd:   [A.AA µs B.BB µs C.CC µs]

   Speedup = Y.YY / B.BB
   ```

### Platform-Specific Testing

| Platform | SIMD Support | Expected Speedup |
|----------|--------------|------------------|
| x86_64 (SSE2) | Always | 1.5-2x |
| x86_64 (AVX2) | If available | 2-3x |
| aarch64 (NEON) | Always | 1.5-2x |

## Troubleshooting

### Common Issues

#### Pool Exhaustion Under Load

**Symptom:** Fallback allocations, increased latency under load

**Solution:** Increase `pool_size` or investigate slow buffer release

```rust
// Monitor exhaustion rate
let exhaustion_rate = metrics.exhaustions / metrics.total_acquires;
if exhaustion_rate > 0.01 {  // >1%
    // Consider increasing pool_size
}
```

#### Memory Growth

**Symptom:** Increasing memory usage over time

**Cause:** Buffers not being returned to pool

**Solution:** Ensure all code paths call `release()`:

```rust
// Use RAII pattern for automatic release
struct BufferGuard<'a> {
    pool: &'a BufferPool,
    buffer: Option<Vec<u8>>,
}

impl Drop for BufferGuard<'_> {
    fn drop(&mut self) {
        if let Some(buffer) = self.buffer.take() {
            self.pool.release(buffer);
        }
    }
}
```

#### Wrong Buffer Size

**Symptom:** Buffers too small for incoming packets

**Solution:** Match buffer size to expected packet size:

```rust
// For UDP: MTU - IP header - UDP header
let buffer_size = 1500 - 20 - 8;  // 1472 bytes

// For WRAITH protocol frames
let buffer_size = 9000;  // Max frame size
```

## Related Documentation

- [AF_XDP Overview](../xdp/overview.md) - Zero-copy packet I/O
- [io_uring Integration](../xdp/io_uring.md) - Async file I/O
- [Performance Report](../PERFORMANCE_REPORT.md) - Benchmark results
- [Transport Architecture](../xdp/architecture.md) - Network layer design

---

*Last updated: 2025-12-07*
