//! TLS 1.3 protocol mimicry.
//!
//! Wraps WRAITH packets to look like TLS 1.3 application data,
//! allowing traffic to pass DPI inspection and blend with HTTPS traffic.

use rand::Rng;

/// TLS content type: Application Data
const TLS_CONTENT_TYPE_APPLICATION_DATA: u8 = 23;
/// TLS content type: Handshake
const TLS_CONTENT_TYPE_HANDSHAKE: u8 = 22;
/// Legacy TLS version in TLS 1.3 records
const TLS_VERSION_1_2: u16 = 0x0303;

/// TLS record layer wrapper
///
/// Wraps WRAITH packets in TLS 1.3 record format.
///
/// # Examples
///
/// ```
/// use wraith_obfuscation::tls_mimicry::TlsRecordWrapper;
///
/// let mut wrapper = TlsRecordWrapper::new();
/// let payload = b"hello world";
/// let record = wrapper.wrap(payload);
///
/// let unwrapped = wrapper.unwrap(&record).unwrap();
/// assert_eq!(unwrapped, payload);
/// ```
pub struct TlsRecordWrapper {
    sequence_number: u64,
}

impl TlsRecordWrapper {
    /// Create a new TLS record wrapper
    #[must_use]
    pub const fn new() -> Self {
        Self { sequence_number: 0 }
    }

    /// Wrap payload in TLS record
    ///
    /// Creates a TLS 1.3 application data record containing the payload.
    ///
    /// # Examples
    ///
    /// ```
    /// use wraith_obfuscation::tls_mimicry::TlsRecordWrapper;
    ///
    /// let mut wrapper = TlsRecordWrapper::new();
    /// let record = wrapper.wrap(b"test");
    /// assert_eq!(record[0], 23); // Application Data type
    /// ```
    pub fn wrap(&mut self, payload: &[u8]) -> Vec<u8> {
        let mut record = Vec::with_capacity(5 + payload.len());

        // TLS Record Header (5 bytes)
        record.push(TLS_CONTENT_TYPE_APPLICATION_DATA); // Content Type
        record.extend_from_slice(&TLS_VERSION_1_2.to_be_bytes()); // Legacy version
        record.extend_from_slice(&(payload.len() as u16).to_be_bytes()); // Length

        // Payload (encrypted in real TLS, our already-encrypted data)
        record.extend_from_slice(payload);

        self.sequence_number += 1;

        record
    }

    /// Unwrap TLS record to get payload
    ///
    /// Extracts the original payload from a TLS record.
    ///
    /// # Errors
    ///
    /// Returns `TlsError` if the record is malformed or incomplete.
    ///
    /// # Examples
    ///
    /// ```
    /// use wraith_obfuscation::tls_mimicry::TlsRecordWrapper;
    ///
    /// let mut wrapper = TlsRecordWrapper::new();
    /// let original = b"test data";
    /// let record = wrapper.wrap(original);
    ///
    /// let unwrapped = wrapper.unwrap(&record).unwrap();
    /// assert_eq!(unwrapped, original);
    /// ```
    pub fn unwrap(&self, record: &[u8]) -> Result<Vec<u8>, TlsError> {
        if record.len() < 5 {
            return Err(TlsError::TooShort);
        }

        // Parse TLS record header
        let content_type = record[0];
        let _version = u16::from_be_bytes([record[1], record[2]]);
        let length = u16::from_be_bytes([record[3], record[4]]) as usize;

        if content_type != TLS_CONTENT_TYPE_APPLICATION_DATA {
            return Err(TlsError::InvalidContentType);
        }

        if record.len() < 5 + length {
            return Err(TlsError::IncompleteRecord);
        }

        // Extract payload
        let payload = record[5..5 + length].to_vec();

        Ok(payload)
    }

    /// Get current sequence number
    #[must_use]
    pub const fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    /// Reset sequence number
    pub fn reset(&mut self) {
        self.sequence_number = 0;
    }
}

impl Default for TlsRecordWrapper {
    fn default() -> Self {
        Self::new()
    }
}

/// Full TLS session mimicry (including handshake simulation)
///
/// Generates fake TLS handshake messages to establish a session,
/// then wraps application data in TLS records.
///
/// # Examples
///
/// ```
/// use wraith_obfuscation::tls_mimicry::TlsSessionMimicry;
///
/// let mut session = TlsSessionMimicry::new();
/// let handshake = session.generate_handshake();
/// assert_eq!(handshake.len(), 3);
/// ```
pub struct TlsSessionMimicry {
    handshake_complete: bool,
    wrapper: TlsRecordWrapper,
}

