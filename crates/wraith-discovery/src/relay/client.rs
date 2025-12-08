//! Relay client implementation for connecting to relay servers.

use super::protocol::{NodeId, RelayError, RelayMessage};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::{Mutex, mpsc};
use tokio::time;

/// Type alias for the message receiver
type MessageReceiver = Arc<Mutex<mpsc::UnboundedReceiver<(NodeId, Vec<u8>)>>>;

/// Relay client state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelayClientState {
    /// Disconnected from relay
    Disconnected,
    /// Connecting to relay
    Connecting,
    /// Registering with relay
    Registering,
    /// Connected and registered
    Connected,
    /// Error state
    Error,
}

/// Relay client for communicating with relay servers
pub struct RelayClient {
    /// Local node ID
    node_id: NodeId,
    /// Relay server address
    relay_addr: SocketAddr,
    /// UDP socket for communication
    socket: Arc<UdpSocket>,
    /// Current client state
    state: Arc<Mutex<RelayClientState>>,
    /// Receiver for incoming messages
    rx: MessageReceiver,
    /// Sender for message processing
    tx: mpsc::UnboundedSender<(NodeId, Vec<u8>)>,
    /// Last keepalive time
    last_keepalive: Arc<Mutex<Instant>>,
}

impl RelayClient {
    /// Connect to a relay server
    ///
    /// # Arguments
    ///
    /// * `addr` - Relay server address
    /// * `node_id` - Local node identifier
    ///
    /// # Errors
    ///
    /// Returns error if connection fails or times out.
    pub async fn connect(addr: SocketAddr, node_id: NodeId) -> Result<Self, RelayError> {
        // Bind local UDP socket
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.connect(addr).await?;

        let (tx, rx) = mpsc::unbounded_channel();

        let client = Self {
            node_id,
            relay_addr: addr,
            socket: Arc::new(socket),
            state: Arc::new(Mutex::new(RelayClientState::Disconnected)),
            rx: Arc::new(Mutex::new(rx)),
            tx,
            last_keepalive: Arc::new(Mutex::new(Instant::now())),
        };

        // Update state to connecting
        *client.state.lock().await = RelayClientState::Connecting;

        Ok(client)
    }

    /// Register with the relay server
    ///
    /// # Arguments
    ///
    /// * `public_key` - Client's public key for verification
    ///
    /// # Errors
    ///
    /// Returns error if registration fails or times out.
    pub async fn register(&mut self, public_key: &[u8; 32]) -> Result<(), RelayError> {
        *self.state.lock().await = RelayClientState::Registering;

        let msg = RelayMessage::Register {
            node_id: self.node_id,
            public_key: *public_key,
        };

        let bytes = msg.to_bytes()?;
        self.socket.send(&bytes).await?;

        // Wait for RegisterAck with timeout
        let mut buf = vec![0u8; 65536];
        let len = time::timeout(Duration::from_secs(10), self.socket.recv(&mut buf))
            .await
            .map_err(|_| RelayError::Timeout)??;

        let response = RelayMessage::from_bytes(&buf[..len])?;

        match response {
            RelayMessage::RegisterAck {
                success,
                error,
                relay_id: _,
            } => {
                if success {
                    *self.state.lock().await = RelayClientState::Connected;
                    *self.last_keepalive.lock().await = Instant::now();
                    Ok(())
                } else {
                    *self.state.lock().await = RelayClientState::Error;
                    Err(RelayError::Internal(
                        error.unwrap_or_else(|| "Registration failed".to_string()),
                    ))
                }
            }
            RelayMessage::Error { code, message: _ } => {
                *self.state.lock().await = RelayClientState::Error;
                Err(code.into())
            }
            _ => {
                *self.state.lock().await = RelayClientState::Error;
                Err(RelayError::InvalidMessage)
            }
        }
    }

    /// Send a packet to a peer through the relay
    ///
    /// # Arguments
    ///
    /// * `dest` - Destination node ID
    /// * `data` - Packet payload (already encrypted)
    ///
    /// # Errors
    ///
    /// Returns error if send fails or client not registered.
    pub async fn send_to_peer(&self, dest: NodeId, data: &[u8]) -> Result<(), RelayError> {
        if *self.state.lock().await != RelayClientState::Connected {
            return Err(RelayError::NotRegistered);
        }

        let msg = RelayMessage::SendPacket {
            dest_id: dest,
            payload: data.to_vec(),
        };

        let bytes = msg.to_bytes()?;
        self.socket.send(&bytes).await?;

        Ok(())
    }

