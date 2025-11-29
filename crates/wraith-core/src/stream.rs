//! Stream multiplexing within sessions.
//!
//! Streams are logical bidirectional byte channels within a session,
//! used for individual file transfers.

/// A multiplexed stream within a session
pub struct Stream {
    id: u16,
    window: u64,
    // TODO: Add stream implementation fields
}

impl Stream {
    /// Create a new stream with the given ID and initial window
    pub fn new(id: u16, initial_window: u64) -> Self {
        Self {
            id,
            window: initial_window,
        }
    }

    /// Get the stream ID
    pub fn id(&self) -> u16 {
        self.id
    }

    /// Get the current flow control window
    pub fn window(&self) -> u64 {
        self.window
    }
}
