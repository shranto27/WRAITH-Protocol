# WRAITH-Chat Features

**Document Version:** 1.0.0
**Last Updated:** 2025-11-28
**Client Version:** 1.0.0

---

## Overview

WRAITH-Chat provides secure messaging with Signal-level encryption combined with WRAITH protocol's traffic obfuscation. This document details all features for both 1:1 and group conversations.

---

## Core Messaging Features

### 1. End-to-End Encrypted Messages

**Description:** All messages encrypted using Double Ratchet algorithm (Signal Protocol).

**User Stories:**
- As a user, I can send messages knowing only the recipient can read them
- As a user, I can verify encryption status via safety numbers
- As a user, old messages remain secure even if keys are compromised later

**Encryption Properties:**
- Forward secrecy: Compromised keys don't expose past messages
- Post-compromise security: New keys heal from compromise
- Deniable authentication: No cryptographic proof of who sent messages

**Safety Numbers:**
```
Verify safety number with contact:
Alice: 12345 67890 12345 67890 12345
Bob:   98765 43210 98765 43210 98765

Both users must verify these match to ensure
no man-in-the-middle attack.
```

---

### 2. Group Chats

**Description:** Create group conversations with up to 250 members.

**User Stories:**
- As a user, I can create a group chat and invite contacts
- As an admin, I can add/remove members and assign other admins
- As a member, I can see group member list and admin status

**Group Roles:**
- **Admin:** Can add/remove members, promote to admin, change group name/avatar
- **Member:** Can send messages, leave group

**Group Features:**
- Member invitation via QR code or shareable link
- Group name and avatar customization
- Member join/leave notifications
- Admin controls (member management)

**Implementation:**
```typescript
interface Group {
  id: string;
  name: string;
  avatar?: string;
  members: GroupMember[];
  createdAt: number;
  createdBy: string;
}

interface GroupMember {
  peerId: string;
  displayName: string;
  role: 'admin' | 'member';
  joinedAt: number;
}
```

---

### 3. Disappearing Messages

**Description:** Messages automatically deleted after specified time.

**User Stories:**
- As a user, I can set messages to disappear after 5 seconds to 1 week
- As a user, I can configure different expiration times per conversation
- As a user, I receive notification when expiration timer changes

**Expiration Options:**
- 5 seconds
- 30 seconds
- 1 minute
- 5 minutes
- 30 minutes
- 1 hour
- 6 hours
- 1 day
- 1 week

**Security Note:** Expiration prevents long-term storage but doesn't prevent screenshots or photos of screen.

---

### 4. Message Delivery Receipts

**Description:** Track message delivery and read status.

**Receipt Types:**
- **Sent:** Message sent from your device
- **Delivered:** Message delivered to recipient's device
- **Read:** Recipient opened conversation and saw message

**User Stories:**
- As a user, I can see when my messages are delivered
- As a user, I can see when my messages are read
- As a user, I can disable read receipts for privacy

**UI Indicators:**
```
✓   Sent
✓✓  Delivered
✓✓  Read (blue checks)
```

---

### 5. Typing Indicators

**Description:** See when contacts are typing in real-time.

**User Stories:**
- As a user, I see "Alice is typing..." when contact types
- As a user, I can disable typing indicators for privacy

**Configuration:**
```toml
[privacy]
send_typing_indicators = true
show_typing_indicators = true
```

---

## Media Sharing

### 1. Image Attachments

**Description:** Send and receive encrypted images.

**Supported Formats:**
- JPEG, PNG, WebP, GIF

**Features:**
- Automatic compression (configurable quality)
- Thumbnail generation
- Full-resolution download on tap
- Save to device gallery

**Compression Settings:**
```toml
[media.images]
max_size = 5242880  # 5 MB
quality = 85  # 0-100
generate_thumbnails = true
thumbnail_size = 200  # pixels
```

---

### 2. Video Attachments

**Description:** Send and receive encrypted videos.

