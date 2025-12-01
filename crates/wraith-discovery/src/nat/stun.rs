//! STUN Protocol Implementation (RFC 5389)
//!
//! This module implements the STUN (Session Traversal Utilities for NAT) protocol
//! for discovering server reflexive addresses and performing NAT type detection.

use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::UdpSocket;

/// STUN magic cookie (0x2112A442)
const MAGIC_COOKIE: u32 = 0x2112_A442;

/// STUN message header size (20 bytes)
const HEADER_SIZE: usize = 20;

/// Default STUN timeout
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(3);

/// STUN message class
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StunMessageClass {
    /// Request message
    Request,
    /// Success response
    SuccessResponse,
    /// Error response
    ErrorResponse,
    /// Indication (no response expected)
    Indication,
}

/// STUN message type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StunMessageType {
    /// Binding request/response
    Binding,
}

impl StunMessageType {
    /// Encode message type and class into a 16-bit value
    ///
    /// RFC 5389 Section 6 encoding:
    /// ```text
    ///  0                 1
    ///  2  3  4 5 6 7 8 9 0 1 2 3 4 5
    /// +--+--+-+-+-+-+-+-+-+-+-+-+-+-+
    /// |M |M |M|M|M|C|M|M|M|C|M|M|M|M|
    /// |11|10|9|8|7|1|6|5|4|0|3|2|1|0|
    /// +--+--+-+-+-+-+-+-+-+-+-+-+-+-+
    /// ```
    fn encode(&self, class: StunMessageClass) -> u16 {
        let method = match self {
            Self::Binding => 0x0001,
        };

        let class_bits = match class {
            StunMessageClass::Request => 0b00,
            StunMessageClass::Indication => 0b01,
            StunMessageClass::SuccessResponse => 0b10,
            StunMessageClass::ErrorResponse => 0b11,
        };

        // STUN message type encoding (RFC 5389 Section 6)
        // Bits 0-3: M0-M3
        let m0_m3 = method & 0x0F;
        // Bit 4: C0
        let c0 = (class_bits & 0x01) << 4;
        // Bits 5-7: M4-M6
        let m4_m6 = (method & 0x70) << 1;
        // Bit 8: C1
        let c1 = (class_bits & 0x02) << 7;
        // Bits 9-13: M7-M11
        let m7_m11 = (method & 0xF80) << 2;

        m0_m3 | c0 | m4_m6 | c1 | m7_m11
    }
}

/// STUN attribute types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StunAttribute {
    /// Mapped address (0x0001)
    MappedAddress(SocketAddr),
    /// XOR-Mapped address (0x0020) - preferred over MAPPED-ADDRESS
    XorMappedAddress(SocketAddr),
    /// Software identifier (0x8022)
    Software(String),
    /// Fingerprint (0x8028)
    Fingerprint(u32),
    /// Unknown attribute type
    Unknown(u16, Vec<u8>),
}

impl StunAttribute {
    /// Attribute type code
    fn attr_type(&self) -> u16 {
        match self {
            Self::MappedAddress(_) => 0x0001,
            Self::XorMappedAddress(_) => 0x0020,
            Self::Software(_) => 0x8022,
            Self::Fingerprint(_) => 0x8028,
            Self::Unknown(t, _) => *t,
        }
    }

    /// Encode attribute to bytes
    fn encode(&self, transaction_id: &[u8; 12]) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Attribute type (2 bytes)
        bytes.extend_from_slice(&self.attr_type().to_be_bytes());

        // Attribute value
        let value = self.encode_value(transaction_id);

        // Attribute length (2 bytes)
        bytes.extend_from_slice(&(value.len() as u16).to_be_bytes());

        // Attribute value
        bytes.extend_from_slice(&value);

        // Padding to 4-byte boundary
        let padding = (4 - (value.len() % 4)) % 4;
        bytes.extend(std::iter::repeat_n(0, padding));

