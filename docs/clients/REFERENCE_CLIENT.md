# WRAITH Reference Client Design

**Document Version:** 1.0.0
**Last Updated:** 2025-12-05
**Status:** Design Specification
**Target:** Cross-Platform Desktop GUI Application

---

## 1. Overview

This document provides design specifications for the WRAITH Reference Client, a cross-platform desktop GUI application that demonstrates best practices for building user-friendly interfaces on top of the WRAITH Protocol. The reference client serves both as a functional file transfer application and as a template for third-party client development.

**Design Philosophy:**
- **Security by Default:** Safe defaults with opt-in power features
- **Progressive Disclosure:** Simple for beginners, powerful for experts
- **Cross-Platform Consistency:** Native look-and-feel with consistent behavior
- **Accessibility First:** WCAG 2.1 Level AA compliance minimum
- **Performance Transparent:** Users understand what's happening and why

---

## 2. Application Architecture

### 2.1 Technology Stack

**Recommended Stack:**
- **Frontend Framework:** Tauri 2.0 (Rust backend + Web frontend)
- **UI Library:** React 18+ with TypeScript 5+
- **State Management:** Zustand (lightweight) or Redux Toolkit (complex apps)
- **Styling:** Tailwind CSS + shadcn/ui components
- **Icons:** Lucide React (consistent iconography)
- **Charts:** Recharts (performance graphs, transfer stats)

**Alternative Stacks:**
- **Native Performance:** egui (immediate mode GUI, pure Rust)
- **Qt/C++:** Qt6 with QML (enterprise deployments)
- **Electron:** Acceptable for web-first teams (higher memory usage)

### 2.2 Application Structure

```
wraith-reference-client/
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── main.rs         # Application entry point
│   │   ├── protocol.rs     # WRAITH protocol integration
│   │   ├── commands.rs     # Tauri IPC commands
│   │   └── state.rs        # Application state management
│   └── Cargo.toml
├── src/                    # React frontend
│   ├── components/         # UI components
│   │   ├── Transfer/       # File transfer UI
│   │   ├── Peers/          # Peer discovery and management
│   │   ├── Settings/       # Configuration UI
│   │   └── Dashboard/      # Main dashboard
│   ├── hooks/              # Custom React hooks
│   ├── lib/                # Utility functions
│   ├── types/              # TypeScript type definitions
│   └── App.tsx             # Root component
└── package.json
```

### 2.3 Backend Integration (Rust/Tauri)

```rust
// src-tauri/src/protocol.rs
use wraith_core::node::{Node, NodeConfig};
use wraith_files::FileTransfer;

#[tauri::command]
async fn send_file(
    state: tauri::State<'_, AppState>,
    file_path: String,
    peer_address: String,
) -> Result<String, String> {
    let node = state.node.lock().await;
    let transfer_id = node
        .send_file(&file_path, &peer_address)
        .await
        .map_err(|e| e.to_string())?;

    Ok(transfer_id)
}

#[tauri::command]
async fn get_transfer_progress(
    state: tauri::State<'_, AppState>,
    transfer_id: String,
) -> Result<f64, String> {
    let node = state.node.lock().await;
    node.get_transfer_progress(&transfer_id)
        .await
        .map_err(|e| e.to_string())
}
```

---

## 3. User Interface Design

### 3.1 Primary Window Layout

```
┌─────────────────────────────────────────────────────────────────────┐
│  WRAITH                                    [−] [□] [×]               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  ┌───────────┐                                                       │
│  │           │  Dashboard                                            │
│  │  Sidebar  │  ┌──────────────────────────────────────────────┐   │
│  │           │  │                                                │   │
│  │ • Transfer│  │  Recent Transfers                              │   │
│  │ • Peers   │  │  ┌──────────────────────────────────────────┐ │   │
│  │ • History │  │  │ document.pdf      ↓ Receiving   █░░░ 45% │ │   │
│  │ • Settings│  │  │ 12.5 MB / 27.8 MB │ 5.2 MB/s    2m left  │ │   │
│  │           │  │  └──────────────────────────────────────────┘ │   │
│  │           │  │  ┌──────────────────────────────────────────┐ │   │
│  │           │  │  │ video.mp4         ↑ Sending     ██████ 100%│  │
│  │           │  │  │ Complete          │ 15.3 MB/s   Done     │ │   │
│  │           │  │  └──────────────────────────────────────────┘ │   │
│  │           │  │                                                │   │
│  │           │  │  Network Stats                                 │   │
│  │           │  │  • Active Peers: 3                             │   │
│  │           │  │  • Throughput: 8.7 MB/s                        │   │
│  │           │  │  • Latency: 12 ms                              │   │
│  │           │  │                                                │   │
│  │           │  └──────────────────────────────────────────────┘   │
│  └───────────┘                                                       │
│                                                                       │
└─────────────────────────────────────────────────────────────────────┘
```

