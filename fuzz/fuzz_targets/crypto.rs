//! Fuzz target for cryptographic operations
//!
//! Tests that the AEAD encrypt/decrypt operations correctly handle arbitrary input.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use wraith_crypto::aead::{AeadKey, Nonce};

#[derive(Debug, Arbitrary)]
struct CryptoInput {
    key: [u8; 32],
    nonce: [u8; 24],
    plaintext: Vec<u8>,
    aad: Vec<u8>,
}

fuzz_target!(|input: CryptoInput| {
    let key = AeadKey::new(input.key);
    let nonce = Nonce::from_bytes(input.nonce);

    // Fuzz encryption - should never panic
    if let Ok(ciphertext) = key.encrypt(&nonce, &input.plaintext, &input.aad) {
        // If encryption succeeded, decryption with same params should work
        let _ = key.decrypt(&nonce, &ciphertext, &input.aad);
    }

    // Fuzz decryption with arbitrary data - should never panic
    let _ = key.decrypt(&nonce, &input.plaintext, &input.aad);

    // Fuzz key commitment
    let _ = key.commitment();
});
