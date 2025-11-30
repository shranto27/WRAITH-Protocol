//! Adaptive obfuscation profile selection.
//!
//! Automatically selects appropriate obfuscation strategies based on
//! threat level and performance requirements.

use crate::padding::PaddingMode;

/// Threat level assessment for obfuscation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreatLevel {
    /// No obfuscation needed - performance priority
    Low,
    /// Light obfuscation for casual observers
    Medium,
    /// Strong obfuscation for capable adversaries
    High,
    /// Maximum obfuscation regardless of cost
    Paranoid,
}

/// Protocol mimicry modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MimicryMode {
    /// Mimic TLS 1.3 traffic
    Tls,
    /// Mimic WebSocket traffic
    WebSocket,
    /// Mimic DNS-over-HTTPS traffic
    DnsOverHttps,
}

/// Complete obfuscation configuration profile
///
/// Provides a comprehensive set of obfuscation parameters based on
/// threat level assessment.
///
/// # Examples
///
/// ```
/// use wraith_obfuscation::adaptive::{ObfuscationProfile, ThreatLevel};
///
/// let profile = ObfuscationProfile::from_threat_level(ThreatLevel::High);
/// assert!(profile.timing_jitter);
/// assert!(profile.cover_traffic);
/// assert!(profile.protocol_mimicry.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct ObfuscationProfile {
    /// Padding mode to use
    pub padding_mode: PaddingMode,
    /// Enable timing jitter
    pub timing_jitter: bool,
    /// Enable cover traffic generation
    pub cover_traffic: bool,
    /// Optional protocol mimicry
    pub protocol_mimicry: Option<MimicryMode>,
}

impl ObfuscationProfile {
    /// Select profile based on threat level
    ///
    /// Automatically configures obfuscation parameters appropriate for
    /// the assessed threat level.
    ///
    /// # Examples
    ///
    /// ```
    /// use wraith_obfuscation::adaptive::{ObfuscationProfile, ThreatLevel};
    /// use wraith_obfuscation::padding::PaddingMode;
    ///
    /// let low = ObfuscationProfile::from_threat_level(ThreatLevel::Low);
    /// assert_eq!(low.padding_mode, PaddingMode::None);
    ///
    /// let high = ObfuscationProfile::from_threat_level(ThreatLevel::High);
    /// assert_eq!(high.padding_mode, PaddingMode::Statistical);
    /// ```
    #[must_use]
    pub fn from_threat_level(level: ThreatLevel) -> Self {
        match level {
            ThreatLevel::Low => Self {
                padding_mode: PaddingMode::None,
                timing_jitter: false,
                cover_traffic: false,
                protocol_mimicry: None,
            },

            ThreatLevel::Medium => Self {
                padding_mode: PaddingMode::SizeClasses,
                timing_jitter: true,
                cover_traffic: false,
                protocol_mimicry: None,
            },

            ThreatLevel::High => Self {
                padding_mode: PaddingMode::Statistical,
                timing_jitter: true,
                cover_traffic: true,
                protocol_mimicry: Some(MimicryMode::Tls),
            },

            ThreatLevel::Paranoid => Self {
                padding_mode: PaddingMode::ConstantRate,
                timing_jitter: true,
                cover_traffic: true,
                protocol_mimicry: Some(MimicryMode::Tls),
            },
        }
    }

    /// Create a custom profile
    ///
    /// # Examples
    ///
    /// ```
    /// use wraith_obfuscation::adaptive::{ObfuscationProfile, MimicryMode};
    /// use wraith_obfuscation::padding::PaddingMode;
    ///
    /// let profile = ObfuscationProfile::custom(
    ///     PaddingMode::SizeClasses,
    ///     true,
    ///     false,
    ///     Some(MimicryMode::WebSocket),
    /// );
    /// ```
    #[must_use]
    pub const fn custom(
        padding_mode: PaddingMode,
        timing_jitter: bool,
        cover_traffic: bool,
        protocol_mimicry: Option<MimicryMode>,
    ) -> Self {
        Self {
            padding_mode,
            timing_jitter,
            cover_traffic,
            protocol_mimicry,
        }
    }

