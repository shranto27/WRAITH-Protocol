//! `XChaCha20-Poly1305` AEAD encryption.
//!
//! Provides authenticated encryption with associated data (AEAD) using
//! `XChaCha20-Poly1305`. Features include:
//! - 256-bit keys
//! - 192-bit nonces (extended nonce for safe random generation)
//! - 128-bit authentication tags
//! - Associated data authentication
//! - In-place encryption/decryption for zero-copy operations
//!
//! ## Security Properties
//!
//! - Confidentiality: `XChaCha20` stream cipher
//! - Integrity: Poly1305 MAC with 128-bit security
//! - Nonce misuse: 192-bit nonce makes random collisions negligible
//!
//! ## Usage
//!
//! ```ignore
//! use wraith_crypto::aead::{AeadKey, AeadCipher, Nonce};
//!
//! let key = AeadKey::generate(&mut OsRng);
//! let nonce = Nonce::generate(&mut OsRng);
//!
//! let ciphertext = key.encrypt(&nonce, b"secret", b"aad")?;
//! let plaintext = key.decrypt(&nonce, &ciphertext, b"aad")?;
//! ```

use crate::CryptoError;
use chacha20poly1305::{
    XChaCha20Poly1305,
    aead::{Aead, AeadInPlace, KeyInit},
};
use rand_core::{CryptoRng, RngCore};
use zeroize::ZeroizeOnDrop;

/// Authentication tag size (16 bytes / 128 bits).
pub const TAG_SIZE: usize = 16;

/// XChaCha20-Poly1305 nonce size (24 bytes / 192 bits).
pub const NONCE_SIZE: usize = 24;

/// AEAD key size (32 bytes / 256 bits).
pub const KEY_SIZE: usize = 32;

/// XChaCha20-Poly1305 nonce (24 bytes).
///
/// The extended 192-bit nonce allows safe random nonce generation
/// without risk of collision (birthday bound is 2^96 messages).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Nonce([u8; NONCE_SIZE]);

impl Nonce {
    /// Create a nonce from raw bytes.
    #[must_use]
    pub fn from_bytes(bytes: [u8; NONCE_SIZE]) -> Self {
        Self(bytes)
    }

    /// Create a nonce from a slice.
    #[must_use]
    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        if slice.len() != NONCE_SIZE {
            return None;
        }
        let mut bytes = [0u8; NONCE_SIZE];
        bytes.copy_from_slice(slice);
        Some(Self(bytes))
    }

    /// Generate a random nonce.
    #[must_use]
    pub fn generate<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        let mut bytes = [0u8; NONCE_SIZE];
        rng.fill_bytes(&mut bytes);
        Self(bytes)
    }

    /// Create a nonce from a counter value.
    ///
    /// The counter is placed in the first 8 bytes (little-endian),
    /// with the remaining 16 bytes available for session ID or salt.
    #[must_use]
    pub fn from_counter(counter: u64, salt: &[u8; 16]) -> Self {
        let mut bytes = [0u8; NONCE_SIZE];
        bytes[..8].copy_from_slice(&counter.to_le_bytes());
        bytes[8..].copy_from_slice(salt);
        Self(bytes)
    }

    /// Get raw bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; NONCE_SIZE] {
        &self.0
    }

    /// Get as a reference for chacha20poly1305.
    fn as_generic(&self) -> &chacha20poly1305::XNonce {
        chacha20poly1305::XNonce::from_slice(&self.0)
    }
}

impl Default for Nonce {
    fn default() -> Self {
        Self([0u8; NONCE_SIZE])
    }
}

/// Authentication tag (16 bytes).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tag([u8; TAG_SIZE]);

impl Tag {
    /// Create a tag from raw bytes.
    #[must_use]
    pub fn from_bytes(bytes: [u8; TAG_SIZE]) -> Self {
        Self(bytes)
    }

    /// Create from slice.
    #[must_use]
    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        if slice.len() != TAG_SIZE {
            return None;
        }
        let mut bytes = [0u8; TAG_SIZE];
        bytes.copy_from_slice(slice);
        Some(Self(bytes))
    }

    /// Get raw bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; TAG_SIZE] {
        &self.0
    }
}

