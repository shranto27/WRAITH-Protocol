# WRAITH-Chat Implementation

**Document Version:** 1.0.0
**Last Updated:** 2025-11-28
**Client Version:** 1.0.0

---

## Overview

This document provides implementation details for WRAITH-Chat, including cryptographic protocols, database schema, and platform-specific code.

---

## Technology Stack

### Cross-Platform (React Native)

```json
{
  "dependencies": {
    "react-native": "^0.73.0",
    "@react-navigation/native": "^6.1.0",
    "@react-navigation/stack": "^6.3.0",
    "react-native-sqlcipher-storage": "^0.2.0",
    "@react-native-async-storage/async-storage": "^1.21.0",
    "react-native-webrtc": "^118.0.0",
    "react-native-opus": "^1.0.0",
    "react-native-camera": "^4.2.0",
    "react-native-qrcode-scanner": "^1.5.0"
  }
}
```

### Desktop Wrapper (Tauri)

```toml
[dependencies]
tauri = { version = "2.0", features = ["notification-all"] }
wraith-core = { path = "../../crates/wraith-core" }
wraith-crypto = { path = "../../crates/wraith-crypto" }
tokio = { version = "1.40", features = ["full"] }
sqlcipher = "0.31"
```

---

## Double Ratchet Implementation

### Ratchet State Management