    /// Estimate performance overhead
    ///
    /// Returns an estimated percentage overhead from all obfuscation
    /// features enabled in this profile.
    ///
    /// # Examples
    ///
    /// ```
    /// use wraith_obfuscation::adaptive::{ObfuscationProfile, ThreatLevel};
    ///
    /// let low = ObfuscationProfile::from_threat_level(ThreatLevel::Low);
    /// assert_eq!(low.estimated_overhead(), 0.0);
    ///
    /// let paranoid = ObfuscationProfile::from_threat_level(ThreatLevel::Paranoid);
    /// assert!(paranoid.estimated_overhead() > 50.0);
    /// ```
    #[must_use]
    pub fn estimated_overhead(&self) -> f64 {
        let mut overhead = 0.0;

        // Padding overhead
        overhead += match self.padding_mode {
            PaddingMode::None => 0.0,
            PaddingMode::PowerOfTwo => 15.0,
            PaddingMode::SizeClasses => 10.0,
            PaddingMode::ConstantRate => 50.0,
            PaddingMode::Statistical => 20.0,
        };

        // Timing overhead
        if self.timing_jitter {
            overhead += 5.0;
        }

        // Cover traffic overhead
        if self.cover_traffic {
            overhead += 25.0;
        }

        // Protocol mimicry overhead
        if self.protocol_mimicry.is_some() {
            overhead += 8.0;
        }

        overhead
    }

    /// Get recommended threat level for a given security requirement
    ///
    /// # Examples
    ///
    /// ```
    /// use wraith_obfuscation::adaptive::{ObfuscationProfile, ThreatLevel};
    ///
    /// let level = ObfuscationProfile::recommend_threat_level("public_wifi");
    /// assert!(matches!(level, ThreatLevel::Medium | ThreatLevel::High));
    /// ```
    #[must_use]
    pub fn recommend_threat_level(context: &str) -> ThreatLevel {
        match context {
            "local_network" | "trusted" => ThreatLevel::Low,
            "public_wifi" | "internet" => ThreatLevel::Medium,
            "censored_region" | "hostile" => ThreatLevel::High,
            "targeted_surveillance" => ThreatLevel::Paranoid,
            _ => ThreatLevel::Medium, // Default to medium
        }
    }

    /// Check if profile meets minimum security requirements
    ///
    /// # Examples
    ///
    /// ```
    /// use wraith_obfuscation::adaptive::{ObfuscationProfile, ThreatLevel};
    ///
    /// let profile = ObfuscationProfile::from_threat_level(ThreatLevel::High);
    /// assert!(profile.meets_minimum_security(ThreatLevel::Medium));
    /// assert!(!profile.meets_minimum_security(ThreatLevel::Paranoid));
    /// ```
    #[must_use]
    pub fn meets_minimum_security(&self, required: ThreatLevel) -> bool {
        let current_level = self.assess_security_level();
        (current_level as u8) >= (required as u8)
    }

    // Assess security level based on current configuration
    fn assess_security_level(&self) -> ThreatLevel {
        // Heuristic based on enabled features
        let padding_score = match self.padding_mode {
            PaddingMode::None => 0,
            PaddingMode::PowerOfTwo | PaddingMode::SizeClasses => 1,
            PaddingMode::Statistical => 2,
            PaddingMode::ConstantRate => 3,
        };

        let timing_score = if self.timing_jitter { 1 } else { 0 };
        let cover_score = if self.cover_traffic { 1 } else { 0 };
        let mimicry_score = if self.protocol_mimicry.is_some() {
            1
        } else {
            0
        };

        let total_score = padding_score + timing_score + cover_score + mimicry_score;

        match total_score {
            0 => ThreatLevel::Low,
            1..=2 => ThreatLevel::Medium,
            3..=5 => ThreatLevel::High,
            _ => ThreatLevel::Paranoid,
        }
    }
}