**Key Elements:**
- **Sidebar Navigation:** Always visible, collapsible on small screens
- **Main Content Area:** Dynamic based on selected navigation item
- **Status Bar:** (optional) Connection status, background task indicators
- **Action Buttons:** Context-sensitive (Send File, Add Peer, etc.)

### 3.2 Transfer View

**Primary Actions:**
- **Send File Button:** Prominent, always accessible
- **Drag & Drop Zone:** Full-window drop target when no active transfers
- **Quick Send:** Right-click file in system file manager → Send via WRAITH

**Transfer Card Design:**

```
┌──────────────────────────────────────────────────────────────┐
│  document.pdf                                [⋯]            │
│  ↓ Receiving from 192.0.2.10                                 │
├──────────────────────────────────────────────────────────────┤
│  Progress: ███████░░░░░░░░░░░░░░░░ 45% (12.5 MB / 27.8 MB) │
│  Speed: 5.2 MB/s     ETA: 2m 15s     Peers: 1/3              │
├──────────────────────────────────────────────────────────────┤
│  [Pause]  [Cancel]  [Show in Folder]                         │
└──────────────────────────────────────────────────────────────┘
```

**Transfer States:**
- **Initiating:** "Connecting to peer..."
- **Handshake:** "Establishing secure connection..."
- **Active:** Progress bar with speed/ETA
- **Paused:** "Transfer paused (click Resume)"
- **Complete:** "Complete (12.5 MB in 2m 15s)"
- **Failed:** "Transfer failed: Connection lost" with [Retry] button
- **Cancelled:** "Transfer cancelled by user"

### 3.3 Peer Discovery View

```
┌──────────────────────────────────────────────────────────────┐
│  Discovered Peers                        [Add Peer Manually]│
├──────────────────────────────────────────────────────────────┤
│  ┌────────────────────────────────────────────────────────┐ │
│  │  Alice's Laptop                           [Connect]     │ │
│  │  192.0.2.10:41641 │ 12 ms latency │ ●  Online         │ │
│  │  Last seen: 2 minutes ago                              │ │
│  └────────────────────────────────────────────────────────┘ │
│  ┌────────────────────────────────────────────────────────┐ │
│  │  Bob's Desktop                            [Connect]     │ │
│  │  2001:db8::1:41641 │ 45 ms latency │ ● Online         │ │
│  │  Last seen: 5 seconds ago                              │ │
│  └────────────────────────────────────────────────────────┘ │
│  ┌────────────────────────────────────────────────────────┐ │
│  │  Charlie's Server                         [Connect]     │ │
│  │  NAT'd (via relay) │ 150 ms latency │ ● Online        │ │
│  │  Last seen: 30 seconds ago                             │ │
│  └────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────┘
```

**Peer Information Display:**
- **Peer Name:** User-friendly identifier (editable)
- **Address:** IP:port or "NAT'd (via relay)"
- **Latency:** Color-coded (green <50ms, yellow 50-150ms, red >150ms)
- **Status:** Online/Offline with indicator dot
- **Last Seen:** Human-readable relative time

**Manual Peer Addition Dialog:**