```typescript
// src/crypto/DoubleRatchet.ts
import * as crypto from 'crypto';
import { X25519 } from './x25519';
import { HKDF } from './hkdf';

export class DoubleRatchet {
  private rootKey: Buffer;
  private sendingChainKey: Buffer;
  private receivingChainKey: Buffer;
  private sendingChainIndex: number = 0;
  private receivingChainIndex: number = 0;
  private dhSendingKey: Buffer;
  private dhReceivingKey?: Buffer;
  private skippedKeys: Map<number, Buffer> = new Map();

  constructor(sharedSecret: Buffer) {
    // Initialize from Noise_XX handshake output
    this.rootKey = HKDF(sharedSecret, 'wraith-chat-root-key', 32);
    this.sendingChainKey = HKDF(sharedSecret, 'wraith-chat-send-chain', 32);
    this.receivingChainKey = HKDF(sharedSecret, 'wraith-chat-recv-chain', 32);
    this.dhSendingKey = X25519.generateKeyPair().privateKey;
  }

  public encrypt(plaintext: Buffer): EncryptedMessage {
    // Derive message key from chain key
    const messageKey = this.deriveMessageKey(this.sendingChainKey);

    // Advance chain
    this.sendingChainKey = this.deriveChainKey(this.sendingChainKey);
    const index = this.sendingChainIndex++;

    // Encrypt with XChaCha20-Poly1305
    const nonce = crypto.randomBytes(24);
    const cipher = crypto.createCipheriv('chacha20-poly1305', messageKey, nonce);
    const ciphertext = Buffer.concat([
      cipher.update(plaintext),
      cipher.final(),
      cipher.getAuthTag()
    ]);

    return {
      ratchetPublicKey: X25519.getPublicKey(this.dhSendingKey),
      ratchetIndex: index,
      nonce,
      ciphertext
    };
  }

  public decrypt(message: EncryptedMessage): Buffer {
    // Check if we need to perform DH ratchet
    if (message.ratchetPublicKey !== this.dhReceivingKey) {
      this.performDHRatchet(message.ratchetPublicKey);
    }

    // Check for skipped messages
    const currentIndex = this.receivingChainIndex;
    if (message.ratchetIndex > currentIndex) {
      // Save skipped keys
      for (let i = currentIndex; i < message.ratchetIndex; i++) {
        const skippedKey = this.deriveMessageKey(this.receivingChainKey);
        this.skippedKeys.set(i, skippedKey);
        this.receivingChainKey = this.deriveChainKey(this.receivingChainKey);
      }
    }

    // Get message key
    const messageKey = message.ratchetIndex === this.receivingChainIndex
      ? this.deriveMessageKey(this.receivingChainKey)
      : this.skippedKeys.get(message.ratchetIndex)!;

    // Advance chain if current message
    if (message.ratchetIndex === this.receivingChainIndex) {
      this.receivingChainKey = this.deriveChainKey(this.receivingChainKey);
      this.receivingChainIndex++;
    } else {
      this.skippedKeys.delete(message.ratchetIndex);
    }

    // Decrypt
    const decipher = crypto.createDecipheriv('chacha20-poly1305', messageKey, message.nonce);
    const plaintext = Buffer.concat([
      decipher.update(message.ciphertext.slice(0, -16)),
      decipher.final()
    ]);

    return plaintext;
  }

  private performDHRatchet(peerPublicKey: Buffer): void {
    // Compute new shared secret
    const dhOutput = X25519.computeSharedSecret(this.dhSendingKey, peerPublicKey);

    // Derive new root key and chain keys
    const [newRootKey, newReceivingChainKey] = HKDF(
      Buffer.concat([this.rootKey, dhOutput]),
      'wraith-chat-ratchet',
      64
    ).split(32);

    this.rootKey = newRootKey;
    this.receivingChainKey = newReceivingChainKey;
    this.receivingChainIndex = 0;
    this.dhReceivingKey = peerPublicKey;

    // Generate new sending key pair
    this.dhSendingKey = X25519.generateKeyPair().privateKey;

    // Derive new sending chain
    const dhOutput2 = X25519.computeSharedSecret(this.dhSendingKey, peerPublicKey);
    const [newRootKey2, newSendingChainKey] = HKDF(
      Buffer.concat([this.rootKey, dhOutput2]),
      'wraith-chat-ratchet',
      64
    ).split(32);

    this.rootKey = newRootKey2;
    this.sendingChainKey = newSendingChainKey;
    this.sendingChainIndex = 0;
  }

  private deriveMessageKey(chainKey: Buffer): Buffer {
    return HKDF(chainKey, 'wraith-chat-message-key', 32);
  }

  private deriveChainKey(chainKey: Buffer): Buffer {
    return HKDF(chainKey, 'wraith-chat-chain-key', 32);
  }

  public serialize(): string {
    return JSON.stringify({
      rootKey: this.rootKey.toString('base64'),
      sendingChainKey: this.sendingChainKey.toString('base64'),
      receivingChainKey: this.receivingChainKey.toString('base64'),
      sendingChainIndex: this.sendingChainIndex,
      receivingChainIndex: this.receivingChainIndex,
      dhSendingKey: this.dhSendingKey.toString('base64'),
      dhReceivingKey: this.dhReceivingKey?.toString('base64'),
      skippedKeys: Array.from(this.skippedKeys.entries())
        .map(([idx, key]) => [idx, key.toString('base64')])
    });
  }

  public static deserialize(data: string): DoubleRatchet {
    const obj = JSON.parse(data);
    const ratchet = Object.create(DoubleRatchet.prototype);

    ratchet.rootKey = Buffer.from(obj.rootKey, 'base64');
    ratchet.sendingChainKey = Buffer.from(obj.sendingChainKey, 'base64');
    ratchet.receivingChainKey = Buffer.from(obj.receivingChainKey, 'base64');
    ratchet.sendingChainIndex = obj.sendingChainIndex;
    ratchet.receivingChainIndex = obj.receivingChainIndex;
    ratchet.dhSendingKey = Buffer.from(obj.dhSendingKey, 'base64');
    ratchet.dhReceivingKey = obj.dhReceivingKey
      ? Buffer.from(obj.dhReceivingKey, 'base64')
      : undefined;
    ratchet.skippedKeys = new Map(
      obj.skippedKeys.map(([idx, key]: [number, string]) =>
        [idx, Buffer.from(key, 'base64')]
      )
    );

    return ratchet;
  }
}
```

---

## Database Schema

### SQLCipher Database

