//! UDP socket fallback transport.

use std::net::SocketAddr;

/// UDP transport for systems without AF_XDP support
pub struct UdpTransport {
    // TODO: Implement
    _private: (),
}

impl UdpTransport {
    /// Create a new UDP transport bound to the given address
    pub fn bind(_addr: SocketAddr) -> std::io::Result<Self> {
        Ok(Self { _private: () })
    }
}
