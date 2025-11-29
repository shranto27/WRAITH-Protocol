# WRAITH-Chat Architecture

**Document Version:** 1.0.0
**Last Updated:** 2025-11-28
**Client Version:** 1.0.0

---

## Overview

WRAITH-Chat is a secure messaging application providing Signal-level security with WRAITH protocol's traffic obfuscation and decentralized architecture. It supports 1:1 conversations, group chats, and voice/video calls without central servers.

**Design Goals:**
- End-to-end encryption with forward secrecy (Double Ratchet)
- Sub-second message delivery on local networks
- Support for 250+ member group chats
- Cross-platform sync without cloud servers
- Metadata protection (sealed sender)
- Voice/video calls with P2P media streaming

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────┐
│               User Interface Layer                  │
│  ┌────────────────┐  ┌──────────────────────────┐   │
│  │  Desktop UI    │  │     Mobile UI            │   │
│  │ (Tauri+React)  │  │   (React Native)         │   │
│  └────────────────┘  └──────────────────────────┘   │
└─────────────────────────────────────────────────────┘
                         │
┌─────────────────────────────────────────────────────┐
│            Application Logic Layer                  │
│  ┌──────────────────────────────────────────────┐   │
│  │  Message Manager                             │   │
│  │  - Send/receive messages                     │   │
│  │  - Group chat coordination                   │   │
│  │  │  - Delivery receipts                      │   │
│  └──────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────┐   │
│  │  Crypto Manager (Double Ratchet)             │   │
│  │  - Per-contact ratchet state                 │   │
│  │  - Sender key distribution (groups)          │   │
│  │  - Key rotation                              │   │
│  └──────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────┐   │
│  │  Media Manager                               │   │
│  │  - Image/video encryption                    │   │
│  │  - Voice message recording                   │   │
│  │  - Thumbnail generation                      │   │
│  └──────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────┘
                         │
┌──────────────────────────────────────────────────────┐
│           Database Layer (SQLCipher)                 │
│  - Messages, contacts, groups                        │
│  - Ratchet states (encrypted)                        │
│  - Media metadata                                    │
└──────────────────────────────────────────────────────┘
                         │
┌──────────────────────────────────────────────────────┐
│         WRAITH Protocol Stack                        │
│  (wraith-core, wraith-discovery,                     │
│   wraith-transport, wraith-crypto)                   │
└──────────────────────────────────────────────────────┘
```

---

## Components

### 1. Message Manager

**Responsibilities:**
- Send/receive text messages
- Manage group message distribution
- Track delivery and read receipts
- Handle typing indicators
- Manage message expiration (disappearing messages)

**Implementation:**
```rust
pub struct MessageManager {
    db: Arc<Database>,
    crypto: Arc<CryptoManager>,
    wraith: Arc<WraithClient>,
    pending_messages: Arc<RwLock<HashMap<MessageId, PendingMessage>>>,
}

impl MessageManager {
    pub async fn send_message(
        &self,
        conversation_id: ConversationId,
        content: MessageContent,
    ) -> Result<MessageId> {
        // Encrypt message with Double Ratchet
        let encrypted = self.crypto.encrypt_message(
            conversation_id,
            &content,
        ).await?;

        // Send via WRAITH protocol
        let message_id = self.wraith.send_message(
            &encrypted,
            self.get_peer_id(conversation_id).await?,
        ).await?;

        // Store in database
        self.db.insert_message(message_id, conversation_id, content).await?;

        Ok(message_id)
    }

    pub async fn receive_message(
        &self,
        encrypted: EncryptedMessage,
    ) -> Result<()> {
        // Decrypt message
        let (peer_id, content) = self.crypto.decrypt_message(&encrypted).await?;

        // Store in database
        let conversation_id = self.get_or_create_conversation(peer_id).await?;
        self.db.insert_message(
            encrypted.message_id,
            conversation_id,
            content,
        ).await?;

        // Send delivery receipt
        self.send_receipt(peer_id, encrypted.message_id, ReceiptType::Delivered).await?;

        Ok(())
    }

    pub async fn send_group_message(
        &self,
        group_id: GroupId,
        content: MessageContent,
    ) -> Result<MessageId> {
        let group = self.db.get_group(group_id).await?;

        // Encrypt with sender key
        let encrypted = self.crypto.encrypt_group_message(
            group_id,
            &content,
        ).await?;

        // Send to all members
        let mut tasks = Vec::new();
        for member in group.members {
            if member.peer_id == self.wraith.local_peer_id() {
                continue; // Skip self
            }

            let encrypted = encrypted.clone();
            let peer_id = member.peer_id.clone();
            let wraith = self.wraith.clone();

            tasks.push(tokio::spawn(async move {
                wraith.send_message(&encrypted, peer_id).await
            }));
        }

        // Wait for all sends
        futures::future::join_all(tasks).await;

        // Store in database
        let message_id = MessageId::new();
        self.db.insert_message(message_id, group_id.into(), content).await?;

        Ok(message_id)
    }
}
```

---

### 2. Double Ratchet Cryptography

**Description:** Signal Protocol's Double Ratchet algorithm provides forward secrecy and post-compromise security.

**Ratchet State:**
```rust
pub struct RatchetState {
    // Root key (derives chain keys)
    pub root_key: [u8; 32],

