//! # WRAITH Files
//!
//! File transfer engine for the WRAITH protocol.
//!
//! This crate provides:
//! - File chunking with configurable chunk size
//! - BLAKE3 tree hashing for integrity verification
//! - Transfer state machine with resume support
//! - Parallel chunk processing

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod chunker;
pub mod hasher;
pub mod transfer;

/// Default chunk size (256 KiB)
pub const DEFAULT_CHUNK_SIZE: usize = 256 * 1024;

/// File metadata for transfers
#[derive(Debug, Clone)]
pub struct FileMetadata {
    /// File name
    pub name: String,
    /// File size in bytes
    pub size: u64,
    /// BLAKE3 hash of entire file
    pub hash: [u8; 32],
    /// Number of chunks
    pub chunk_count: u64,
}