impl TlsSessionMimicry {
    /// Create a new TLS session mimicry
    #[must_use]
    pub const fn new() -> Self {
        Self {
            handshake_complete: false,
            wrapper: TlsRecordWrapper::new(),
        }
    }

    /// Generate fake TLS handshake messages
    ///
    /// Returns a sequence of fake handshake messages that mimic
    /// a TLS 1.3 handshake.
    ///
    /// # Examples
    ///
    /// ```
    /// use wraith_obfuscation::tls_mimicry::TlsSessionMimicry;
    ///
    /// let mut session = TlsSessionMimicry::new();
    /// let messages = session.generate_handshake();
    /// assert_eq!(messages.len(), 3); // ClientHello, ServerHello, Finished
    /// ```
    pub fn generate_handshake(&mut self) -> Vec<Vec<u8>> {
        let messages = vec![
            // ClientHello
            self.fake_client_hello(),
            // ServerHello + Certificate + ... + Finished
            self.fake_server_hello(),
            // ClientFinished
            self.fake_client_finished(),
        ];

        self.handshake_complete = true;

        messages
    }

    fn fake_client_hello(&self) -> Vec<u8> {
        // Simplified ClientHello structure
        let mut hello = Vec::new();

        // Record header
        hello.push(TLS_CONTENT_TYPE_HANDSHAKE); // Handshake content type
        hello.extend_from_slice(&TLS_VERSION_1_2.to_be_bytes());

        // We'll calculate and set length later
        let length_offset = hello.len();
        hello.extend_from_slice(&[0x00, 0x00]); // Placeholder for length

        // Handshake header
        hello.push(1); // ClientHello type

        let handshake_length_offset = hello.len();
        hello.extend_from_slice(&[0x00, 0x00, 0x00]); // Placeholder for handshake length

        // TLS version in handshake
        hello.extend_from_slice(&TLS_VERSION_1_2.to_be_bytes());

        // Random (32 bytes)
        let mut rng = rand::thread_rng();
        let mut random = [0u8; 32];
        rng.fill(&mut random[..]);
        hello.extend_from_slice(&random);

        // Session ID (empty)
        hello.push(0);

        // Cipher suites
        hello.extend_from_slice(&[0x00, 0x04]); // Length: 4 bytes (2 ciphers)
        hello.extend_from_slice(&[0x13, 0x01]); // TLS_AES_128_GCM_SHA256
        hello.extend_from_slice(&[0x13, 0x02]); // TLS_AES_256_GCM_SHA384

        // Compression methods
        hello.push(1); // Length
        hello.push(0); // No compression

        // Extensions (simplified - just supported_versions)
        let extensions_start = hello.len();
        hello.extend_from_slice(&[0x00, 0x00]); // Placeholder for extensions length

        // supported_versions extension
        hello.extend_from_slice(&[0x00, 0x2b]); // Type: supported_versions
        hello.extend_from_slice(&[0x00, 0x03]); // Length: 3
        hello.push(0x02); // Versions length: 2
        hello.extend_from_slice(&[0x03, 0x04]); // TLS 1.3

        // Calculate and set extensions length
        let extensions_len = (hello.len() - extensions_start - 2) as u16;
        hello[extensions_start..extensions_start + 2]
            .copy_from_slice(&extensions_len.to_be_bytes());

        // Calculate and set handshake length
        let handshake_len = (hello.len() - handshake_length_offset - 3) as u32;
        hello[handshake_length_offset..handshake_length_offset + 3].copy_from_slice(&[
            ((handshake_len >> 16) & 0xFF) as u8,
            ((handshake_len >> 8) & 0xFF) as u8,
            (handshake_len & 0xFF) as u8,
        ]);

        // Calculate and set record length
        let record_len = (hello.len() - length_offset - 2) as u16;
        hello[length_offset..length_offset + 2].copy_from_slice(&record_len.to_be_bytes());

        hello
    }

    fn fake_server_hello(&self) -> Vec<u8> {
        // Simplified ServerHello
        let mut hello = Vec::new();

        hello.push(TLS_CONTENT_TYPE_HANDSHAKE);
        hello.extend_from_slice(&TLS_VERSION_1_2.to_be_bytes());
        hello.extend_from_slice(&[0x00, 0x5a]); // Length (placeholder)

        // Simplified handshake message
        let mut rng = rand::thread_rng();
        let mut payload = vec![0u8; 90];
        rng.fill(&mut payload[..]);

        hello.extend_from_slice(&payload);

        hello
    }

