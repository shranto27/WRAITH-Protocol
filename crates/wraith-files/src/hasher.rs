//! BLAKE3 tree hashing for file integrity.

/// Hash a chunk and return truncated hash for frame
pub fn hash_chunk(data: &[u8]) -> [u8; 16] {
    let hash = blake3::hash(data);
    let mut truncated = [0u8; 16];
    truncated.copy_from_slice(&hash.as_bytes()[..16]);
    truncated
}

/// Hash an entire file
pub fn hash_file(data: &[u8]) -> [u8; 32] {
    *blake3::hash(data).as_bytes()
}

/// Verify a chunk against its expected hash
pub fn verify_chunk(data: &[u8], expected: &[u8; 16]) -> bool {
    hash_chunk(data) == *expected
}