**Supported Formats:**
- MP4, WebM, MOV

**Features:**
- Automatic transcoding to H.264
- Thumbnail extraction (first frame)
- Streaming playback
- Save to device

**Limits:**
```toml
[media.videos]
max_size = 104857600  # 100 MB
max_duration = 300  # 5 minutes
transcode_bitrate = "2M"
```

---

### 3. Voice Messages

**Description:** Record and send voice messages.

**User Stories:**
- As a user, I can tap-and-hold to record voice message
- As a user, I can slide to cancel recording
- As a user, voice messages play inline in conversation

**Audio Format:**
- Codec: Opus (high-quality, low bitrate)
- Bitrate: 64 kbps
- Max duration: 60 seconds

**UI:**
```
┌─────────────────────────────────┐
│  🎤 [=========>        ] 0:15    │
│  Slide to cancel  →             │
└─────────────────────────────────┘
```

---

### 4. File Attachments

**Description:** Send any file type up to 100 MB.

**Features:**
- File icon based on type
- Progress indicator for large files
- Resume support for interrupted transfers
- Open in external app

**Supported Types:**
- Documents (PDF, DOCX, XLSX, etc.)
- Archives (ZIP, RAR, TAR.GZ)
- Code files
- Any other type

---

## Advanced Features

### 1. Voice Calls

**Description:** Encrypted peer-to-peer voice calls.

**User Stories:**
- As a user, I can initiate voice call from contact profile
- As a user, I receive incoming call notifications
- As a user, calls are encrypted end-to-end

**Call Features:**
- HD audio (Opus codec, 48 kHz)
- Echo cancellation
- Noise suppression
- Auto gain control

**Call UI:**
```
┌─────────────────────────────────┐
│      🔒 Encrypted Call          │
│                                 │
│      Alice Johnson              │
│      Calling...                 │
│      00:00                      │
│                                 │
│   [Mute]  [Speaker]  [End]      │
└─────────────────────────────────┘
```

---

### 2. Video Calls

**Description:** Encrypted peer-to-peer video calls.

**User Stories:**
- As a user, I can make video calls with contacts
- As a user, I can toggle camera on/off during call
- As a user, I can switch between front/rear camera

**Video Settings:**
- Resolution: 720p (default), up to 1080p
- Frame rate: 30 FPS
- Codec: VP8/VP9 or H.264
- Adaptive bitrate (500 kbps - 2 Mbps)

**Call Controls:**
- Mute microphone
- Toggle camera
- Switch camera (front/rear)
- End call

---

### 3. Contact Verification

**Description:** Verify contact identity via safety numbers.

**User Stories:**
- As a user, I can compare safety numbers with contact
- As a user, I can scan contact's QR code for quick verification
- As a user, verified contacts show special indicator

**Verification Methods:**
1. **Manual comparison:** Read 60-digit safety number over secure channel
2. **QR code:** Scan contact's QR code in person
3. **Out-of-band:** Verify fingerprint via trusted third party

**Safety Number Format:**
```
12345 67890 12345 67890 12345 67890
12345 67890 12345 67890 12345 67890
```

---

### 4. Message Search

**Description:** Full-text search across all conversations.

**User Stories:**
- As a user, I can search messages by keyword
- As a user, I can filter search by contact or group
- As a user, search results show message context

**Search Features:**
- Instant search (results as you type)
- Filter by conversation
- Filter by date range
- Filter by media type
- Jump to message in conversation

---

### 5. Emoji Reactions

**Description:** React to messages with emoji.

**User Stories:**
- As a user, I can tap-and-hold message to add reaction
- As a user, I can see who reacted with each emoji
- As a user, I can remove my reactions

**Popular Reactions:**
- ❤️ Heart
- 👍 Thumbs up
- 😂 Laughing
- 😮 Surprised
- 😢 Sad
- 🎉 Party

---

### 6. Link Previews

**Description:** Automatic preview for shared links.

