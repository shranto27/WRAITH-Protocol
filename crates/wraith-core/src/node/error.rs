//! Error types for Node API
//!
//! This module provides a comprehensive error hierarchy for the WRAITH Node API.
//! Errors are categorized to support retry logic and proper error handling.
//!
//! # Error Categories
//!
//! - **Transient**: Temporary failures that may succeed on retry (network timeouts, congestion)
//! - **Permanent**: Failures that will not succeed without intervention (invalid config, not found)
//! - **Retriable**: A subset of transient errors with specific retry semantics
//!
//! # Example
//!
//! ```no_run
//! use wraith_core::node::{NodeError, Result};
//!
//! fn handle_error(err: NodeError) {
//!     if err.is_transient() {
//!         // Consider retrying with backoff
//!         println!("Transient error, may retry: {}", err);
//!     } else {
//!         // Permanent failure, needs user intervention
//!         println!("Permanent error: {}", err);
//!     }
//! }
//! ```

use thiserror::Error;

/// Errors that can occur in Node operations
#[derive(Debug, Error)]
pub enum NodeError {
    // ============ Transport Errors ============
    /// Failed to initialize transport layer
    #[error("Transport initialization failed: {0}")]
    TransportInit(String),

    /// Transport operation failed
    #[error("Transport error: {0}")]
    Transport(String),

    // ============ Cryptographic Errors ============
    /// Cryptographic operation failed
    #[error("Crypto error: {0}")]
    Crypto(#[from] wraith_crypto::CryptoError),

    /// Handshake failed
    #[error("Handshake failed: {0}")]
    Handshake(String),

    // ============ Session Errors ============
    /// Session establishment failed
    #[error("Session establishment failed: {0}")]
    SessionEstablishment(String),

    /// Session not found
    #[error("Session not found for peer {}", hex::encode(&.0[..8]))]
    SessionNotFound([u8; 32]),

    /// Session migration failed
    #[error("Session migration failed: {0}")]
    SessionMigration(String),

    // ============ Transfer Errors ============
    /// Transfer operation failed
    #[error("Transfer error: {0}")]
    Transfer(String),

    /// Transfer not found
    #[error("Transfer not found: {}", hex::encode(&.0[..8]))]
    TransferNotFound([u8; 32]),

    /// Hash mismatch during verification
    #[error("Hash mismatch: integrity verification failed")]
    HashMismatch,

    // ============ I/O Errors ============
    /// File I/O error
    #[error("File I/O error: {0}")]
    Io(#[from] std::io::Error),

    // ============ Discovery Errors ============
    /// Discovery operation failed
    #[error("Discovery error: {0}")]
    Discovery(String),

    /// NAT traversal failed
    #[error("NAT traversal failed: {0}")]
    NatTraversal(String),

    /// Peer not found in DHT or local cache
    #[error("Peer not found: {}", hex::encode(&.0[..8]))]
    PeerNotFound([u8; 32]),

    // ============ Connection Errors ============
    /// Connection migration failed
    #[error("Connection migration failed: {0}")]
    Migration(String),

    /// Obfuscation operation failed
    #[error("Obfuscation error: {0}")]
    Obfuscation(String),

    // ============ Configuration & State Errors ============
    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Invalid state transition
    #[error("Invalid state: {0}")]
    InvalidState(String),

    // ============ Operational Errors ============
    /// Operation timed out
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// Task join error
    #[error("Task join error: {0}")]
    TaskJoin(String),

    /// Channel send/receive error
    #[error("Channel error: {0}")]
    Channel(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Generic error for edge cases
    #[error("{0}")]
    Other(String),
}

impl NodeError {
    /// Returns true if this error is transient and may succeed on retry
    ///
    /// Transient errors include:
    /// - Network timeouts
    /// - Transport failures (may be congestion)
    /// - NAT traversal failures (may succeed with different candidates)
    /// - Channel errors (may be temporary backpressure)
    #[must_use]
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            NodeError::Timeout(_)
                | NodeError::Transport(_)
                | NodeError::NatTraversal(_)
                | NodeError::Channel(_)
                | NodeError::Migration(_)
        )
    }

    /// Returns true if this error is permanent and will not succeed on retry
    ///
    /// Permanent errors include:
    /// - Invalid configuration
    /// - Session/transfer not found
    /// - Hash mismatches (data corruption)
    /// - Cryptographic failures
    #[must_use]
    pub fn is_permanent(&self) -> bool {
        matches!(
            self,
            NodeError::InvalidConfig(_)
                | NodeError::SessionNotFound(_)
                | NodeError::TransferNotFound(_)
                | NodeError::PeerNotFound(_)
                | NodeError::HashMismatch
                | NodeError::InvalidState(_)
        )
    }

    /// Returns true if this error should trigger a retry with exponential backoff
    #[must_use]
    pub fn should_retry(&self) -> bool {
        self.is_transient() && !matches!(self, NodeError::Timeout(_))
    }

    /// Create a transport error with context
    #[must_use]
    pub fn transport(context: impl Into<String>) -> Self {
        NodeError::Transport(context.into())
    }

    /// Create a timeout error with context
    #[must_use]
    pub fn timeout(context: impl Into<String>) -> Self {
        NodeError::Timeout(context.into())
    }

    /// Create a handshake error with context
    #[must_use]
    pub fn handshake(context: impl Into<String>) -> Self {
        NodeError::Handshake(context.into())
    }

    /// Create an invalid state error with context
    #[must_use]
    pub fn invalid_state(context: impl Into<String>) -> Self {
        NodeError::InvalidState(context.into())
    }

    /// Create a discovery error with context
    #[must_use]
    pub fn discovery(context: impl Into<String>) -> Self {
        NodeError::Discovery(context.into())
    }

    /// Create a serialization error with context
    #[must_use]
    pub fn serialization(context: impl Into<String>) -> Self {
        NodeError::Serialization(context.into())
    }
}

/// Result type for Node operations
pub type Result<T> = std::result::Result<T, NodeError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transient_errors() {
        assert!(NodeError::Timeout("test".to_string()).is_transient());
        assert!(NodeError::Transport("test".to_string()).is_transient());
        assert!(NodeError::NatTraversal("test".to_string()).is_transient());
        assert!(NodeError::Channel("test".to_string()).is_transient());
        assert!(NodeError::Migration("test".to_string()).is_transient());
    }