    // Sending chain
    pub sending_chain_key: [u8; 32],
    pub sending_chain_index: u32,

    // Receiving chain
    pub receiving_chain_key: [u8; 32],
    pub receiving_chain_index: u32,

    // DH ratchet keys
    pub dh_sending_key: [u8; 32],
    pub dh_receiving_key: Option<[u8; 32]>,

    // Skipped message keys (for out-of-order)
    pub skipped_keys: HashMap<u32, [u8; 32]>,
}
```

**Encryption Flow:**
```
Alice                                    Bob
  │                                       │
  │──── Initial DH Exchange ────────────> │
  │  (Noise_XX handshake)                 │
  │                                       │
  │  Derive root key from shared secret   │
  │                                       │
  │──── Message 1 (encrypted) ──────────> │
  │  Chain index: 0                       │
  │  Ratchet sending key                  │
  │                                       │
  │<─── Message 2 (encrypted) ─────────── │
  │  Chain index: 0                       │
  │  Ratchet receiving key                │
  │                                       │
  │  Perform DH ratchet step              │
  │  (derive new root key)                │
  │                                       │
  │──── Message 3 (encrypted) ──────────> │
  │  New sending chain                    │
```

---

### 3. Group Chat Architecture

**Sender Keys:** For efficiency, group chats use sender key encryption (similar to Signal).

**Group State:**
```rust
pub struct Group {
    pub id: GroupId,
    pub name: String,
    pub members: Vec<GroupMember>,
    pub sender_key: [u8; 32],
    pub sender_key_generation: u32,
    pub created_at: SystemTime,
}

pub struct GroupMember {
    pub peer_id: PeerId,
    pub display_name: String,
    pub role: GroupRole,
    pub joined_at: SystemTime,
}

pub enum GroupRole {
    Admin,
    Member,
}
```

**Group Message Flow:**
```
Admin                    Member A                Member B
  │                         │                       │
  │── Create Group ────────>│                       │
  │   (sender key)          │                       │
  │                         │                       │
  │────────────────────────────────────────────────>│
  │   (sender key)          │                       │
  │                         │                       │
  │                         │── Group Message ─────>│
  │<─────────────────────────┘  (encrypted with     │
  │                             sender key)         │
  │                         │                       │
```

---

### 4. Media Handling

**Media Types:**
- Images (JPEG, PNG, WebP)
- Videos (MP4, WebM)
- Voice messages (Opus)
- Files (any type, <100 MB)

**Media Storage:**
```
~/.local/share/wraith-chat/media/
├── images/
│   ├── <hash>.jpg
│   └── <hash>.webp
├── videos/
│   └── <hash>.mp4
├── voice/
│   └── <hash>.opus
└── files/
    └── <hash>
```

**Media Encryption:**
```rust
pub fn encrypt_media(data: &[u8], key: &[u8; 32]) -> Vec<u8> {
    // Generate random nonce
    let nonce = generate_nonce();

    // Encrypt with XChaCha20-Poly1305
    let cipher = XChaCha20Poly1305::new(key.into());
    let encrypted = cipher.encrypt(&nonce.into(), data)
        .expect("encryption failure");

    // Prepend nonce to encrypted data
    let mut result = nonce.to_vec();
    result.extend_from_slice(&encrypted);

    result
}
```

---

### 5. Voice/Video Calls

**Signaling:** Call setup uses WRAITH protocol messages.

**Media Transport:** WebRTC data channels for voice/video.

**Call Flow:**
```
Caller                          Callee
  │                                │
  │──── CALL_OFFER ──────────────> │
  │  (SDP offer)                   │
  │                                │
  │<─── CALL_ANSWER ─────────────── │
  │  (SDP answer)                  │
  │                                │
  │<══ ICE Candidates ═══════════> │
  │                                │
  │<══ WebRTC Media Stream ══════> │
  │  (encrypted RTP)               │
```

**Call Manager:**
```rust
pub struct CallManager {
    webrtc: Arc<WebRtcPeer>,
    active_calls: Arc<RwLock<HashMap<CallId, Call>>>,
}

