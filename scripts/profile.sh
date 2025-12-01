#!/bin/bash
# Performance profiling script for WRAITH Protocol
#
# This script provides comprehensive profiling for the WRAITH Protocol using
# various tools: perf, flamegraph, valgrind, and criterion.
#
# Prerequisites:
#   - Linux kernel with perf support
#   - perf, valgrind installed
#   - cargo-flamegraph: cargo install flamegraph
#   - inferno: cargo install inferno (optional, for flamegraphs)
#
# Usage:
#   ./scripts/profile.sh [cpu|memory|cache|bench|all]
#
# Output:
#   - flamegraph.svg: CPU profiling flamegraph
#   - memory_profile.txt: Memory allocation profile
#   - cache_stats.txt: Cache efficiency statistics
#   - benchmark_results/: Criterion benchmark results

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
OUTPUT_DIR="$PROJECT_ROOT/target/profile"

# Create output directory
mkdir -p "$OUTPUT_DIR"

echo "=== WRAITH Protocol Performance Profiling ==="
echo "Output directory: $OUTPUT_DIR"
echo ""

# Build release version
build_release() {
    echo "Building release binary..."
    cd "$PROJECT_ROOT"
    cargo build --release --workspace
    echo "Build complete."
    echo ""
}

# CPU profiling with perf and flamegraph
profile_cpu() {
    echo "=== CPU Profiling ==="

    # Check for perf
    if ! command -v perf &> /dev/null; then
        echo "WARNING: perf not found. Skipping perf profiling."
        echo "Install with: sudo apt install linux-tools-generic"
        return
    fi

    # Check for cargo-flamegraph
    if ! cargo flamegraph --help &> /dev/null; then
        echo "INFO: cargo-flamegraph not found. Installing..."
        cargo install flamegraph
    fi

    echo "Running CPU profiling with flamegraph..."
    cd "$PROJECT_ROOT"

    # Run benchmarks with flamegraph
    CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph \
        --bench transfer \
        --output "$OUTPUT_DIR/flamegraph_transfer.svg" \
        -- --bench 2>&1 | tee "$OUTPUT_DIR/flamegraph_output.txt" || {
        echo "WARNING: Flamegraph requires sudo for perf. Try: sudo ./scripts/profile.sh cpu"
    }

    # Also profile crypto benchmarks
    CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph \
        --bench crypto_bench \
        -p wraith-crypto \
        --output "$OUTPUT_DIR/flamegraph_crypto.svg" \
        -- --bench 2>&1 || true

    echo "CPU profiling complete."
    echo "Flamegraphs: $OUTPUT_DIR/flamegraph_*.svg"
    echo ""
}

# Memory profiling with valgrind
profile_memory() {
    echo "=== Memory Profiling ==="

    # Check for valgrind
    if ! command -v valgrind &> /dev/null; then
        echo "WARNING: valgrind not found. Skipping memory profiling."
        echo "Install with: sudo apt install valgrind"
        return
    fi

    cd "$PROJECT_ROOT"

    # Build test binary
    cargo build --release --tests -p wraith-files

    # Find test binary
    TEST_BIN=$(find target/release/deps -name "wraith_files-*" -type f -executable | head -1)

    if [ -z "$TEST_BIN" ]; then
        echo "WARNING: Test binary not found. Skipping memory profiling."
        return
    fi

    echo "Running valgrind massif..."
    valgrind --tool=massif \
        --massif-out-file="$OUTPUT_DIR/massif.out" \
        --pages-as-heap=yes \
        "$TEST_BIN" --test-threads=1 2>&1 | tee "$OUTPUT_DIR/valgrind_output.txt" || true

    # Generate readable report
    if command -v ms_print &> /dev/null; then
        ms_print "$OUTPUT_DIR/massif.out" > "$OUTPUT_DIR/memory_profile.txt"
        echo "Memory profile: $OUTPUT_DIR/memory_profile.txt"
    fi

    # Run leak check
    echo "Running valgrind leak check..."
    valgrind --leak-check=full \
        --show-leak-kinds=all \
        --track-origins=yes \
        "$TEST_BIN" --test-threads=1 2>&1 | tee "$OUTPUT_DIR/leak_check.txt" || true

    echo "Memory profiling complete."
    echo ""
}

# Cache profiling with perf
profile_cache() {
    echo "=== Cache Profiling ==="

    if ! command -v perf &> /dev/null; then
        echo "WARNING: perf not found. Skipping cache profiling."
        return
    fi

    cd "$PROJECT_ROOT"

    echo "Running cache analysis..."

    # Build benchmark binary
    cargo build --release --benches -p wraith-files

    # Find benchmark binary
    BENCH_BIN=$(find target/release/deps -name "transfer-*" -type f -executable | head -1)

    if [ -z "$BENCH_BIN" ]; then
        echo "WARNING: Benchmark binary not found."
        return
    fi

    # Run perf stat for cache analysis
    perf stat -e cache-references,cache-misses,L1-dcache-loads,L1-dcache-load-misses,instructions,cycles \
        "$BENCH_BIN" --bench 2>&1 | tee "$OUTPUT_DIR/cache_stats.txt" || {
        echo "WARNING: perf requires elevated permissions. Try: sudo ./scripts/profile.sh cache"
    }

    echo "Cache profiling complete."
    echo "Results: $OUTPUT_DIR/cache_stats.txt"
    echo ""
}

# Run criterion benchmarks
run_benchmarks() {
    echo "=== Running Benchmarks ==="

    cd "$PROJECT_ROOT"

    echo "Running all benchmarks..."
    cargo bench --workspace 2>&1 | tee "$OUTPUT_DIR/benchmark_results.txt"

    echo "Benchmarks complete."
    echo "Results: $OUTPUT_DIR/benchmark_results.txt"
    echo "HTML reports: target/criterion/"
    echo ""
}

# Print summary
print_summary() {
    echo "=== Profiling Summary ==="
    echo ""
    echo "Output files:"
    ls -la "$OUTPUT_DIR/" 2>/dev/null || true
    echo ""
    echo "Key metrics to look for:"
    echo "  - Flamegraph: Look for tall stacks indicating hotspots"
    echo "  - Memory: Peak usage, allocation patterns"
    echo "  - Cache: Cache miss rate should be <5% for hot paths"
    echo "  - Benchmarks: Compare against performance targets"
    echo ""
    echo "Performance targets:"
    echo "  - AEAD throughput: >3 GB/s"
    echo "  - Frame parsing: >10M frames/sec"
    echo "  - Tree hashing: >3 GB/s"
    echo "  - Chunk verification: <1us"
    echo ""
}

# Main
case "${1:-all}" in
    cpu)
        build_release
        profile_cpu
        ;;
    memory)
        build_release
        profile_memory
        ;;
    cache)
        build_release
        profile_cache
        ;;
    bench)
        run_benchmarks
        ;;
    all)
        build_release
        profile_cpu
        profile_memory
        profile_cache
        run_benchmarks
        print_summary
        ;;
    *)
        echo "Usage: $0 [cpu|memory|cache|bench|all]"
        exit 1
        ;;
esac

echo "Profiling complete!"