```typescript
// src/database/Database.ts
import SQLite from 'react-native-sqlcipher-storage';

export class Database {
  private db: SQLite.SQLiteDatabase;

  public async open(password: string): Promise<void> {
    this.db = await SQLite.openDatabase({
      name: 'wraith_chat.db',
      location: 'default',
      key: password,
    });

    await this.createTables();
  }

  private async createTables(): Promise<void> {
    await this.db.executeSql(`
      CREATE TABLE IF NOT EXISTS contacts (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        peer_id TEXT UNIQUE NOT NULL,
        display_name TEXT,
        identity_key BLOB NOT NULL,
        safety_number TEXT NOT NULL,
        verified INTEGER DEFAULT 0,
        blocked INTEGER DEFAULT 0,
        created_at INTEGER NOT NULL,
        last_seen INTEGER
      );

      CREATE TABLE IF NOT EXISTS conversations (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        type TEXT NOT NULL CHECK(type IN ('direct', 'group')),
        peer_id TEXT,
        group_id TEXT,
        display_name TEXT,
        avatar BLOB,
        muted INTEGER DEFAULT 0,
        archived INTEGER DEFAULT 0,
        last_message_id INTEGER,
        last_message_at INTEGER,
        unread_count INTEGER DEFAULT 0,
        expires_in INTEGER,
        FOREIGN KEY (last_message_id) REFERENCES messages(id)
      );

      CREATE TABLE IF NOT EXISTS messages (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        conversation_id INTEGER NOT NULL,
        sender_peer_id TEXT NOT NULL,
        content_type TEXT NOT NULL CHECK(content_type IN ('text', 'media', 'voice', 'file')),
        body TEXT,
        media_path TEXT,
        media_mime_type TEXT,
        media_size INTEGER,
        timestamp INTEGER NOT NULL,
        sent INTEGER DEFAULT 0,
        delivered INTEGER DEFAULT 0,
        read INTEGER DEFAULT 0,
        expires_at INTEGER,
        direction TEXT NOT NULL CHECK(direction IN ('incoming', 'outgoing')),
        FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
      );

      CREATE TABLE IF NOT EXISTS group_members (
        group_id TEXT NOT NULL,
        peer_id TEXT NOT NULL,
        role TEXT NOT NULL CHECK(role IN ('admin', 'member')),
        joined_at INTEGER NOT NULL,
        PRIMARY KEY (group_id, peer_id)
      );

      CREATE TABLE IF NOT EXISTS ratchet_states (
        peer_id TEXT PRIMARY KEY,
        state_json TEXT NOT NULL,
        updated_at INTEGER NOT NULL
      );

      CREATE INDEX idx_messages_conversation ON messages(conversation_id, timestamp DESC);
      CREATE INDEX idx_messages_sender ON messages(sender_peer_id);
      CREATE INDEX idx_contacts_peer_id ON contacts(peer_id);
    `);
  }

  public async insertMessage(message: Message): Promise<number> {
    const result = await this.db.executeSql(`
      INSERT INTO messages (
        conversation_id, sender_peer_id, content_type, body,
        timestamp, direction, sent, delivered, read
      ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
    `, [
      message.conversationId,
      message.senderPeerId,
      message.contentType,
      message.body,
      message.timestamp,
      message.direction,
      message.sent ? 1 : 0,
      message.delivered ? 1 : 0,
      message.read ? 1 : 0,
    ]);

    return result[0].insertId;
  }

  public async getMessages(conversationId: number, limit: number = 50): Promise<Message[]> {
    const [result] = await this.db.executeSql(`
      SELECT * FROM messages
      WHERE conversation_id = ?
      ORDER BY timestamp DESC
      LIMIT ?
    `, [conversationId, limit]);

    return result.rows.raw();
  }

  public async saveRatchetState(peerId: string, ratchet: DoubleRatchet): Promise<void> {
    await this.db.executeSql(`
      INSERT OR REPLACE INTO ratchet_states (peer_id, state_json, updated_at)
      VALUES (?, ?, ?)
    `, [peerId, ratchet.serialize(), Date.now()]);
  }

  public async loadRatchetState(peerId: string): Promise<DoubleRatchet | null> {
    const [result] = await this.db.executeSql(`
      SELECT state_json FROM ratchet_states WHERE peer_id = ?
    `, [peerId]);

    if (result.rows.length === 0) {
      return null;
    }

    return DoubleRatchet.deserialize(result.rows.item(0).state_json);
  }
}
```

---

## WebRTC Call Implementation

```typescript
// src/calls/CallManager.ts
import { RTCPeerConnection, RTCSessionDescription, MediaStream } from 'react-native-webrtc';

export class CallManager {
  private peerConnection: RTCPeerConnection | null = null;
  private localStream: MediaStream | null = null;

  public async initiateCall(peerId: string, audioOnly: boolean): Promise<string> {
    // Get local media
    this.localStream = await this.getLocalMedia(audioOnly);

    // Create peer connection
    this.peerConnection = new RTCPeerConnection({
      iceServers: [
        { urls: 'stun:stun.l.google.com:19302' },
      ],
    });

    // Add local tracks
    this.localStream.getTracks().forEach(track => {
      this.peerConnection!.addTrack(track, this.localStream!);
    });

    // Handle ICE candidates
    this.peerConnection.onicecandidate = (event) => {
      if (event.candidate) {
        this.sendICECandidate(peerId, event.candidate);
      }
    };

    // Create offer
    const offer = await this.peerConnection.createOffer({
      offerToReceiveAudio: true,
      offerToReceiveVideo: !audioOnly,
    });

    await this.peerConnection.setLocalDescription(offer);

    // Send offer via WRAITH
    await this.sendCallSignaling(peerId, {
      type: 'offer',
      sdp: offer.sdp!,
    });

    return 'call-id-' + Date.now();
  }

  public async answerCall(offer: RTCSessionDescription, audioOnly: boolean): Promise<void> {
    // Get local media
    this.localStream = await this.getLocalMedia(audioOnly);

    // Create peer connection
    this.peerConnection = new RTCPeerConnection({
      iceServers: [
        { urls: 'stun:stun.l.google.com:19302' },
      ],
    });

    // Add local tracks
    this.localStream.getTracks().forEach(track => {
      this.peerConnection!.addTrack(track, this.localStream!);
    });

    // Set remote description
    await this.peerConnection.setRemoteDescription(offer);

    // Create answer
    const answer = await this.peerConnection.createAnswer();
    await this.peerConnection.setLocalDescription(answer);

    // Send answer
    await this.sendCallSignaling(peerId, {
      type: 'answer',
      sdp: answer.sdp!,
    });
  }

  private async getLocalMedia(audioOnly: boolean): Promise<MediaStream> {
    const constraints = {
      audio: {
        echoCancellation: true,
        noiseSuppression: true,
        autoGainControl: true,
      },
      video: audioOnly ? false : {
        width: { ideal: 1280 },
        height: { ideal: 720 },
        frameRate: { ideal: 30 },
      },
    };

    return await navigator.mediaDevices.getUserMedia(constraints);
  }

  public toggleMute(): void {
    if (!this.localStream) return;

    this.localStream.getAudioTracks().forEach(track => {
      track.enabled = !track.enabled;
    });
  }

  public toggleCamera(): void {
    if (!this.localStream) return;

    this.localStream.getVideoTracks().forEach(track => {
      track.enabled = !track.enabled;
    });
  }

  public endCall(): void {
    if (this.localStream) {
      this.localStream.getTracks().forEach(track => track.stop());
      this.localStream = null;
    }

    if (this.peerConnection) {
      this.peerConnection.close();
      this.peerConnection = null;
    }
  }

  private async sendCallSignaling(peerId: string, data: any): Promise<void> {
    // Send via WRAITH protocol
    const { invoke } = await import('@tauri-apps/api/tauri');
    await invoke('send_call_signaling', { peerId, data: JSON.stringify(data) });
  }

  private async sendICECandidate(peerId: string, candidate: RTCIceCandidate): Promise<void> {
    await this.sendCallSignaling(peerId, {
      type: 'ice_candidate',
      candidate: candidate.toJSON(),
    });
  }
}
```

---

## Push Notifications

### Firebase Cloud Messaging (FCM)

```typescript
// src/notifications/PushNotifications.ts
import messaging from '@react-native-firebase/messaging';

export class PushNotifications {
  public async initialize(): Promise<void> {
    // Request permission
    const authStatus = await messaging().requestPermission();
    const enabled =
      authStatus === messaging.AuthorizationStatus.AUTHORIZED ||
      authStatus === messaging.AuthorizationStatus.PROVISIONAL;

    if (!enabled) {
      console.warn('Push notifications not authorized');
      return;
    }

    // Get FCM token
    const token = await messaging().getToken();
    console.log('FCM Token:', token);

    // Register token with backend (if using relay server)
    await this.registerToken(token);

    // Handle foreground messages
    messaging().onMessage(async remoteMessage => {
      console.log('Foreground message:', remoteMessage);
      this.showNotification(remoteMessage);
    });

    // Handle background messages
    messaging().setBackgroundMessageHandler(async remoteMessage => {
      console.log('Background message:', remoteMessage);
    });
  }

  private async registerToken(token: string): Promise<void> {
    // Register with relay server for wake-up notifications
    // (Actual messages still sent P2P via WRAITH)
    const { invoke } = await import('@tauri-apps/api/tauri');
    await invoke('register_push_token', { token });
  }

  private showNotification(message: any): void {
    // Show local notification
    const notification = {
      title: message.notification.title,
      body: message.notification.body,
      data: message.data,
    };

    // Use local notification API
  }
}
```

---

## Build and Deployment

### Android Build

```bash
# Development
npx react-native run-android

# Production APK
cd android
./gradlew assembleRelease

# Outputs: android/app/build/outputs/apk/release/app-release.apk
```

### iOS Build

```bash
# Development
npx react-native run-ios

# Production IPA
cd ios
xcodebuild -workspace WraithChat.xcworkspace \
  -scheme WraithChat \
  -configuration Release \
  -archivePath build/WraithChat.xcarchive \
  archive

xcodebuild -exportArchive \
  -archivePath build/WraithChat.xcarchive \
  -exportPath build \
  -exportOptionsPlist ExportOptions.plist
```

### Desktop Build

```bash
# Build with Tauri
npm run tauri build

# Outputs platform-specific bundles
```

---

## Testing

### Unit Tests

```typescript
describe('DoubleRatchet', () => {
  it('should encrypt and decrypt message', () => {
    const sharedSecret = crypto.randomBytes(32);
    const ratchet1 = new DoubleRatchet(sharedSecret);
    const ratchet2 = new DoubleRatchet(sharedSecret);

    const plaintext = Buffer.from('Hello, World!');
    const encrypted = ratchet1.encrypt(plaintext);
    const decrypted = ratchet2.decrypt(encrypted);

    expect(decrypted.toString()).toBe('Hello, World!');
  });

  it('should handle out-of-order messages', () => {
    const ratchet = new DoubleRatchet(crypto.randomBytes(32));

    const msg1 = ratchet.encrypt(Buffer.from('Message 1'));
    const msg2 = ratchet.encrypt(Buffer.from('Message 2'));
    const msg3 = ratchet.encrypt(Buffer.from('Message 3'));

    // Decrypt out of order
    const dec3 = ratchet.decrypt(msg3);
    const dec1 = ratchet.decrypt(msg1);
    const dec2 = ratchet.decrypt(msg2);

    expect(dec1.toString()).toBe('Message 1');
    expect(dec2.toString()).toBe('Message 2');
    expect(dec3.toString()).toBe('Message 3');
  });
});
```

---

## See Also

- [Architecture](architecture.md)
- [Features](features.md)
- [Client Overview](../overview.md)