impl CallManager {
    pub async fn initiate_call(
        &self,
        peer_id: PeerId,
        audio_only: bool,
    ) -> Result<CallId> {
        let call_id = CallId::new();

        // Create WebRTC peer connection
        let pc = self.webrtc.create_peer_connection().await?;

        // Add media tracks
        if !audio_only {
            pc.add_video_track().await?;
        }
        pc.add_audio_track().await?;

        // Create SDP offer
        let offer = pc.create_offer().await?;

        // Send offer via WRAITH
        self.send_call_signaling(peer_id, CallMessage::Offer {
            call_id,
            sdp: offer,
        }).await?;

        // Store call state
        self.active_calls.write().await.insert(call_id, Call {
            id: call_id,
            peer_id,
            peer_connection: pc,
            status: CallStatus::Ringing,
        });

        Ok(call_id)
    }

    pub async fn answer_call(
        &self,
        call_id: CallId,
        offer: String,
    ) -> Result<()> {
        // Create peer connection
        let pc = self.webrtc.create_peer_connection().await?;

        // Set remote description
        pc.set_remote_description(offer).await?;

        // Add media tracks
        pc.add_audio_track().await?;

        // Create answer
        let answer = pc.create_answer().await?;

        // Send answer
        let call = self.active_calls.read().await
            .get(&call_id).cloned()
            .ok_or("call not found")?;

        self.send_call_signaling(call.peer_id, CallMessage::Answer {
            call_id,
            sdp: answer,
        }).await?;

        Ok(())
    }
}
```

---

## Data Flow

### Message Send Flow

```
1. User types message in UI
2. UI calls MessageManager.send_message()
3. MessageManager encrypts message with Double Ratchet
4. Encrypted message sent via WRAITH protocol
5. Message stored in local database
6. UI updated with "Sending" status
7. Peer receives message
8. Peer sends delivery receipt
9. UI updated with "Delivered" status
10. Peer marks message as read
11. Peer sends read receipt
12. UI updated with "Read" status
```

### Message Receive Flow

```
1. WRAITH protocol receives encrypted message
2. MessageManager.receive_message() called
3. Message decrypted with Double Ratchet
4. Message stored in database
5. UI notified of new message
6. Delivery receipt sent to sender
7. User opens conversation
8. Read receipt sent to sender
```

---

## Protocol Integration

### Message Format

```rust
pub struct EncryptedMessage {
    pub message_id: MessageId,
    pub sender_id: PeerId,
    pub timestamp: u64,
    pub ciphertext: Vec<u8>,
    pub ratchet_public_key: [u8; 32],
    pub ratchet_index: u32,
}
```

**Wire Format:**
```
┌─────────────────────────────────────────┐
│ Message ID (16 bytes)                   │
│ Sender ID (32 bytes)                    │
│ Timestamp (8 bytes)                     │
│ Ratchet Public Key (32 bytes)           │
│ Ratchet Index (4 bytes)                 │
│ Ciphertext Length (4 bytes)             │
│ Ciphertext (variable)                   │
└─────────────────────────────────────────┘
```

---

## Security Considerations

### Metadata Protection

**Sealed Sender:** Message sender identity encrypted for all but recipient.

```rust
pub fn seal_sender(
    message: &EncryptedMessage,
    recipient_public_key: &[u8; 32],
) -> SealedMessage {
    // Encrypt sender ID with recipient's public key
    let sealed_sender = encrypt_sender_id(
        &message.sender_id,
        recipient_public_key,
    );

    SealedMessage {
        message_id: message.message_id,
        sealed_sender,
        ciphertext: message.ciphertext.clone(),
        // ...
    }
}
```

### Forward Secrecy

**Key Deletion:** Old ratchet keys deleted after use.

```rust
pub fn ratchet_forward(&mut self) {
    // Derive new chain key
    let new_chain_key = kdf(&self.sending_chain_key, b"chain_key");

    // Derive message key (then delete)
    let message_key = kdf(&self.sending_chain_key, b"message_key");

    // Update state
    self.sending_chain_key = new_chain_key;
    self.sending_chain_index += 1;

    // Old chain key automatically dropped (Rust ownership)
}
```

### Post-Compromise Security

**DH Ratchet:** Periodic Diffie-Hellman exchanges heal from key compromise.

---

## Performance Characteristics

**Message Latency:**
- Local network: <100ms
- Internet (direct): <500ms
- Internet (relay): <1000ms

**Throughput:**
- Text messages: 1000+ messages/second
- Media: Limited by network bandwidth

**Memory Usage:**
- Baseline: 100 MB
- + 10 KB per conversation
- + 500 KB per cached image

**Database Size:**
- 50 KB per 1000 text messages
- Media stored separately (actual file size)

---

## See Also

- [Features](features.md)
- [Implementation](implementation.md)
- [Client Overview](../overview.md)
- [Security Model](../../architecture/security-model.md)