        bytes
    }

    fn encode_value(&self, transaction_id: &[u8; 12]) -> Vec<u8> {
        match self {
            Self::XorMappedAddress(addr) => {
                let mut value = Vec::new();
                value.push(0); // Reserved
                value.push(if addr.is_ipv4() { 0x01 } else { 0x02 });

                // XOR port with most significant 16 bits of magic cookie
                let xor_port = addr.port() ^ (MAGIC_COOKIE >> 16) as u16;
                value.extend_from_slice(&xor_port.to_be_bytes());

                // XOR address with magic cookie (+ transaction ID for IPv6)
                match addr.ip() {
                    std::net::IpAddr::V4(ipv4) => {
                        let ip_bytes = ipv4.octets();
                        let magic_bytes = MAGIC_COOKIE.to_be_bytes();
                        for i in 0..4 {
                            value.push(ip_bytes[i] ^ magic_bytes[i]);
                        }
                    }
                    std::net::IpAddr::V6(ipv6) => {
                        let ip_bytes = ipv6.octets();
                        let mut xor_key = MAGIC_COOKIE.to_be_bytes().to_vec();
                        xor_key.extend_from_slice(transaction_id);
                        for i in 0..16 {
                            value.push(ip_bytes[i] ^ xor_key[i]);
                        }
                    }
                }

                value
            }
            Self::Software(s) => s.as_bytes().to_vec(),
            Self::Fingerprint(f) => f.to_be_bytes().to_vec(),
            Self::Unknown(_, data) => data.clone(),
            Self::MappedAddress(_) => Vec::new(), // Not implemented
        }
    }

    /// Decode attribute from bytes
    fn decode(attr_type: u16, value: &[u8], transaction_id: &[u8; 12]) -> Result<Self, StunError> {
        match attr_type {
            0x0020 => {
                // XOR-MAPPED-ADDRESS
                if value.len() < 4 {
                    return Err(StunError::InvalidAttribute);
                }

                let family = value[1];
                let xor_port = u16::from_be_bytes([value[2], value[3]]);
                let port = xor_port ^ (MAGIC_COOKIE >> 16) as u16;

                let addr = if family == 0x01 {
                    // IPv4
                    if value.len() < 8 {
                        return Err(StunError::InvalidAttribute);
                    }
                    let magic_bytes = MAGIC_COOKIE.to_be_bytes();
                    let mut ip_bytes = [0u8; 4];
                    for i in 0..4 {
                        ip_bytes[i] = value[4 + i] ^ magic_bytes[i];
                    }
                    let ip = std::net::Ipv4Addr::from(ip_bytes);
                    SocketAddr::new(ip.into(), port)
                } else {
                    // IPv6
                    if value.len() < 20 {
                        return Err(StunError::InvalidAttribute);
                    }
                    let mut xor_key = MAGIC_COOKIE.to_be_bytes().to_vec();
                    xor_key.extend_from_slice(transaction_id);
                    let mut ip_bytes = [0u8; 16];
                    for i in 0..16 {
                        ip_bytes[i] = value[4 + i] ^ xor_key[i];
                    }
                    let ip = std::net::Ipv6Addr::from(ip_bytes);
                    SocketAddr::new(ip.into(), port)
                };

                Ok(Self::XorMappedAddress(addr))
            }
            0x8022 => {
                // SOFTWARE
                let s = String::from_utf8_lossy(value).to_string();
                Ok(Self::Software(s))
            }
            _ => Ok(Self::Unknown(attr_type, value.to_vec())),
        }
    }
}

/// STUN message
#[derive(Debug, Clone)]
pub struct StunMessage {
    /// Message type
    pub message_type: StunMessageType,
    /// Message class
    pub message_class: StunMessageClass,
    /// Transaction ID (96 bits)
    pub transaction_id: [u8; 12],
    /// Message attributes
    pub attributes: Vec<StunAttribute>,
}

impl StunMessage {
    /// Create a new STUN Binding Request
    #[must_use]
    pub fn binding_request() -> Self {
        let mut transaction_id = [0u8; 12];
        use rand::RngCore;
        rand::thread_rng().fill_bytes(&mut transaction_id);

        Self {
            message_type: StunMessageType::Binding,
            message_class: StunMessageClass::Request,
            transaction_id,
            attributes: Vec::new(),
        }
    }

    /// Add an attribute to the message
    pub fn add_attribute(&mut self, attr: StunAttribute) {
        self.attributes.push(attr);
    }

    /// Encode message to bytes
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Message Type (2 bytes)
        let msg_type = self.message_type.encode(self.message_class);
        bytes.extend_from_slice(&msg_type.to_be_bytes());

        // Message Length (2 bytes) - placeholder
        let length_offset = bytes.len();
        bytes.extend_from_slice(&[0u8; 2]);

        // Magic Cookie (4 bytes)
        bytes.extend_from_slice(&MAGIC_COOKIE.to_be_bytes());