```
┌──────────────────────────────────────────────────────────────┐
│  Add Peer                                         [×]        │
├──────────────────────────────────────────────────────────────┤
│  Peer Address:                                               │
│  ┌────────────────────────────────────────────────────────┐ │
│  │ 192.0.2.10:41641                                       │ │
│  └────────────────────────────────────────────────────────┘ │
│                                                               │
│  Peer Name (optional):                                       │
│  ┌────────────────────────────────────────────────────────┐ │
│  │ Alice's Laptop                                         │ │
│  └────────────────────────────────────────────────────────┘ │
│                                                               │
│  [ ] Save peer for future connections                        │
│                                                               │
│                              [Cancel]  [Add Peer]            │
└──────────────────────────────────────────────────────────────┘
```

### 3.4 Settings View

**Organized into Tabs:**

#### General Settings
- **Download Directory:** File picker with [Browse] button
- **Notifications:** Toggle for desktop notifications
- **Auto-Resume:** Enable resume after network disruption
- **Language:** Dropdown for internationalization

#### Network Settings
- **Listen Port:** Numeric input (default: 41641)
- **Enable UPnP/NAT-PMP:** Toggle for automatic port forwarding
- **Enable DHT:** Toggle for peer discovery
- **Bootstrap Nodes:** List of DHT bootstrap nodes (editable)
- **Relay Servers:** List of relay servers for NAT traversal
- **Connection Timeout:** Slider (5-60 seconds)

#### Performance Settings
- **Transfer Mode:**
  - ○ Standard (UDP)
  - ○ High Performance (AF_XDP) - requires root/admin
- **Worker Threads:** Slider (1-16, auto-detect by default)
- **Maximum Concurrent Transfers:** Slider (1-256)
- **Bandwidth Limit:** Input with unit selector (MB/s, Mbps)

#### Privacy Settings
- **Obfuscation Mode:**
  - ○ None (maximum performance)
  - ○ Basic (padding only)
  - ○ Standard (padding + timing jitter)
  - ○ High Privacy (full obfuscation + cover traffic)
- **Protocol Mimicry:**
  - ○ None
  - ○ TLS 1.3
  - ○ WebSocket
  - ○ DNS-over-HTTPS
- **Clear History on Exit:** Toggle

#### Security Settings
- **Node Keypair:** Display public key with [Export] [Regenerate] buttons
- **Peer Verification:** Toggle for manual peer key verification
- **Trusted Peers:** List of pinned peer public keys

### 3.5 Transfer History View

```
┌──────────────────────────────────────────────────────────────┐
│  Transfer History                [Clear History] [Export]    │
├──────────────────────────────────────────────────────────────┤
│  Filters: [All] [Sent] [Received]   Search: [___________]   │
├──────────────────────────────────────────────────────────────┤
│  Dec 5, 2025                                                 │
│  ─────────────────────────────────────────────────────────── │
│  ↓ document.pdf (27.8 MB) from Alice's Laptop       Complete│
│     2:34 PM │ 5.2 MB/s │ 2m 15s                              │
│                                                               │
│  ↑ video.mp4 (450.2 MB) to Bob's Desktop            Complete│
│     1:15 PM │ 15.3 MB/s │ 30m 5s                             │
│                                                               │
│  Dec 4, 2025                                                 │
│  ─────────────────────────────────────────────────────────── │
│  ↓ archive.zip (1.2 GB) from Charlie's Server       Complete│
│     11:22 AM │ 8.7 MB/s │ 2h 15m                             │
└──────────────────────────────────────────────────────────────┘
```

**Features:**
- **Filtering:** By direction (sent/received), date range, peer
- **Search:** Full-text search across filenames and peer names
- **Export:** CSV/JSON export for audit trails
- **Context Menu:** Right-click → [Send Again] [Show in Folder] [Delete from History]

---

## 4. User Experience Guidelines

### 4.1 Progressive Disclosure

**Beginner Mode (Default):**
- Simple Send/Receive workflow
- Auto-discovery of peers
- Safe default settings
- Minimal configuration required

**Advanced Mode (Opt-In):**
- Manual peer management
- Advanced network configuration
- Performance tuning
- Privacy/obfuscation controls

### 4.2 Error Handling

**Principles:**
- **User-Friendly Messages:** No technical jargon unless necessary
- **Actionable Guidance:** Always suggest next steps
- **Contextual Help:** Link to relevant documentation

**Example Error Messages:**

❌ **Bad:** "Error: ECONNREFUSED at socket.connect"