impl Default for ObfuscationProfile {
    fn default() -> Self {
        Self::from_threat_level(ThreatLevel::Medium)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threat_level_low() {
        let profile = ObfuscationProfile::from_threat_level(ThreatLevel::Low);
        assert_eq!(profile.padding_mode, PaddingMode::None);
        assert!(!profile.timing_jitter);
        assert!(!profile.cover_traffic);
        assert!(profile.protocol_mimicry.is_none());
    }

    #[test]
    fn test_threat_level_medium() {
        let profile = ObfuscationProfile::from_threat_level(ThreatLevel::Medium);
        assert_eq!(profile.padding_mode, PaddingMode::SizeClasses);
        assert!(profile.timing_jitter);
        assert!(!profile.cover_traffic);
        assert!(profile.protocol_mimicry.is_none());
    }

    #[test]
    fn test_threat_level_high() {
        let profile = ObfuscationProfile::from_threat_level(ThreatLevel::High);
        assert_eq!(profile.padding_mode, PaddingMode::Statistical);
        assert!(profile.timing_jitter);
        assert!(profile.cover_traffic);
        assert!(profile.protocol_mimicry.is_some());
        assert_eq!(profile.protocol_mimicry.unwrap(), MimicryMode::Tls);
    }

    #[test]
    fn test_threat_level_paranoid() {
        let profile = ObfuscationProfile::from_threat_level(ThreatLevel::Paranoid);
        assert_eq!(profile.padding_mode, PaddingMode::ConstantRate);
        assert!(profile.timing_jitter);
        assert!(profile.cover_traffic);
        assert!(profile.protocol_mimicry.is_some());
    }

    #[test]
    fn test_overhead_estimation_low() {
        let profile = ObfuscationProfile::from_threat_level(ThreatLevel::Low);
        assert_eq!(profile.estimated_overhead(), 0.0);
    }

    #[test]
    fn test_overhead_estimation_medium() {
        let profile = ObfuscationProfile::from_threat_level(ThreatLevel::Medium);
        let overhead = profile.estimated_overhead();
        assert!((overhead - 15.0).abs() < 1.0); // 10% padding + 5% timing
    }

    #[test]
    fn test_overhead_estimation_high() {
        let profile = ObfuscationProfile::from_threat_level(ThreatLevel::High);
        let overhead = profile.estimated_overhead();
        // 20% padding + 5% timing + 25% cover + 8% mimicry = 58%
        assert!((overhead - 58.0).abs() < 1.0);
    }

    #[test]
    fn test_overhead_estimation_paranoid() {
        let profile = ObfuscationProfile::from_threat_level(ThreatLevel::Paranoid);
        let overhead = profile.estimated_overhead();
        // 50% padding + 5% timing + 25% cover + 8% mimicry = 88%
        assert!((overhead - 88.0).abs() < 1.0);
    }

    #[test]
    fn test_custom_profile() {
        let profile = ObfuscationProfile::custom(
            PaddingMode::PowerOfTwo,
            true,
            false,
            Some(MimicryMode::WebSocket),
        );

        assert_eq!(profile.padding_mode, PaddingMode::PowerOfTwo);
        assert!(profile.timing_jitter);
        assert!(!profile.cover_traffic);
        assert_eq!(profile.protocol_mimicry.unwrap(), MimicryMode::WebSocket);
    }

    #[test]
    fn test_default_profile() {
        let profile = ObfuscationProfile::default();
        assert_eq!(profile.padding_mode, PaddingMode::SizeClasses);
        assert!(profile.timing_jitter);
    }

    #[test]
    fn test_recommend_threat_level() {
        assert_eq!(
            ObfuscationProfile::recommend_threat_level("local_network"),
            ThreatLevel::Low
        );
        assert_eq!(
            ObfuscationProfile::recommend_threat_level("trusted"),
            ThreatLevel::Low
        );
        assert_eq!(
            ObfuscationProfile::recommend_threat_level("public_wifi"),
            ThreatLevel::Medium
        );
        assert_eq!(
            ObfuscationProfile::recommend_threat_level("internet"),
            ThreatLevel::Medium
        );
        assert_eq!(
            ObfuscationProfile::recommend_threat_level("censored_region"),
            ThreatLevel::High
        );
        assert_eq!(
            ObfuscationProfile::recommend_threat_level("hostile"),
            ThreatLevel::High
        );
        assert_eq!(
            ObfuscationProfile::recommend_threat_level("targeted_surveillance"),
            ThreatLevel::Paranoid
        );
        assert_eq!(
            ObfuscationProfile::recommend_threat_level("unknown"),
            ThreatLevel::Medium
        );
    }

    #[test]
    fn test_meets_minimum_security() {
        let low_profile = ObfuscationProfile::from_threat_level(ThreatLevel::Low);
        let medium_profile = ObfuscationProfile::from_threat_level(ThreatLevel::Medium);
        let high_profile = ObfuscationProfile::from_threat_level(ThreatLevel::High);

        // Low profile doesn't meet medium requirements
        assert!(!low_profile.meets_minimum_security(ThreatLevel::Medium));
        assert!(low_profile.meets_minimum_security(ThreatLevel::Low));

        // Medium profile meets medium but not high
        assert!(medium_profile.meets_minimum_security(ThreatLevel::Medium));
        assert!(medium_profile.meets_minimum_security(ThreatLevel::Low));
        assert!(!medium_profile.meets_minimum_security(ThreatLevel::High));

        // High profile meets high and below
        assert!(high_profile.meets_minimum_security(ThreatLevel::High));
        assert!(high_profile.meets_minimum_security(ThreatLevel::Medium));
        assert!(high_profile.meets_minimum_security(ThreatLevel::Low));
    }

    #[test]
    fn test_mimicry_mode_variants() {
        let tls = MimicryMode::Tls;
        let ws = MimicryMode::WebSocket;
        let doh = MimicryMode::DnsOverHttps;

        assert_ne!(tls, ws);
        assert_ne!(tls, doh);
        assert_ne!(ws, doh);
    }

    #[test]
    fn test_threat_level_ordering() {
        // Ensure threat levels have expected ordering
        assert!((ThreatLevel::Low as u8) < (ThreatLevel::Medium as u8));
        assert!((ThreatLevel::Medium as u8) < (ThreatLevel::High as u8));
        assert!((ThreatLevel::High as u8) < (ThreatLevel::Paranoid as u8));
    }

    #[test]
    fn test_profile_clone() {
        let original = ObfuscationProfile::from_threat_level(ThreatLevel::High);
        let cloned = original.clone();

        assert_eq!(cloned.padding_mode, original.padding_mode);
        assert_eq!(cloned.timing_jitter, original.timing_jitter);
        assert_eq!(cloned.cover_traffic, original.cover_traffic);
        assert_eq!(cloned.protocol_mimicry, original.protocol_mimicry);
    }

    #[test]
    fn test_assess_security_level() {
        let low = ObfuscationProfile::from_threat_level(ThreatLevel::Low);
        assert_eq!(low.assess_security_level(), ThreatLevel::Low);

        let medium = ObfuscationProfile::from_threat_level(ThreatLevel::Medium);
        assert!(matches!(
            medium.assess_security_level(),
            ThreatLevel::Medium
        ));

        let high = ObfuscationProfile::from_threat_level(ThreatLevel::High);
        assert!(matches!(
            high.assess_security_level(),
            ThreatLevel::High | ThreatLevel::Paranoid
        ));
    }

    #[test]
    fn test_overhead_components() {
        // Test overhead calculation with individual components
        let no_padding = ObfuscationProfile::custom(PaddingMode::None, false, false, None);
        assert_eq!(no_padding.estimated_overhead(), 0.0);

        let only_padding = ObfuscationProfile::custom(PaddingMode::SizeClasses, false, false, None);
        assert_eq!(only_padding.estimated_overhead(), 10.0);

        let only_timing = ObfuscationProfile::custom(PaddingMode::None, true, false, None);
        assert_eq!(only_timing.estimated_overhead(), 5.0);

        let only_cover = ObfuscationProfile::custom(PaddingMode::None, false, true, None);
        assert_eq!(only_cover.estimated_overhead(), 25.0);

        let only_mimicry =
            ObfuscationProfile::custom(PaddingMode::None, false, false, Some(MimicryMode::Tls));
        assert_eq!(only_mimicry.estimated_overhead(), 8.0);
    }
}