    #[test]
    fn test_permanent_errors() {
        assert!(NodeError::InvalidConfig("test".to_string()).is_permanent());
        assert!(NodeError::SessionNotFound([0u8; 32]).is_permanent());
        assert!(NodeError::TransferNotFound([0u8; 32]).is_permanent());
        assert!(NodeError::PeerNotFound([0u8; 32]).is_permanent());
        assert!(NodeError::HashMismatch.is_permanent());
        assert!(NodeError::InvalidState("test".to_string()).is_permanent());
    }

    #[test]
    fn test_should_retry() {
        // Transient non-timeout errors should retry
        assert!(NodeError::Transport("test".to_string()).should_retry());
        assert!(NodeError::NatTraversal("test".to_string()).should_retry());
        assert!(NodeError::Channel("test".to_string()).should_retry());

        // Timeouts should not auto-retry (caller decides)
        assert!(!NodeError::Timeout("test".to_string()).should_retry());

        // Permanent errors should not retry
        assert!(!NodeError::InvalidConfig("test".to_string()).should_retry());
        assert!(!NodeError::SessionNotFound([0u8; 32]).should_retry());
    }

    #[test]
    fn test_error_display() {
        let mut peer_id = [0u8; 32];
        peer_id[0..8].copy_from_slice(&[0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0]);
        let err = NodeError::SessionNotFound(peer_id);
        assert!(err.to_string().contains("123456789abcdef0"));

        let err = NodeError::HashMismatch;
        assert!(err.to_string().contains("integrity verification"));
    }

    #[test]
    fn test_convenience_constructors() {
        let err = NodeError::transport("connection refused");
        assert!(matches!(err, NodeError::Transport(_)));

        let err = NodeError::timeout("handshake");
        assert!(matches!(err, NodeError::Timeout(_)));

        let err = NodeError::handshake("invalid message");
        assert!(matches!(err, NodeError::Handshake(_)));

        let err = NodeError::invalid_state("not running");
        assert!(matches!(err, NodeError::InvalidState(_)));

        let err = NodeError::discovery("DHT unreachable");
        assert!(matches!(err, NodeError::Discovery(_)));

        let err = NodeError::serialization("invalid JSON");
        assert!(matches!(err, NodeError::Serialization(_)));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let node_err: NodeError = io_err.into();
        assert!(matches!(node_err, NodeError::Io(_)));
    }

    #[test]
    fn test_mutual_exclusivity() {
        // Errors should be either transient or permanent, not both
        let transient_errors = [
            NodeError::Timeout("test".to_string()),
            NodeError::Transport("test".to_string()),
            NodeError::NatTraversal("test".to_string()),
        ];

        for err in &transient_errors {
            assert!(err.is_transient());
            assert!(!err.is_permanent());
        }

        let permanent_errors = [
            NodeError::InvalidConfig("test".to_string()),
            NodeError::SessionNotFound([0u8; 32]),
            NodeError::HashMismatch,
        ];

        for err in &permanent_errors {
            assert!(err.is_permanent());
            assert!(!err.is_transient());
        }
    }
}
