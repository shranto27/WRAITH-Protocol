//! Performance benchmarks for file transfer operations

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use std::io::Write;
use tempfile::NamedTempFile;
use wraith_files::DEFAULT_CHUNK_SIZE;
use wraith_files::chunker::{FileChunker, FileReassembler};
use wraith_files::tree_hash::{compute_tree_hash, compute_tree_hash_from_data};

/// Benchmark file chunking performance
fn bench_file_chunking(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_chunking");

    for size in [
        1_000_000,   // 1 MB
        10_000_000,  // 10 MB
        100_000_000, // 100 MB
    ] {
        group.throughput(Throughput::Bytes(size));

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            // Create temporary file
            let mut temp_file = NamedTempFile::new().unwrap();
            let data = vec![0xAA; size as usize];
            temp_file.write_all(&data).unwrap();
            temp_file.flush().unwrap();
            let path = temp_file.path().to_path_buf();

            b.iter(|| {
                let mut chunker = FileChunker::new(&path, DEFAULT_CHUNK_SIZE).unwrap();
                let mut total = 0;
                while let Some(chunk) = chunker.read_chunk().unwrap() {
                    total += black_box(chunk.len());
                }
                total
            });
        });
    }

    group.finish();
}

/// Benchmark tree hash computation
fn bench_tree_hashing(c: &mut Criterion) {
    let mut group = c.benchmark_group("tree_hashing");

    for size in [
        1_000_000,   // 1 MB
        10_000_000,  // 10 MB
        100_000_000, // 100 MB
    ] {
        group.throughput(Throughput::Bytes(size));

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            // Create temporary file
            let mut temp_file = NamedTempFile::new().unwrap();
            let data = vec![0xBB; size as usize];
            temp_file.write_all(&data).unwrap();
            temp_file.flush().unwrap();
            let path = temp_file.path().to_path_buf();

            b.iter(|| {
                let tree = compute_tree_hash(&path, DEFAULT_CHUNK_SIZE).unwrap();
                black_box(tree.root)
            });
        });
    }

    group.finish();
}

/// Benchmark in-memory tree hashing
fn bench_tree_hashing_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("tree_hashing_memory");

    for size in [
        1_000_000,   // 1 MB
        10_000_000,  // 10 MB
        100_000_000, // 100 MB
    ] {
        group.throughput(Throughput::Bytes(size));

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let data = vec![0xCC; size as usize];

            b.iter(|| {
                let tree = compute_tree_hash_from_data(&data, DEFAULT_CHUNK_SIZE);
                black_box(tree.root)
            });
        });
    }

    group.finish();
}

/// Benchmark chunk verification
fn bench_chunk_verification(c: &mut Criterion) {
    let mut group = c.benchmark_group("chunk_verification");

    // Create test data
    let chunk_data = vec![0xDD; DEFAULT_CHUNK_SIZE];
    let tree = compute_tree_hash_from_data(&chunk_data, DEFAULT_CHUNK_SIZE);

    group.throughput(Throughput::Bytes(DEFAULT_CHUNK_SIZE as u64));

    group.bench_function("verify_chunk", |b| {
        b.iter(|| {
            let result = tree.verify_chunk(0, &chunk_data);
            black_box(result)
        });
    });

    group.finish();
}

/// Benchmark file reassembly
fn bench_file_reassembly(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_reassembly");

    for size in [
        1_000_000,  // 1 MB
        10_000_000, // 10 MB
    ] {
        group.throughput(Throughput::Bytes(size));

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            // Prepare chunks
            let num_chunks = (size as usize).div_ceil(DEFAULT_CHUNK_SIZE);
            let chunks: Vec<_> = (0..num_chunks)
                .map(|_| vec![0xEE; DEFAULT_CHUNK_SIZE])
                .collect();

            b.iter(|| {
                let temp_file = NamedTempFile::new().unwrap();
                let mut reassembler =
                    FileReassembler::new(temp_file.path(), size, DEFAULT_CHUNK_SIZE).unwrap();

                for (i, chunk) in chunks.iter().enumerate() {
                    reassembler.write_chunk(i as u64, chunk).unwrap();
                }

                black_box(reassembler.is_complete())
            });
        });
    }

    group.finish();
}

/// Placeholder: Full transfer throughput benchmark
/// Requires protocol integration (Phase 7)
fn bench_transfer_throughput(_c: &mut Criterion) {
    // Placeholder for full transfer benchmark
    // Will be implemented in Phase 7

    // Benchmark structure:
    // - Setup sender and receiver nodes
    // - Transfer files of various sizes (1MB, 10MB, 100MB, 1GB)
    // - Measure throughput (bytes/sec)
    // - Target: >300 Mbps on 1 Gbps LAN
    // - Measure with different obfuscation levels
}

/// Placeholder: Transfer latency benchmark
/// Requires protocol integration (Phase 7)
fn bench_transfer_latency(_c: &mut Criterion) {
    // Placeholder for latency benchmark
    // Will be implemented in Phase 7

    // Benchmark structure:
    // - Measure round-trip time for chunk requests
    // - Measure handshake latency
    // - Measure initial chunk delivery time
    // - Target: <10ms RTT on LAN
}

/// Placeholder: BBR utilization benchmark
/// Requires protocol integration (Phase 7)
fn bench_bbr_utilization(_c: &mut Criterion) {
    // Placeholder for BBR benchmark
    // Will be implemented in Phase 7

    // Benchmark structure:
    // - Transfer large file (1GB)
    // - Measure bandwidth utilization over time
    // - Verify BBR achieves >95% link utilization
    // - Compare with and without BBR
}

/// Placeholder: Multi-peer speedup benchmark
/// Requires protocol integration (Phase 7)
fn bench_multi_peer_speedup(_c: &mut Criterion) {
    // Placeholder for multi-peer benchmark
    // Will be implemented in Phase 7

    // Benchmark structure:
    // - Transfer from 1, 2, 3, 4, 5 peers
    // - Measure throughput for each
    // - Verify linear speedup up to 5 peers
    // - Measure coordination overhead
}

criterion_group!(
    benches,
    bench_file_chunking,
    bench_tree_hashing,
    bench_tree_hashing_memory,
    bench_chunk_verification,
    bench_file_reassembly,
    // Full protocol benchmarks (Phase 7):
    // bench_transfer_throughput,
    // bench_transfer_latency,
    // bench_bbr_utilization,
    // bench_multi_peer_speedup,
);
criterion_main!(benches);
