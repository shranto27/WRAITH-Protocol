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

pub mod adaptive;
pub mod cover;
pub mod doh_tunnel;
pub mod padding;
pub mod timing;
pub mod tls_mimicry;
pub mod websocket_mimicry;

pub use adaptive::{MimicryMode, ObfuscationProfile, ThreatLevel};
pub use cover::{CoverTrafficGenerator, TrafficDistribution};
pub use doh_tunnel::{DohError, DohTunnel};
pub use padding::{PaddingEngine, PaddingMode};
pub use timing::{TimingMode, TimingObfuscator, TrafficShaper};
pub use tls_mimicry::{TlsError, TlsRecordWrapper, TlsSessionMimicry};
pub use websocket_mimicry::{WebSocketFrameWrapper, WsError};
