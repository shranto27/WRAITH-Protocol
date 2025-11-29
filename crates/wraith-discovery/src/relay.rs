//! DERP-style relay network.

/// Relay client for NAT traversal
pub struct RelayClient {
    // TODO: Implement
    _private: (),
}

impl RelayClient {
    /// Connect to a relay server
    pub async fn connect(_url: &str) -> Result<Self, std::io::Error> {
        Ok(Self { _private: () })
    }
}
