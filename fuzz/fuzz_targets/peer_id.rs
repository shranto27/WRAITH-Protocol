//! Fuzz test for peer ID parsing
//!
//! Tests that arbitrary input doesn't cause panics when parsed as peer IDs.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Test 1: Try parsing as hex string (peer IDs are 64 hex chars)
    if let Ok(s) = std::str::from_utf8(data) {
        // Try decoding as hex
        let _ = hex::decode(s);

        // Try decoding with different case variations
        let _ = hex::decode(s.to_lowercase());
        let _ = hex::decode(s.to_uppercase());

        // Test parsing specific lengths
        if s.len() == 64 {
            if let Ok(bytes) = hex::decode(s) {
                assert_eq!(bytes.len(), 32, "64 hex chars should decode to 32 bytes");
            }
        }
    }

    // Test 2: Try using raw bytes as peer ID (should be exactly 32 bytes)
    if data.len() == 32 {
        let peer_id: [u8; 32] = data.try_into().unwrap();

        // Test encoding to hex
        let hex_str = hex::encode(peer_id);
        assert_eq!(hex_str.len(), 64, "32 bytes should encode to 64 hex chars");

        // Test round-trip
        let decoded = hex::decode(&hex_str).unwrap();
        assert_eq!(decoded.as_slice(), &peer_id);
    }

    // Test 3: Try truncating or padding to 32 bytes
    let mut padded = data.to_vec();
    padded.resize(32, 0);
    let _peer_id: [u8; 32] = padded[..32].try_into().unwrap();
});
