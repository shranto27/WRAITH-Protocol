//! Fuzz target for tree hash operations
//!
//! Tests that the tree hash functions correctly handle arbitrary input.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use wraith_files::tree_hash::{
    compute_merkle_root, compute_tree_hash_from_data, verify_chunk, IncrementalTreeHasher,
};

#[derive(Debug, Arbitrary)]
struct TreeHashInput {
    data: Vec<u8>,
    chunk_size: usize,
    chunk_index: usize,
    leaf_data: Vec<u8>,
}

fuzz_target!(|input: TreeHashInput| {
    // Use reasonable chunk sizes (1 byte to 1MB)
    let chunk_size = (input.chunk_size % (1024 * 1024)).max(1);

    // Fuzz compute_tree_hash_from_data - should never panic
    let tree = compute_tree_hash_from_data(&input.data, chunk_size);

    // Verify basic invariants
    if !input.data.is_empty() {
        assert!(tree.chunk_count() > 0, "Non-empty data should have chunks");
    }

    // Fuzz verify_chunk - should never panic
    let chunk_data = &input.data[..input.data.len().min(chunk_size)];
    let _ = verify_chunk(input.chunk_index, chunk_data, &tree);

    // Fuzz IncrementalTreeHasher
    let mut hasher = IncrementalTreeHasher::new(chunk_size);
    hasher.update(&input.data);
    let _ = hasher.finalize();

    // Fuzz compute_merkle_root with arbitrary leaf hashes
    let mut leaves = Vec::new();
    for chunk in input.leaf_data.chunks(32) {
        let mut hash = [0u8; 32];
        hash[..chunk.len()].copy_from_slice(chunk);
        leaves.push(hash);
    }
    let _ = compute_merkle_root(&leaves);
});
