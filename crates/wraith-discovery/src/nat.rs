//! NAT traversal and hole punching.

/// NAT type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NatType {
    /// Any external host can send
    FullCone,
    /// Only contacted IPs can send
    AddressRestricted,
    /// Only contacted IP:port can send
    PortRestricted,
    /// Different mapping per destination
    Symmetric,
    /// No NAT detected
    None,
    /// Unknown NAT type
    Unknown,
}
