//! # WRAITH Obfuscation
//!
//! Traffic obfuscation layer for the WRAITH protocol.
//!
//! This crate provides:
//! - Packet padding to fixed size classes
//! - Timing obfuscation with jitter
//! - Cover traffic generation
//! - Protocol mimicry (HTTPS, WebSocket, DoH)

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod cover;
pub mod padding;
pub mod timing;

pub use cover::{CoverTrafficGenerator, TrafficDistribution};

/// Padding mode for traffic analysis resistance
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaddingMode {
    /// Minimal padding for maximum performance
    Performance,
    /// Random padding class selection
    Privacy,
    /// Match typical HTTPS traffic patterns
    Stealth,
}

/// Timing mode for inter-packet delay
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimingMode {
    /// No delay, maximum throughput
    LowLatency,
    /// Moderate jitter
    Moderate,
    /// Match HTTPS timing patterns
    HighPrivacy,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_padding_mode_debug() {
        // Verify Debug trait implementation
        assert_eq!(format!("{:?}", PaddingMode::Performance), "Performance");
        assert_eq!(format!("{:?}", PaddingMode::Privacy), "Privacy");
        assert_eq!(format!("{:?}", PaddingMode::Stealth), "Stealth");
    }

    #[test]
    fn test_padding_mode_clone() {
        let mode = PaddingMode::Performance;
        let cloned = mode;
        assert_eq!(mode, cloned);
    }

    #[test]
    fn test_padding_mode_equality() {
        assert_eq!(PaddingMode::Performance, PaddingMode::Performance);
        assert_eq!(PaddingMode::Privacy, PaddingMode::Privacy);
        assert_eq!(PaddingMode::Stealth, PaddingMode::Stealth);

        assert_ne!(PaddingMode::Performance, PaddingMode::Privacy);
        assert_ne!(PaddingMode::Performance, PaddingMode::Stealth);
        assert_ne!(PaddingMode::Privacy, PaddingMode::Stealth);
    }

    #[test]
    fn test_padding_mode_all_variants() {
        // Ensure all variants can be constructed
        let _perf = PaddingMode::Performance;
        let _priv = PaddingMode::Privacy;
        let _stealth = PaddingMode::Stealth;
    }

    #[test]
    fn test_timing_mode_debug() {
        // Verify Debug trait implementation
        assert_eq!(format!("{:?}", TimingMode::LowLatency), "LowLatency");
        assert_eq!(format!("{:?}", TimingMode::Moderate), "Moderate");
        assert_eq!(format!("{:?}", TimingMode::HighPrivacy), "HighPrivacy");
    }

    #[test]
    fn test_timing_mode_clone() {
        let mode = TimingMode::LowLatency;
        let cloned = mode;
        assert_eq!(mode, cloned);
    }

    #[test]
    fn test_timing_mode_equality() {
        assert_eq!(TimingMode::LowLatency, TimingMode::LowLatency);
        assert_eq!(TimingMode::Moderate, TimingMode::Moderate);
        assert_eq!(TimingMode::HighPrivacy, TimingMode::HighPrivacy);

        assert_ne!(TimingMode::LowLatency, TimingMode::Moderate);
        assert_ne!(TimingMode::LowLatency, TimingMode::HighPrivacy);
        assert_ne!(TimingMode::Moderate, TimingMode::HighPrivacy);
    }

    #[test]
    fn test_timing_mode_all_variants() {
        // Ensure all variants can be constructed
        let _low = TimingMode::LowLatency;
        let _mod = TimingMode::Moderate;
        let _high = TimingMode::HighPrivacy;
    }

    #[test]
    fn test_padding_mode_copy() {
        let mode1 = PaddingMode::Performance;
        let mode2 = mode1; // Copy
        assert_eq!(mode1, mode2);
        // mode1 is still valid after copy
        assert_eq!(mode1, PaddingMode::Performance);
    }

    #[test]
    fn test_timing_mode_copy() {
        let mode1 = TimingMode::LowLatency;
        let mode2 = mode1; // Copy
        assert_eq!(mode1, mode2);
        // mode1 is still valid after copy
        assert_eq!(mode1, TimingMode::LowLatency);
    }
}