    fn fake_client_finished(&self) -> Vec<u8> {
        // Simplified Finished message
        let mut finished = Vec::new();

        finished.push(TLS_CONTENT_TYPE_HANDSHAKE);
        finished.extend_from_slice(&TLS_VERSION_1_2.to_be_bytes());
        finished.extend_from_slice(&[0x00, 0x35]); // Length (placeholder)

        // Simplified handshake message
        let mut rng = rand::thread_rng();
        let mut payload = vec![0u8; 53];
        rng.fill(&mut payload[..]);

        finished.extend_from_slice(&payload);

        finished
    }

    /// Wrap application data (after handshake)
    ///
    /// # Errors
    ///
    /// Returns `TlsError::HandshakeNotComplete` if handshake hasn't been generated.
    ///
    /// # Examples
    ///
    /// ```
    /// use wraith_obfuscation::tls_mimicry::TlsSessionMimicry;
    ///
    /// let mut session = TlsSessionMimicry::new();
    /// session.generate_handshake();
    ///
    /// let wrapped = session.wrap_application_data(b"test").unwrap();
    /// assert!(wrapped.len() > 5);
    /// ```
    pub fn wrap_application_data(&mut self, data: &[u8]) -> Result<Vec<u8>, TlsError> {
        if !self.handshake_complete {
            return Err(TlsError::HandshakeNotComplete);
        }

        Ok(self.wrapper.wrap(data))
    }

    /// Unwrap application data
    ///
    /// # Errors
    ///
    /// Returns `TlsError::HandshakeNotComplete` if handshake hasn't been generated,
    /// or forwarded errors from record unwrapping.
    pub fn unwrap_application_data(&self, record: &[u8]) -> Result<Vec<u8>, TlsError> {
        if !self.handshake_complete {
            return Err(TlsError::HandshakeNotComplete);
        }

        self.wrapper.unwrap(record)
    }

    /// Check if handshake is complete
    #[must_use]
    pub const fn is_handshake_complete(&self) -> bool {
        self.handshake_complete
    }

    /// Reset the session
    pub fn reset(&mut self) {
        self.handshake_complete = false;
        self.wrapper.reset();
    }
}

impl Default for TlsSessionMimicry {
    fn default() -> Self {
        Self::new()
    }
}

/// TLS error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsError {
    /// Record too short to parse
    TooShort,
    /// Invalid content type
    InvalidContentType,
    /// Incomplete record
    IncompleteRecord,
    /// Handshake not complete
    HandshakeNotComplete,
}

impl std::fmt::Display for TlsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooShort => write!(f, "TLS record too short"),
            Self::InvalidContentType => write!(f, "Invalid TLS content type"),
            Self::IncompleteRecord => write!(f, "Incomplete TLS record"),
            Self::HandshakeNotComplete => write!(f, "TLS handshake not complete"),
        }
    }
}

impl std::error::Error for TlsError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tls_record_wrap_unwrap() {
        let mut wrapper = TlsRecordWrapper::new();
        let payload = b"hello world";

        let record = wrapper.wrap(payload);

        // Check record structure
        assert_eq!(record[0], TLS_CONTENT_TYPE_APPLICATION_DATA);
        assert_eq!(record.len(), 5 + payload.len());