/// AEAD encryption key (32 bytes).
///
/// Wraps the raw key material and provides encryption/decryption methods.
/// Key is zeroized on drop.
#[derive(Clone, ZeroizeOnDrop)]
pub struct AeadKey([u8; KEY_SIZE]);

impl AeadKey {
    /// Create a key from raw bytes.
    #[must_use]
    pub fn new(bytes: [u8; KEY_SIZE]) -> Self {
        Self(bytes)
    }

    /// Create from slice.
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::InvalidKeyLength` if slice length is not 32 bytes.
    pub fn from_slice(slice: &[u8]) -> Result<Self, CryptoError> {
        if slice.len() != KEY_SIZE {
            return Err(CryptoError::InvalidKeyLength {
                expected: KEY_SIZE,
                actual: slice.len(),
            });
        }
        let mut bytes = [0u8; KEY_SIZE];
        bytes.copy_from_slice(slice);
        Ok(Self(bytes))
    }

    /// Generate a random key.
    #[must_use]
    pub fn generate<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        let mut bytes = [0u8; KEY_SIZE];
        rng.fill_bytes(&mut bytes);
        Self(bytes)
    }

    /// Get raw key bytes.
    ///
    /// # Security
    ///
    /// Handle with extreme care - this exposes the raw key material.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; KEY_SIZE] {
        &self.0
    }

    /// Encrypt plaintext with associated data.
    ///
    /// Returns ciphertext with appended authentication tag (`plaintext.len()` + 16 bytes).
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::EncryptionFailed` if AEAD encryption fails.
    pub fn encrypt(
        &self,
        nonce: &Nonce,
        plaintext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>, CryptoError> {
        let cipher = XChaCha20Poly1305::new((&self.0).into());

        cipher
            .encrypt(
                nonce.as_generic(),
                chacha20poly1305::aead::Payload {
                    msg: plaintext,
                    aad,
                },
            )
            .map_err(|_| CryptoError::EncryptionFailed)
    }

    /// Decrypt ciphertext with associated data.
    ///
    /// Input must include the authentication tag at the end.
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::DecryptionFailed` on authentication failure.
    pub fn decrypt(
        &self,
        nonce: &Nonce,
        ciphertext_and_tag: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>, CryptoError> {
        if ciphertext_and_tag.len() < TAG_SIZE {
            return Err(CryptoError::DecryptionFailed);
        }

        let cipher = XChaCha20Poly1305::new((&self.0).into());

        cipher
            .decrypt(
                nonce.as_generic(),
                chacha20poly1305::aead::Payload {
                    msg: ciphertext_and_tag,
                    aad,
                },
            )
            .map_err(|_| CryptoError::DecryptionFailed)
    }

    /// Encrypt in-place, returning the authentication tag.
    ///
    /// The buffer is modified in-place to contain the ciphertext.
    /// Returns the authentication tag separately.
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::EncryptionFailed` if AEAD encryption fails.
    pub fn encrypt_in_place(
        &self,
        nonce: &Nonce,
        buffer: &mut [u8],
        aad: &[u8],
    ) -> Result<Tag, CryptoError> {
        let cipher = XChaCha20Poly1305::new((&self.0).into());

        let tag = cipher
            .encrypt_in_place_detached(nonce.as_generic(), aad, buffer)
            .map_err(|_| CryptoError::EncryptionFailed)?;

        let mut tag_bytes = [0u8; TAG_SIZE];
        tag_bytes.copy_from_slice(&tag);
        Ok(Tag(tag_bytes))
    }

    /// Decrypt in-place, verifying the authentication tag.
    ///
    /// The buffer is modified in-place to contain the plaintext.
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::DecryptionFailed` on authentication failure.
    pub fn decrypt_in_place(
        &self,
        nonce: &Nonce,
        buffer: &mut [u8],
        tag: &Tag,
        aad: &[u8],
    ) -> Result<(), CryptoError> {
        let cipher = XChaCha20Poly1305::new((&self.0).into());

        cipher
            .decrypt_in_place_detached(
                nonce.as_generic(),
                aad,
                buffer,
                chacha20poly1305::Tag::from_slice(&tag.0),
            )
            .map_err(|_| CryptoError::DecryptionFailed)
    }
}

/// AEAD cipher for packet encryption (legacy API).
///
/// Use `AeadKey` directly for new code.
pub struct AeadCipher {
    cipher: XChaCha20Poly1305,
}

impl AeadCipher {
    /// Create a new AEAD cipher with the given key.
    #[must_use]
    pub fn new(key: &[u8; KEY_SIZE]) -> Self {
        Self {
            cipher: XChaCha20Poly1305::new(key.into()),
        }
    }

    /// Encrypt plaintext with the given nonce and associated data.
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::EncryptionFailed` if AEAD encryption fails.
    pub fn encrypt(
        &self,
        nonce: &[u8; NONCE_SIZE],
        plaintext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>, CryptoError> {
        use chacha20poly1305::aead::Payload;

        let payload = Payload {
            msg: plaintext,
            aad,
        };

        self.cipher
            .encrypt(nonce.into(), payload)
            .map_err(|_| CryptoError::EncryptionFailed)
    }

    /// Decrypt ciphertext with the given nonce and associated data.
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::DecryptionFailed` on authentication failure.
    pub fn decrypt(
        &self,
        nonce: &[u8; NONCE_SIZE],
        ciphertext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>, CryptoError> {
        use chacha20poly1305::aead::Payload;

        let payload = Payload {
            msg: ciphertext,
            aad,
        };

        self.cipher
            .decrypt(nonce.into(), payload)
            .map_err(|_| CryptoError::DecryptionFailed)
    }
}

/// Session encryption state for post-handshake communication.
///
/// Manages nonce counters and keys for bidirectional encrypted communication.
#[derive(ZeroizeOnDrop)]
pub struct SessionCrypto {
    /// Key for sending messages
    send_key: AeadKey,
    /// Key for receiving messages
    recv_key: AeadKey,
    /// Nonce salt (derived from session)
    #[zeroize(skip)]
    nonce_salt: [u8; 16],
    /// Send nonce counter
    #[zeroize(skip)]
    send_counter: u64,
    /// Receive nonce counter
    #[zeroize(skip)]
    recv_counter: u64,
    /// Maximum allowed counter before rekey
    #[zeroize(skip)]
    max_counter: u64,
}

impl SessionCrypto {
    /// Create a new session crypto state from session keys.
    #[must_use]
    pub fn new(send_key: [u8; 32], recv_key: [u8; 32], chain_key: &[u8; 32]) -> Self {
        // Derive nonce salt from chain key
        let mut nonce_salt = [0u8; 16];
        nonce_salt.copy_from_slice(&chain_key[..16]);

        Self {
            send_key: AeadKey::new(send_key),
            recv_key: AeadKey::new(recv_key),
            nonce_salt,
            send_counter: 0,
            recv_counter: 0,
            max_counter: 1_000_000, // Rekey after 1M messages
        }
    }

    /// Encrypt a message.
    ///
    /// Returns the ciphertext with authentication tag.
    /// Automatically increments the send counter.
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::NonceOverflow` if send counter is exhausted.
    /// Returns `CryptoError::EncryptionFailed` on AEAD encryption failure.
    pub fn encrypt(&mut self, plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>, CryptoError> {
        if self.send_counter >= self.max_counter {
            return Err(CryptoError::NonceOverflow);
        }

        let nonce = Nonce::from_counter(self.send_counter, &self.nonce_salt);
        self.send_counter += 1;

        self.send_key.encrypt(&nonce, plaintext, aad)
    }

    /// Encrypt a message with explicit counter.
    ///
    /// Returns the ciphertext and the counter used.
    /// Does NOT automatically increment the counter (caller's responsibility).
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::EncryptionFailed` on AEAD encryption failure.
    pub fn encrypt_with_counter(
        &self,
        counter: u64,
        plaintext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>, CryptoError> {
        let nonce = Nonce::from_counter(counter, &self.nonce_salt);
        self.send_key.encrypt(&nonce, plaintext, aad)
    }

    /// Decrypt a message.
    ///
    /// Uses the receive counter for nonce generation.
    /// Automatically increments the receive counter.
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::NonceOverflow` if receive counter is exhausted.
    /// Returns `CryptoError::DecryptionFailed` on authentication failure.
    pub fn decrypt(&mut self, ciphertext: &[u8], aad: &[u8]) -> Result<Vec<u8>, CryptoError> {
        if self.recv_counter >= self.max_counter {
            return Err(CryptoError::NonceOverflow);
        }

        let nonce = Nonce::from_counter(self.recv_counter, &self.nonce_salt);
        self.recv_counter += 1;

        self.recv_key.decrypt(&nonce, ciphertext, aad)
    }

    /// Decrypt a message with explicit counter.
    ///
    /// Does NOT automatically increment the counter.
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::DecryptionFailed` on authentication failure.
    pub fn decrypt_with_counter(
        &self,
        counter: u64,
        ciphertext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>, CryptoError> {
        let nonce = Nonce::from_counter(counter, &self.nonce_salt);
        self.recv_key.decrypt(&nonce, ciphertext, aad)
    }

    /// Get the current send counter.
    #[must_use]
    pub fn send_counter(&self) -> u64 {
        self.send_counter
    }

    /// Get the current receive counter.
    #[must_use]
    pub fn recv_counter(&self) -> u64 {
        self.recv_counter
    }

    /// Check if rekey is needed.
    #[must_use]
    pub fn needs_rekey(&self) -> bool {
        self.send_counter >= self.max_counter || self.recv_counter >= self.max_counter
    }

    /// Update keys for a new session (ratchet).
    pub fn update_keys(&mut self, send_key: [u8; 32], recv_key: [u8; 32], chain_key: &[u8; 32]) {
        self.send_key = AeadKey::new(send_key);
        self.recv_key = AeadKey::new(recv_key);
        self.nonce_salt.copy_from_slice(&chain_key[..16]);
        self.send_counter = 0;
        self.recv_counter = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_core::OsRng;

    #[test]
    fn test_aead_roundtrip() {
        let key = [0x42u8; 32];
        let nonce = [0x00u8; 24];
        let plaintext = b"Hello, WRAITH!";
        let aad = b"additional data";

        let cipher = AeadCipher::new(&key);

        let ciphertext = cipher.encrypt(&nonce, plaintext, aad).unwrap();
        let decrypted = cipher.decrypt(&nonce, &ciphertext, aad).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_aead_tamper_detection() {
        let key = [0x42u8; 32];
        let nonce = [0x00u8; 24];
        let plaintext = b"Hello, WRAITH!";
        let aad = b"additional data";

        let cipher = AeadCipher::new(&key);

        let mut ciphertext = cipher.encrypt(&nonce, plaintext, aad).unwrap();
        ciphertext[0] ^= 0xFF; // Tamper with ciphertext

        assert!(cipher.decrypt(&nonce, &ciphertext, aad).is_err());
    }

    #[test]
    fn test_aead_key_encrypt_decrypt() {
        let key = AeadKey::generate(&mut OsRng);
        let nonce = Nonce::generate(&mut OsRng);
        let plaintext = b"secret message";
        let aad = b"header";

        let ciphertext = key.encrypt(&nonce, plaintext, aad).unwrap();
        assert_eq!(ciphertext.len(), plaintext.len() + TAG_SIZE);

        let decrypted = key.decrypt(&nonce, &ciphertext, aad).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_aead_wrong_key_fails() {
        let key1 = AeadKey::generate(&mut OsRng);
        let key2 = AeadKey::generate(&mut OsRng);
        let nonce = Nonce::generate(&mut OsRng);

        let ciphertext = key1.encrypt(&nonce, b"secret", b"").unwrap();
        assert!(key2.decrypt(&nonce, &ciphertext, b"").is_err());
    }

    #[test]
    fn test_aead_wrong_nonce_fails() {
        let key = AeadKey::generate(&mut OsRng);
        let nonce1 = Nonce::generate(&mut OsRng);
        let nonce2 = Nonce::generate(&mut OsRng);

        let ciphertext = key.encrypt(&nonce1, b"secret", b"").unwrap();
        assert!(key.decrypt(&nonce2, &ciphertext, b"").is_err());
    }

    #[test]
    fn test_aead_wrong_aad_fails() {
        let key = AeadKey::generate(&mut OsRng);
        let nonce = Nonce::generate(&mut OsRng);

        let ciphertext = key.encrypt(&nonce, b"secret", b"aad1").unwrap();
        assert!(key.decrypt(&nonce, &ciphertext, b"aad2").is_err());
    }

    #[test]
    fn test_aead_in_place() {
        let key = AeadKey::generate(&mut OsRng);
        let nonce = Nonce::generate(&mut OsRng);
        let plaintext = b"hello world";
        let mut buffer = plaintext.to_vec();

        let tag = key.encrypt_in_place(&nonce, &mut buffer, b"").unwrap();
        assert_ne!(&buffer, plaintext);

        key.decrypt_in_place(&nonce, &mut buffer, &tag, b"")
            .unwrap();
        assert_eq!(&buffer, plaintext);
    }

    #[test]
    fn test_nonce_from_counter() {
        let salt = [0x42u8; 16];
        let nonce1 = Nonce::from_counter(0, &salt);
        let nonce2 = Nonce::from_counter(1, &salt);
        let nonce3 = Nonce::from_counter(0, &salt);

        assert_ne!(nonce1.as_bytes(), nonce2.as_bytes());
        assert_eq!(nonce1.as_bytes(), nonce3.as_bytes());
    }

    #[test]
    fn test_session_crypto_encrypt_decrypt() {
        let send_key = [1u8; 32];
        let recv_key = [2u8; 32];
        let chain_key = [3u8; 32];

        // Create two sessions with swapped keys
        let mut alice = SessionCrypto::new(send_key, recv_key, &chain_key);
        let mut bob = SessionCrypto::new(recv_key, send_key, &chain_key);

        // Alice sends to Bob
        let plaintext1 = b"hello from alice";
        let ct1 = alice.encrypt(plaintext1, b"").unwrap();
        let pt1 = bob.decrypt(&ct1, b"").unwrap();
        assert_eq!(pt1, plaintext1);

        // Bob sends to Alice
        let plaintext2 = b"hello from bob";
        let ct2 = bob.encrypt(plaintext2, b"").unwrap();
        let pt2 = alice.decrypt(&ct2, b"").unwrap();
        assert_eq!(pt2, plaintext2);
    }

    #[test]
    fn test_session_crypto_counter_increment() {
        let mut session = SessionCrypto::new([1u8; 32], [2u8; 32], &[3u8; 32]);

        assert_eq!(session.send_counter(), 0);
        let _ = session.encrypt(b"test", b"").unwrap();
        assert_eq!(session.send_counter(), 1);
        let _ = session.encrypt(b"test", b"").unwrap();
        assert_eq!(session.send_counter(), 2);
    }

    #[test]
    fn test_session_crypto_with_explicit_counter() {
        let send_key = [1u8; 32];
        let recv_key = [2u8; 32];
        let chain_key = [3u8; 32];

        let alice = SessionCrypto::new(send_key, recv_key, &chain_key);
        let bob = SessionCrypto::new(recv_key, send_key, &chain_key);

        // Alice encrypts with counter 5
        let ct = alice.encrypt_with_counter(5, b"message", b"").unwrap();

        // Bob decrypts with counter 5
        let pt = bob.decrypt_with_counter(5, &ct, b"").unwrap();
        assert_eq!(pt, b"message");
    }

    #[test]
    fn test_tag_from_slice() {
        let bytes = [0x42u8; TAG_SIZE];
        let tag = Tag::from_slice(&bytes).unwrap();
        assert_eq!(tag.as_bytes(), &bytes);

        // Wrong size should fail
        assert!(Tag::from_slice(&[0u8; 15]).is_none());
    }

    #[test]
    fn test_nonce_from_slice() {
        let bytes = [0x42u8; NONCE_SIZE];
        let nonce = Nonce::from_slice(&bytes).unwrap();
        assert_eq!(nonce.as_bytes(), &bytes);

        // Wrong size should fail
        assert!(Nonce::from_slice(&[0u8; 23]).is_none());
    }
}
