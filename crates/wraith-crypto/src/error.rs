//! Cryptographic error types.

use thiserror::Error;

/// Cryptographic errors
#[derive(Debug, Error)]
pub enum CryptoError {
    /// AEAD encryption failed
    #[error("encryption failed")]
    EncryptionFailed,

    /// AEAD decryption failed (authentication failure)
    #[error("decryption failed: authentication failure")]
    DecryptionFailed,

    /// Invalid key length
    #[error("invalid key length: expected {expected}, got {actual}")]
    InvalidKeyLength {
        /// Expected length
        expected: usize,
        /// Actual length
        actual: usize,
    },

    /// Invalid nonce length
    #[error("invalid nonce length")]
    InvalidNonceLength,

    /// Noise handshake error
    #[error("handshake error: {0}")]
    Handshake(String),

    /// Key not encodable with Elligator2
    #[error("key not encodable with Elligator2")]
    NotEncodable,

    /// Random number generation failed
    #[error("random number generation failed")]
    RandomFailed,

    /// Nonce overflow (counter exhausted)
    #[error("nonce counter exhausted, rekey required")]
    NonceOverflow,
}