✅ **Good:** "Cannot connect to peer at 192.0.2.10:41641"
- **Why:** The peer may be offline or behind a firewall
- **What to do:**
  - Check that the peer address is correct
  - Verify the peer application is running
  - [Troubleshooting Guide] [Try Again]

### 4.3 Loading States

**Transfer Initiation:**
- "Connecting to peer..." (animated spinner)
- "Establishing secure connection..." (progress indicator)
- "Starting transfer..." (fade in to active transfer UI)

**Peer Discovery:**
- "Searching for peers..." (animated dots)
- "Found 3 peers" (success state with count)

**Settings Changes:**
- "Applying settings..." (brief loading overlay)
- "Settings saved" (toast notification)

### 4.4 Feedback and Notifications

**Types:**
- **Success:** Green toast notification (3s duration)
  - "Transfer complete: document.pdf (27.8 MB)"
- **Info:** Blue toast notification (3s duration)
  - "New peer discovered: Alice's Laptop"
- **Warning:** Yellow toast notification (5s duration)
  - "Transfer paused: Network connection lost"
- **Error:** Red toast notification (persistent until dismissed)
  - "Transfer failed: Insufficient disk space"

**System Notifications:**
- Transfer complete (with thumbnail if available)
- Incoming transfer request (with accept/decline actions)
- Peer comes online (if user has favorited)

---

## 5. Accessibility

### 5.1 Keyboard Navigation

**Global Shortcuts:**
- `Ctrl/Cmd + N`: New transfer (send file)
- `Ctrl/Cmd + O`: Open received files folder
- `Ctrl/Cmd + ,`: Settings
- `Ctrl/Cmd + H`: Transfer history
- `Ctrl/Cmd + Q`: Quit application
- `Esc`: Cancel current dialog/modal

**Navigation:**
- `Tab`/`Shift+Tab`: Navigate between interactive elements
- `Enter`: Activate focused button/link
- `Space`: Toggle checkbox/toggle
- `Arrow Keys`: Navigate lists/grids

### 5.2 Screen Reader Support

**ARIA Labels:**
- All interactive elements have descriptive labels
- Progress bars announce percentage at 10% intervals
- Status updates use `role="status"` for live regions

**Example:**
```tsx
<button
  aria-label="Send file to Alice's Laptop"
  onClick={handleSendFile}
>
  <SendIcon aria-hidden="true" />
  Send File
</button>
```

### 5.3 Visual Accessibility

**Color Contrast:**
- Minimum 4.5:1 for normal text
- Minimum 3:1 for large text
- Status indicators use icons in addition to color

**Font Sizing:**
- Base font size: 16px
- Support for OS-level text scaling (up to 200%)
- User-configurable zoom (Ctrl +/−)

**Focus Indicators:**
- Visible keyboard focus outlines (2px solid)
- High contrast focus indicators in all themes

---

## 6. Platform-Specific Considerations

### 6.1 Windows

**Integration:**
- Windows 10/11 native title bar and controls
- File Explorer context menu: "Send via WRAITH"
- System tray icon with quick actions
- Windows Defender SmartScreen: Code signing certificate required

**Permissions:**
- AF_XDP mode requires Administrator elevation
- Prompt with clear explanation: "High Performance mode requires Administrator access to use kernel networking features"

### 6.2 macOS

**Integration:**
- Native macOS title bar (follow system accent color)
- Finder context menu: "Send via WRAITH"
- Menu bar icon with status menu
- Apple Notarization for Gatekeeper

**Permissions:**
- Network extension entitlement for AF_XDP
- Prompt for network access permission (first launch)

### 6.3 Linux

**Integration:**
- GTK theme integration (detect light/dark)
- Nautilus/Dolphin context menu integration
- System tray icon (AppIndicator)
- .desktop file with MIME type associations

**Permissions:**
- AF_XDP requires `CAP_NET_RAW` capability or root
- Provide instructions for setting capabilities: `sudo setcap cap_net_raw+ep wraith-client`

---

## 7. Security Indicators

### 7.1 Connection Security Visualization