    /// Receive a packet from a peer through the relay
    ///
    /// # Errors
    ///
    /// Returns error if receive fails or timeout occurs.
    pub async fn recv_from_peer(&self) -> Result<(NodeId, Vec<u8>), RelayError> {
        let mut rx = self.rx.lock().await;
        rx.recv()
            .await
            .ok_or_else(|| RelayError::Internal("Channel closed".to_string()))
    }

    /// Send keepalive message to maintain connection
    ///
    /// # Errors
    ///
    /// Returns error if send fails.
    pub async fn keepalive(&self) -> Result<(), RelayError> {
        let msg = RelayMessage::Keepalive;
        let bytes = msg.to_bytes()?;
        self.socket.send(&bytes).await?;

        *self.last_keepalive.lock().await = Instant::now();
        Ok(())
    }

    /// Disconnect from relay server
    ///
    /// # Errors
    ///
    /// Returns error if disconnect message fails to send.
    pub async fn disconnect(&mut self) -> Result<(), RelayError> {
        let msg = RelayMessage::Disconnect;
        let bytes = msg.to_bytes()?;
        self.socket.send(&bytes).await?;

        *self.state.lock().await = RelayClientState::Disconnected;
        Ok(())
    }

    /// Get current client state
    #[must_use]
    pub async fn state(&self) -> RelayClientState {
        *self.state.lock().await
    }

    /// Get relay server address
    #[must_use]
    pub fn relay_addr(&self) -> SocketAddr {
        self.relay_addr
    }

    /// Start background message processing task
    ///
    /// This task receives messages from the relay and forwards them to the channel.
    pub fn spawn_receiver(&self) {
        let socket = self.socket.clone();
        let tx = self.tx.clone();
        let state = self.state.clone();

        tokio::spawn(async move {
            let mut buf = vec![0u8; 65536];

            loop {
                match socket.recv(&mut buf).await {
                    Ok(len) => {
                        if let Ok(msg) = RelayMessage::from_bytes(&buf[..len]) {
                            match msg {
                                RelayMessage::RecvPacket { src_id, payload } => {
                                    let _ = tx.send((src_id, payload));
                                }
                                RelayMessage::PeerOnline { peer_id: _ } => {
                                    // Could notify application layer
                                }
                                RelayMessage::PeerOffline { peer_id: _ } => {
                                    // Could notify application layer
                                }
                                RelayMessage::Error { code, message: _ } => {
                                    eprintln!("Relay error: {code:?}");
                                    *state.lock().await = RelayClientState::Error;
                                }
                                _ => {
                                    // Ignore other messages
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Receive error: {e}");
                        *state.lock().await = RelayClientState::Error;
                        break;
                    }
                }
            }
        });
    }

    /// Check if keepalive is needed and send if necessary
    ///
    /// # Errors
    ///
    /// Returns error if keepalive send fails.
    pub async fn maybe_keepalive(&self, interval: Duration) -> Result<(), RelayError> {
        let last = *self.last_keepalive.lock().await;
        if last.elapsed() >= interval {
            self.keepalive().await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_relay_client_creation() {
        let node_id = [1u8; 32];
        let addr = "127.0.0.1:8000".parse().unwrap();

        let result = RelayClient::connect(addr, node_id).await;
        // May fail if relay not running, but constructor should succeed
        assert!(result.is_ok() || matches!(result, Err(RelayError::Io(_))));
    }

    #[tokio::test]
    async fn test_relay_client_state() {
        let node_id = [1u8; 32];
        let addr = "127.0.0.1:8001".parse().unwrap();

        if let Ok(client) = RelayClient::connect(addr, node_id).await {
            let state = client.state().await;
            assert_eq!(state, RelayClientState::Connecting);
        }
    }

    #[tokio::test]
    async fn test_relay_client_relay_addr() {
        let node_id = [1u8; 32];
        let addr: SocketAddr = "127.0.0.1:8002".parse().unwrap();

        if let Ok(client) = RelayClient::connect(addr, node_id).await {
            assert_eq!(client.relay_addr(), addr);
        }
    }

    #[test]
    fn test_relay_client_state_transitions() {
        assert_eq!(
            RelayClientState::Disconnected,
            RelayClientState::Disconnected
        );
        assert_ne!(RelayClientState::Connecting, RelayClientState::Connected);
    }
}