        // Transaction ID (12 bytes)
        bytes.extend_from_slice(&self.transaction_id);

        // Attributes
        for attr in &self.attributes {
            bytes.extend_from_slice(&attr.encode(&self.transaction_id));
        }

        // Update message length (excludes 20-byte header)
        let msg_length = bytes.len() - HEADER_SIZE;
        bytes[length_offset..length_offset + 2].copy_from_slice(&(msg_length as u16).to_be_bytes());

        bytes
    }

    /// Decode message from bytes
    pub fn decode(bytes: &[u8]) -> Result<Self, StunError> {
        if bytes.len() < HEADER_SIZE {
            return Err(StunError::MessageTooShort);
        }

        // Parse header
        let msg_type = u16::from_be_bytes([bytes[0], bytes[1]]);
        let msg_length = u16::from_be_bytes([bytes[2], bytes[3]]) as usize;
        let magic_cookie = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);

        if magic_cookie != MAGIC_COOKIE {
            return Err(StunError::InvalidMagicCookie);
        }

        let mut transaction_id = [0u8; 12];
        transaction_id.copy_from_slice(&bytes[8..20]);

        // Decode message type and class
        let (message_type, message_class) = Self::decode_type(msg_type)?;

        // Parse attributes
        let mut attributes = Vec::new();
        let mut offset = HEADER_SIZE;

        while offset < bytes.len() && offset - HEADER_SIZE < msg_length {
            if offset + 4 > bytes.len() {
                break;
            }

            let attr_type = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]);
            let attr_length = u16::from_be_bytes([bytes[offset + 2], bytes[offset + 3]]) as usize;

            offset += 4;

            if offset + attr_length > bytes.len() {
                break;
            }

            let attr_value = &bytes[offset..offset + attr_length];
            if let Ok(attr) = StunAttribute::decode(attr_type, attr_value, &transaction_id) {
                attributes.push(attr);
            }

            offset += attr_length;

            // Skip padding to 4-byte boundary
            let padding = (4 - (attr_length % 4)) % 4;
            offset += padding;
        }

        Ok(Self {
            message_type,
            message_class,
            transaction_id,
            attributes,
        })
    }

    fn decode_type(msg_type: u16) -> Result<(StunMessageType, StunMessageClass), StunError> {
        // Extract class bits (C0 at bit 4, C1 at bit 8)
        let c0 = (msg_type >> 4) & 0x01;
        let c1 = (msg_type >> 8) & 0x01;
        let class_bits = c0 | (c1 << 1);

        let message_class = match class_bits {
            0b00 => StunMessageClass::Request,
            0b01 => StunMessageClass::Indication,
            0b10 => StunMessageClass::SuccessResponse,
            0b11 => StunMessageClass::ErrorResponse,
            _ => return Err(StunError::InvalidMessageType),
        };

        // Extract method bits
        let m0_m3 = msg_type & 0x0F;
        let m4_m6 = (msg_type >> 1) & 0x70;
        let m7_m11 = (msg_type >> 2) & 0xF80;
        let method = m0_m3 | m4_m6 | m7_m11;

        let message_type = match method {
            0x0001 => StunMessageType::Binding,
            _ => return Err(StunError::InvalidMessageType),
        };

        Ok((message_type, message_class))
    }

    /// Get XOR-MAPPED-ADDRESS attribute
    #[must_use]
    pub fn xor_mapped_address(&self) -> Option<SocketAddr> {
        for attr in &self.attributes {
            if let StunAttribute::XorMappedAddress(addr) = attr {
                return Some(*addr);
            }
        }
        None
    }
}

/// STUN client for server reflexive address discovery
pub struct StunClient {
    socket: UdpSocket,
    timeout: Duration,
}