**Handshake Progress:**
```
┌──────────────────────────────────────────────────────────────┐
│  Establishing Secure Connection                              │
├──────────────────────────────────────────────────────────────┤
│  ✓ Ephemeral key exchange                                    │
│  ✓ Peer authentication                                       │
│  ⋯ Deriving session keys...                                  │
└──────────────────────────────────────────────────────────────┘
```

**Active Connection:**
- Lock icon next to peer name (green padlock)
- Tooltip: "End-to-end encrypted with XChaCha20-Poly1305"

### 7.2 Peer Verification

**First-Time Peer:**
```
┌──────────────────────────────────────────────────────────────┐
│  Verify Peer Identity                                        │
├──────────────────────────────────────────────────────────────┤
│  You are connecting to:                                      │
│    Name: Alice's Laptop                                      │
│    Address: 192.0.2.10:41641                                 │
│                                                               │
│  Peer Public Key Fingerprint:                                │
│    B4A7 3F21 8D9C 5E14 A6F2 7B89 C3D5 0E41                  │
│                                                               │
│  Verify this fingerprint with the peer via a separate        │
│  secure channel (phone call, Signal message, etc.)           │
│                                                               │
│  [ ] I have verified this fingerprint                        │
│  [ ] Always trust this peer                                  │
│                                                               │
│                      [Cancel]  [Connect Securely]            │
└──────────────────────────────────────────────────────────────┘
```

---

## 8. Implementation Checklist

**Phase 1: Core Functionality**
- [ ] File send/receive with progress tracking
- [ ] Peer discovery via DHT
- [ ] Manual peer addition
- [ ] Basic settings (download directory, port)
- [ ] Transfer history (in-memory, not persistent)

**Phase 2: User Experience**
- [ ] Drag & drop file sending
- [ ] Desktop notifications
- [ ] System tray/menu bar integration
- [ ] Transfer resumption after network disruption
- [ ] Persistent transfer history (SQLite)

**Phase 3: Advanced Features**
- [ ] Multi-peer swarm downloads
- [ ] Bandwidth limiting
- [ ] Advanced privacy settings (obfuscation, mimicry)
- [ ] Peer verification and pinning
- [ ] AF_XDP high-performance mode

**Phase 4: Polish**
- [ ] Internationalization (i18n)
- [ ] Accessibility testing (screen reader, keyboard-only)
- [ ] Platform-specific integrations (context menus)
- [ ] Automatic updates (Tauri updater)
- [ ] Crash reporting (optional, opt-in)

---

## 9. Testing Requirements

**Functional Testing:**
- [ ] Send file (small, medium, large, jumbo)
- [ ] Receive file with progress updates
- [ ] Resume interrupted transfer
- [ ] Cancel active transfer
- [ ] Peer discovery (DHT, manual)
- [ ] Settings persistence across restarts

**Platform Testing:**
- [ ] Windows 10/11 (x86_64)
- [ ] macOS 12+ (Intel + Apple Silicon)
- [ ] Linux (Ubuntu 22.04, Fedora 39, Arch)

**Accessibility Testing:**
- [ ] Keyboard-only navigation
- [ ] Screen reader testing (NVDA, JAWS, VoiceOver)
- [ ] High contrast mode
- [ ] Font scaling (up to 200%)

**Performance Testing:**
- [ ] Transfer 1 GB file over LAN (target: 500+ MB/s)
- [ ] Simultaneous transfers (10 concurrent)
- [ ] Memory usage (idle: <100 MB, active: <500 MB)
- [ ] CPU usage (idle: <5%, active: <40%)

---

## 10. Resources

**Design Assets:**
- [Figma Design File](#) - High-fidelity mockups
- [Tauri Getting Started](https://tauri.app/start/) - Tauri documentation
- [shadcn/ui](https://ui.shadcn.com/) - Component library
- [Lucide Icons](https://lucide.dev/) - Icon library

**Implementation References:**
- [WRAITH Protocol Rust API](../INTEGRATION_GUIDE.md) - Protocol integration guide
- [Tauri IPC Guide](https://tauri.app/develop/calling-rust/) - Rust ↔ JavaScript communication
- [WCAG 2.1](https://www.w3.org/WAI/WCAG21/quickref/) - Accessibility guidelines

---

**Document Revision History:**

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2025-12-05 | Initial reference client design specification |

---

*End of Reference Client Design*