**User Stories:**
- As a user, I see preview with title, description, and image
- As a user, I can disable link previews for privacy
- As a user, link previews load after sending for speed

**Privacy Note:** Link previews are generated client-side. The linked website will see your IP address when fetching preview data.

---

## Privacy Features

### 1. Screenshot Detection

**Description:** Detect when recipient takes screenshot (mobile only).

**User Stories:**
- As a user, I receive notification when recipient screenshots conversation
- As a user, I can disable screenshot notifications

**Platform Support:**
- iOS: Full support
- Android: Supported on most devices
- Desktop: Not supported

---

### 2. App Lock

**Description:** Require PIN/biometric to open app.

**User Stories:**
- As a user, I can set PIN to protect app access
- As a user, I can use fingerprint/face recognition
- As a user, I can configure lock timeout (immediate, 1 min, 5 min, 30 min)

**Lock Options:**
- 4-6 digit PIN
- Fingerprint (iOS/Android)
- Face ID (iOS)
- Face unlock (Android)

---

### 3. Privacy Settings

**Description:** Granular privacy controls.

**Settings:**
```toml
[privacy]
# Read receipts
send_read_receipts = true

# Typing indicators
send_typing_indicators = true

# Last seen
show_last_seen = "contacts"  # everyone, contacts, nobody

# Profile photo
show_profile_photo = "everyone"  # everyone, contacts, nobody

# Status
show_status = "contacts"  # everyone, contacts, nobody

# Link previews
generate_link_previews = false
```

---

## User Interface

### Conversation View

```
┌─────────────────────────────────────────┐
│  ← Alice Johnson          🔒 Verified   │
├─────────────────────────────────────────┤
│                                         │
│  [Them]  Hey, how are you?              │
│          10:30 AM                       │
│                                         │
│                          Great! ✓✓ [Me] │
│                          10:31 AM       │
│                                         │
│  [Them]  Let's grab coffee later?       │
│          10:32 AM                       │
│                                         │
│                     Sounds good! ✓ [Me] │
│                          10:33 AM       │
│                      (Sending...)       │
│                                         │
├─────────────────────────────────────────┤
│ [+] [Type message...]         [Send] →  │
└─────────────────────────────────────────┘
```

### Contact Profile

```
┌─────────────────────────────────────────┐
│              Alice Johnson              │
│                   🔒                    │
│              Verified                   │
│                                         │
│  Phone: +1 (555) 123-4567               │
│  Status: Hey there! I use WRAITH        │
│  Last seen: Today at 10:35 AM           │
│                                         │
│  [🔊 Voice Call]  [📹 Video Call]        │
│                                         │
│  ────────────────────────────────────   │
│                                         │
│  Media shared: 127                      │
│  [View all →]                           │
│                                         │
│  Encryption                             │
│  Safety number: 12345 67890...          │
│  [Verify →]                             │
│                                         │
│  Disappearing messages: Off             │
│  [Configure →]                          │
│                                         │
│  [Block Contact]                        │
│  [Delete Conversation]                  │
└─────────────────────────────────────────┘
```

---

## Notifications

**Types:**
- New message
- Missed call
- Group invite
- Verification request

**Settings:**
```toml
[notifications]
enabled = true
show_message_preview = true
show_sender_name = true
sound = "default"
vibrate = true
led_color = "#2196F3"

# Do Not Disturb
dnd_enabled = false
dnd_start = "22:00"
dnd_end = "07:00"
```

---

## Platform-Specific Features

### Mobile (iOS/Android)

- Push notifications (FCM/APNs)
- Share extension (share from other apps)
- Contact integration
- Call kit integration (iOS)
- Picture-in-picture video calls
- Background message sync

### Desktop (Windows/macOS/Linux)

- System tray icon with badge count
- Desktop notifications
- Global keyboard shortcuts
- Multiple windows support
- Screen sharing (in calls)

---

## See Also

- [Architecture](architecture.md)
- [Implementation](implementation.md)
- [Client Overview](../overview.md)