impl StunClient {
    /// Bind a new STUN client to a local address
    ///
    /// # Errors
    ///
    /// Returns an error if the socket cannot be bound
    pub async fn bind(addr: &str) -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind(addr).await?;
        Ok(Self {
            socket,
            timeout: DEFAULT_TIMEOUT,
        })
    }

    /// Set query timeout
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    /// Get local socket address
    ///
    /// # Errors
    ///
    /// Returns an error if the local address cannot be determined
    pub fn local_addr(&self) -> Result<SocketAddr, std::io::Error> {
        self.socket.local_addr()
    }

    /// Get mapped address from STUN server
    ///
    /// # Errors
    ///
    /// Returns `StunError` if:
    /// - Network I/O fails
    /// - STUN server doesn't respond within timeout
    /// - Response is invalid or missing XOR-MAPPED-ADDRESS
    pub async fn get_mapped_address(&self, server: SocketAddr) -> Result<SocketAddr, StunError> {
        // Create binding request
        let request = StunMessage::binding_request();
        let request_bytes = request.encode();
        let transaction_id = request.transaction_id;

        // Send request
        self.socket.send_to(&request_bytes, server).await?;

        // Receive response with timeout
        let mut buf = [0u8; 1024];

        let (len, _from) = tokio::time::timeout(self.timeout, self.socket.recv_from(&mut buf))
            .await
            .map_err(|_| StunError::Timeout)??;

        // Decode response
        let response = StunMessage::decode(&buf[..len])?;

        // Verify transaction ID
        if response.transaction_id != transaction_id {
            return Err(StunError::TransactionMismatch);
        }

        // Verify response type
        if response.message_class != StunMessageClass::SuccessResponse {
            return Err(StunError::ErrorResponse);
        }

        // Extract XOR-MAPPED-ADDRESS
        response
            .xor_mapped_address()
            .ok_or(StunError::MissingAttribute)
    }
}

/// STUN error types
#[derive(Debug)]
pub enum StunError {
    /// I/O error
    Io(std::io::Error),
    /// Query timeout
    Timeout,
    /// Invalid message format
    MessageTooShort,
    /// Invalid magic cookie
    InvalidMagicCookie,
    /// Invalid message type
    InvalidMessageType,
    /// Invalid attribute
    InvalidAttribute,
    /// Transaction ID mismatch
    TransactionMismatch,
    /// Error response received
    ErrorResponse,
    /// Missing required attribute
    MissingAttribute,
}

impl std::fmt::Display for StunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Timeout => write!(f, "STUN query timeout"),
            Self::MessageTooShort => write!(f, "STUN message too short"),
            Self::InvalidMagicCookie => write!(f, "Invalid STUN magic cookie"),
            Self::InvalidMessageType => write!(f, "Invalid STUN message type"),
            Self::InvalidAttribute => write!(f, "Invalid STUN attribute"),
            Self::TransactionMismatch => write!(f, "Transaction ID mismatch"),
            Self::ErrorResponse => write!(f, "STUN error response"),
            Self::MissingAttribute => write!(f, "Missing required STUN attribute"),
        }
    }
}

impl std::error::Error for StunError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for StunError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stun_message_type_encoding() {
        // Binding Request: method=0x0001, class=Request(0b00)
        let encoded = StunMessageType::Binding.encode(StunMessageClass::Request);
        assert_eq!(encoded, 0x0001);

        // Binding Success Response: method=0x0001, class=SuccessResponse(0b10)
        // Class bits: C0=0 (bit 4), C1=1 (bit 8)
        // Result: 0x0001 | (0 << 4) | (1 << 8) = 0x0001 | 0x0100 = 0x0101
        let encoded = StunMessageType::Binding.encode(StunMessageClass::SuccessResponse);
        assert_eq!(encoded, 0x0101);
    }

    #[test]
    fn test_stun_message_roundtrip() {
        let msg = StunMessage::binding_request();
        let encoded = msg.encode();

        assert!(encoded.len() >= HEADER_SIZE);

        let decoded = StunMessage::decode(&encoded).unwrap();
        assert_eq!(decoded.message_type, msg.message_type);
        assert_eq!(decoded.message_class, msg.message_class);
        assert_eq!(decoded.transaction_id, msg.transaction_id);
    }

    #[test]
    fn test_xor_mapped_address_attribute() {
        let addr: SocketAddr = "192.0.2.1:32853".parse().unwrap();
        let transaction_id = [0u8; 12];

        let attr = StunAttribute::XorMappedAddress(addr);
        let encoded = attr.encode(&transaction_id);

        // Decode it back
        let value = &encoded[4..]; // Skip type and length
        let decoded = StunAttribute::decode(0x0020, value, &transaction_id).unwrap();

        if let StunAttribute::XorMappedAddress(decoded_addr) = decoded {
            assert_eq!(decoded_addr, addr);
        } else {
            panic!("Expected XorMappedAddress");
        }
    }

    #[test]
    fn test_magic_cookie() {
        assert_eq!(MAGIC_COOKIE, 0x2112_A442);
    }
}
