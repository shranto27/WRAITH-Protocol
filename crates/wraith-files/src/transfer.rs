//! Transfer state machine.

/// Transfer state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferState {
    /// Transfer not started
    Pending,
    /// Transfer in progress
    Active,
    /// Transfer paused (can resume)
    Paused,
    /// Transfer completed successfully
    Completed,
    /// Transfer failed
    Failed,
    /// Transfer cancelled
    Cancelled,
}

/// Transfer direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferDirection {
    /// Sending file
    Send,
    /// Receiving file
    Receive,
}
