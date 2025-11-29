//! Forward secrecy key ratcheting.
//!
//! Implements symmetric and DH ratcheting for continuous forward secrecy.

use zeroize::Zeroize;

/// Chain key for symmetric ratcheting
#[derive(Zeroize)]
#[zeroize(drop)]
pub struct ChainKey([u8; 32]);

impl ChainKey {
    /// Create from raw bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Ratchet forward and derive message key
    pub fn ratchet(&mut self) -> MessageKey {
        let old_key = self.0;

        // chain_key[n+1] = BLAKE3(chain_key[n] || 0x01)
        let mut hasher = blake3::Hasher::new();
        hasher.update(&old_key);
        hasher.update(&[0x01]);
        self.0.copy_from_slice(&hasher.finalize().as_bytes()[..32]);

        // message_key[n] = BLAKE3(chain_key[n] || 0x02)
        let mut hasher = blake3::Hasher::new();
        hasher.update(&old_key);
        hasher.update(&[0x02]);
        let mut msg_key = [0u8; 32];
        msg_key.copy_from_slice(&hasher.finalize().as_bytes()[..32]);

        MessageKey(msg_key)
    }
}

/// Message key derived from chain key
#[derive(Zeroize)]
#[zeroize(drop)]
pub struct MessageKey([u8; 32]);

impl MessageKey {
    /// Get the raw key bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ratchet_produces_different_keys() {
        let mut chain = ChainKey::from_bytes([0x42u8; 32]);

        let key1 = chain.ratchet();
        let key2 = chain.ratchet();

        assert_ne!(key1.as_bytes(), key2.as_bytes());
    }
}