        let unwrapped = wrapper.unwrap(&record).unwrap();
        assert_eq!(unwrapped, payload);
    }

    #[test]
    fn test_tls_record_sequence_number() {
        let mut wrapper = TlsRecordWrapper::new();

        assert_eq!(wrapper.sequence_number(), 0);

        wrapper.wrap(b"test");
        assert_eq!(wrapper.sequence_number(), 1);

        wrapper.wrap(b"test2");
        assert_eq!(wrapper.sequence_number(), 2);
    }

    #[test]
    fn test_tls_record_reset() {
        let mut wrapper = TlsRecordWrapper::new();

        wrapper.wrap(b"test");
        assert_eq!(wrapper.sequence_number(), 1);

        wrapper.reset();
        assert_eq!(wrapper.sequence_number(), 0);
    }

    #[test]
    fn test_tls_record_default() {
        let wrapper = TlsRecordWrapper::default();
        assert_eq!(wrapper.sequence_number(), 0);
    }

    #[test]
    fn test_tls_record_too_short() {
        let wrapper = TlsRecordWrapper::new();
        let short_record = [0x17, 0x03, 0x03];

        assert!(matches!(
            wrapper.unwrap(&short_record),
            Err(TlsError::TooShort)
        ));
    }

    #[test]
    fn test_tls_record_invalid_content_type() {
        let wrapper = TlsRecordWrapper::new();
        let invalid_record = [0x14, 0x03, 0x03, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00];

        assert!(matches!(
            wrapper.unwrap(&invalid_record),
            Err(TlsError::InvalidContentType)
        ));
    }

    #[test]
    fn test_tls_record_incomplete() {
        let wrapper = TlsRecordWrapper::new();
        // Says length is 10 but only provides 5 bytes
        let incomplete = [0x17, 0x03, 0x03, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00];

        assert!(matches!(
            wrapper.unwrap(&incomplete),
            Err(TlsError::IncompleteRecord)
        ));
    }

    #[test]
    fn test_tls_handshake_generation() {
        let mut session = TlsSessionMimicry::new();

        assert!(!session.is_handshake_complete());

        let handshake_msgs = session.generate_handshake();
        assert_eq!(handshake_msgs.len(), 3); // ClientHello, ServerHello, Finished

        assert!(session.is_handshake_complete());
    }

    #[test]
    fn test_client_hello_structure() {
        let session = TlsSessionMimicry::new();
        let client_hello = session.fake_client_hello();

        // Should start with handshake content type
        assert_eq!(client_hello[0], TLS_CONTENT_TYPE_HANDSHAKE);

        // Should have TLS 1.2 version (legacy)
        assert_eq!(client_hello[1], 0x03);
        assert_eq!(client_hello[2], 0x03);

        // Should be long enough for a real ClientHello
        assert!(client_hello.len() > 50);
    }

    #[test]
    fn test_application_data_wrapping() {
        let mut session = TlsSessionMimicry::new();

        // Should fail before handshake
        assert!(matches!(
            session.wrap_application_data(b"test"),
            Err(TlsError::HandshakeNotComplete)
        ));

        session.generate_handshake();

        // Should succeed after handshake
        let wrapped = session.wrap_application_data(b"test").unwrap();
        assert!(wrapped.len() > 5);

        let unwrapped = session.unwrap_application_data(&wrapped).unwrap();
        assert_eq!(unwrapped, b"test");
    }

    #[test]
    fn test_session_reset() {
        let mut session = TlsSessionMimicry::new();

        session.generate_handshake();
        assert!(session.is_handshake_complete());

        session.reset();
        assert!(!session.is_handshake_complete());
    }

    #[test]
    fn test_session_default() {
        let session = TlsSessionMimicry::default();
        assert!(!session.is_handshake_complete());
    }

    #[test]
    fn test_tls_error_display() {
        assert_eq!(format!("{}", TlsError::TooShort), "TLS record too short");
        assert_eq!(
            format!("{}", TlsError::InvalidContentType),
            "Invalid TLS content type"
        );
        assert_eq!(
            format!("{}", TlsError::IncompleteRecord),
            "Incomplete TLS record"
        );
        assert_eq!(
            format!("{}", TlsError::HandshakeNotComplete),
            "TLS handshake not complete"
        );
    }

    #[test]
    fn test_multiple_wraps() {
        let mut wrapper = TlsRecordWrapper::new();

        for i in 0..10 {
            let payload = format!("message {}", i);
            let record = wrapper.wrap(payload.as_bytes());
            let unwrapped = wrapper.unwrap(&record).unwrap();

            assert_eq!(unwrapped, payload.as_bytes());
        }

        assert_eq!(wrapper.sequence_number(), 10);
    }

    #[test]
    fn test_large_payload() {
        let mut wrapper = TlsRecordWrapper::new();
        let large_payload = vec![0x42; 10000];

        let record = wrapper.wrap(&large_payload);
        let unwrapped = wrapper.unwrap(&record).unwrap();

        assert_eq!(unwrapped, large_payload);
    }

    #[test]
    fn test_empty_payload() {
        let mut wrapper = TlsRecordWrapper::new();
        let empty: &[u8] = &[];

        let record = wrapper.wrap(empty);
        assert_eq!(record.len(), 5); // Just header

        let unwrapped = wrapper.unwrap(&record).unwrap();
        assert_eq!(unwrapped.len(), 0);
    }

    #[test]
    fn test_handshake_messages_different() {
        let session = TlsSessionMimicry::new();

        let hello1 = session.fake_client_hello();
        let hello2 = session.fake_client_hello();

        // Random values should make them different
        assert_ne!(hello1, hello2);
    }
}
