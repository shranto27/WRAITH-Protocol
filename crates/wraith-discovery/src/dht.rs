//! Privacy-enhanced DHT for peer discovery.

/// DHT key derivation for announcements
pub fn derive_announce_key(group_secret: &[u8], file_hash: &[u8]) -> [u8; 20] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(group_secret);
    hasher.update(file_hash);
    hasher.update(b"announce");

    let hash = hasher.finalize();
    let mut key = [0u8; 20];
    key.copy_from_slice(&hash.as_bytes()[..20]);
    key
}
