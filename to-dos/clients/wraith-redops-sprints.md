# WRAITH-RedOps Client - Sprint Planning (Granular)

**Client Name:** WRAITH-RedOps
**Tier:** 3 (Advanced)
**Timeline:** 16 weeks (4 sprints × 4 weeks)
**Total Story Points:** 240
**Protocol Alignment:** Synchronized with core protocol development (Phases 1-5)
**Governance:** [Security Testing Parameters](../../ref-docs/WRAITH-Security-Testing-Parameters-v1.0.md)

**WRAITH Protocol Stack Dependencies:**
| Crate | Purpose | Integration Phase |
|-------|---------|-------------------|
| wraith-core | C2 session management, stream multiplexing (cmd/file/SOCKS) | Phase 1 (Weeks 40-43) |
| wraith-crypto | Noise_XX C2 handshake, XChaCha20-Poly1305, key ratcheting | Phase 2 (Weeks 44-47) |
| wraith-transport | AF_XDP (Linux beacons), UDP/TCP/HTTPS/DNS transports | Phase 1-2 (Weeks 40-47) |
| wraith-obfuscation | C2 traffic obfuscation (padding, timing, TLS/DNS mimicry) | Phase 2-3 (Weeks 44-51) |
| wraith-discovery | P2P beacon mesh (DHT), relay coordination, NAT traversal | Phase 3 (Weeks 48-51) |
| wraith-files | Data exfiltration chunking, BLAKE3 integrity, multi-path | Phase 3-4 (Weeks 48-55) |

**Protocol Milestones:**
- ✓ Core frame encoding complete (wraith-core v0.1.0)
- ✓ Basic UDP transport functional (wraith-transport v0.1.0)
- ⏳ Noise_XX C2 handshake (wraith-crypto v0.2.0) - **Critical for Beacon-Server auth**
- ⏳ AF_XDP kernel bypass (wraith-transport v0.2.0) - **Linux beacon performance**
- ⏳ Protocol mimicry (wraith-obfuscation v0.1.0) - **C2 stealth (TLS/DNS/ICMP)**
- ⏳ P2P beacon mesh (wraith-discovery v0.1.0) - **Lateral movement support**
- ⏳ Multi-path exfiltration (wraith-files v0.1.0) - **Data exfil resilience**

**MITRE ATT&CK Technique Integration:**
- T1071 (Application Layer Protocol) - HTTPS/DNS C2 channels
- T1041 (Exfiltration Over C2 Channel) - Multi-path data exfiltration
- T1090 (Proxy: Multi-hop Proxy) - P2P beacon mesh, relay network
- T1055 (Process Injection) - BOF loader, shellcode execution
- T1027 (Obfuscated Files or Information) - Elligator2, protocol mimicry

---

## Phase 1: Command Infrastructure (Weeks 40-43)
**Protocol Dependencies:** wraith-core v0.1.0, wraith-transport v0.1.0 (UDP), wraith-crypto v0.1.0 (basic AEAD)
**ATT&CK Focus:** T1071 (C2 Protocol), T1132 (Data Encoding)

### S1.1: Team Server Core (25 pts)
*   [ ] **Task:** Setup Async Rust Project (Axum/Tokio).
    *   *Acceptance Criteria:* Project compiles, HTTP server listens on configured port, graceful shutdown works.
*   [ ] **Task:** Implement Database migrations (Sqlx/Postgres).
    *   *Acceptance Criteria:* Schema initializes correctly on empty DB; migrations up/down work without data loss.
*   [ ] **Task:** Define gRPC Protos (`c2.proto`, `admin.proto`).
    *   *Acceptance Criteria:* `.proto` files compile to Rust structs; client/server can exchange Hello message.
*   [ ] **Task:** Implement Listener Trait and UDP Listener.
    *   *Acceptance Criteria:* Trait allows pluggable transports; UDP listener handles 1000 concurrent connection states.
*   [ ] **Task:** Implement `TaskQueue` logic with priority support.
    *   *Acceptance Criteria:* Tasks are persisted to DB; High priority tasks fetched first.

### S1.2: Operator Client (25 pts)
*   [ ] **Task:** Scaffold Tauri App (Vite + React + TS).
    *   *Acceptance Criteria:* App launches on Windows/Linux/Mac; connects to backend API.
*   [ ] **Task:** Implement Auth Logic (JWT + mTLS).
    *   *Acceptance Criteria:* Login screen enforces mTLS; JWT token stored securely; 401 redirects to login.
*   [ ] **Task:** Create Session Grid Component.
    *   *Acceptance Criteria:* Real-time updates of Last Seen; sorting/filtering works.
*   [ ] **Task:** Integrate `xterm.js` for Beacon Console.
    *   *Acceptance Criteria:* Interactive terminal supports colors, history, and copy/paste.
*   [ ] **Task:** Implement file upload/download manager UI.
    *   *Acceptance Criteria:* Drag-and-drop uploads; progress bars for large files.

---

## Phase 2: The "Spectre" Implant - Core (Weeks 44-47)
**Protocol Dependencies:** wraith-crypto v0.2.0 (Noise_XX, Elligator2, ratcheting), wraith-obfuscation v0.1.0 (padding, timing), wraith-transport v0.2.0 (multi-transport)
**ATT&CK Focus:** T1071.001 (Web Protocols), T1573 (Encrypted Channel), T1027.002 (Software Packing)
**Testing Milestone:** Cryptographic C2 channel (Noise handshake, AEAD encryption, nonce uniqueness, key ratcheting)

### S2.1: `no_std` Foundation (30 pts)
*   [ ] **Task:** Create `no_std` crate layout.
    *   *Acceptance Criteria:* Compiles to standalone PE file; Imports only `ntdll.dll` (or zero imports).
*   [ ] **Task:** Implement `PanicHandler` (Abort/Loop).
    *   *Acceptance Criteria:* Panic does not crash host process; optional debug logging to ring buffer.
*   [ ] **Task:** Implement `ApiResolver` (Hash-based import resolution).
    *   *Acceptance Criteria:* Resolve `VirtualAlloc` by hash (djb2/ror13) without string presence in binary.
*   [ ] **Task:** Implement `MiniHeap` allocator (Static array backing).
    *   *Acceptance Criteria:* `Box`, `Vec`, `String` work in `no_std` environment.
*   [ ] **Task:** Write Entry Point Assembly (Stack alignment).
    *   *Acceptance Criteria:* Stack aligned to 16 bytes; registers saved/restored; Reflective loader compatibility.

### S2.2: WRAITH Integration (30 pts)
*   [ ] **Task:** Port `wraith-crypto` to `no_std`.
    *   *Acceptance Criteria:* ChaCha20/Poly1305 passes test vectors without std lib.
*   [ ] **Task:** Implement WinSock (UDP) via Syscalls.
    *   *Acceptance Criteria:* Send/Recv UDP packets using only direct syscalls (no `ws2_32.dll`).
*   [ ] **Task:** Implement C2 Loop (Poll -> Dispatch -> Sleep).
    *   *Acceptance Criteria:* Beacon checks in at interval; executes task; sleeps with jitter.
*   [ ] **Task:** Implement Command Dispatcher (Match opcode).
    *   *Acceptance Criteria:* Correctly parses PDU; routes to handler function; handles invalid opcodes gracefully.

---

## Phase 3: Tradecraft & Evasion (Weeks 48-51)
**Protocol Dependencies:** wraith-discovery v0.1.0 (P2P mesh, relay), wraith-files v0.1.0 (exfiltration), all prior crates integrated
**ATT&CK Focus:** T1055 (Process Injection), T1090 (Multi-hop Proxy), T1041 (Exfiltration Over C2), T1027.005 (Indicator Removal)
**Testing Milestone:** Evasion effectiveness (EDR bypass validation, network stealth testing, protocol mimicry verification)

### S3.1: Advanced Loader (35 pts)
*   [ ] **Task:** Implement Hell's Gate Syscall resolver.
    *   *Acceptance Criteria:* Dynamically find SSNs from ntdll.dll; handle hooked functions.
*   [ ] **Task:** Implement ROP Chain generator for Sleep Mask.
    *   *Acceptance Criteria:* Memory is RX during execution, RW during sleep; spoofed return address.
*   [ ] **Task:** Implement Stack Spoofing (Frame rewriting).
    *   *Acceptance Criteria:* Call stack looks legitimate (e.g., rooted in `kernel32!BaseThreadInitThunk`) during sleep.
*   [ ] **Task:** Implement AMSI Patching logic.
    *   *Acceptance Criteria:* `AmsiScanBuffer` returns `AMSI_RESULT_CLEAN` always; patch applied in memory only.

### S3.2: Post-Exploitation Features (25 pts)
*   [ ] **Task:** Implement COFF Loader (BOF support).
    *   *Acceptance Criteria:* Load standard Cobalt Strike BOFs; resolve symbols; capture output.
*   [ ] **Task:** Implement SOCKS4a Server state machine.
    *   *Acceptance Criteria:* Tunnel TCP traffic; handle Connect/Bind requests; high throughput.
*   [ ] **Task:** Implement File System VFS (WinAPI wrappers).
    *   *Acceptance Criteria:* `ls`, `cd`, `pwd`, `cat` work reliably; handle restricted permissions.
*   [ ] **Task:** Implement Token Manipulation (Steal Token).
    *   *Acceptance Criteria:* Impersonate logged-on user; revert to self.

---

## Phase 4: Lateral Movement & Polish (Weeks 52-55)
**Protocol Dependencies:** All WRAITH crates integrated and tested, focus on P2P mesh optimization and multi-path exfiltration
**ATT&CK Focus:** T1021 (Remote Services), T1090.001 (Internal Proxy), T1105 (Ingress Tool Transfer), T1567 (Exfiltration Over Web Service)
**Testing Milestone:** End-to-end C2 reliability (P2P mesh routing, multi-transport failover, data exfiltration integrity, MITRE ATT&CK technique validation)

### S4.1: Peer-to-Peer C2 (30 pts)
*   [ ] **Task:** Implement Named Pipe Server/Client.
    *   *Acceptance Criteria:* Parent beacon creates pipe; Child connects; Frames routed bidirectionally.
*   [ ] **Task:** Implement Routing Logic (Mesh forwarding).
    *   *Acceptance Criteria:* Packets traverse A->B->C->Server correctly; loop detection.
*   [ ] **Task:** Update Team Server Graph to render P2P links.
    *   *Acceptance Criteria:* UI shows parent-child relationships visually.

### S4.2: Automation & Builder (40 pts)
*   [ ] **Task:** Implement LLVM/LLD invocation logic.
    *   *Acceptance Criteria:* Server invokes linker; produces valid PE/ELF.
*   [ ] **Task:** Implement Config Patcher (Byte replacement).
    *   *Acceptance Criteria:* C2 domain/key replaced in `.data` section without recompilation.
*   [ ] **Task:** Implement Obfuscation Pass (LLVM Pass).
    *   *Acceptance Criteria:* Control flow graph flattened; strings encrypted.
*   [ ] **Task:** Write Aggressor Script (Lua) bindings.
    *   *Acceptance Criteria:* Hooks for `on_beacon_initial`; API for `task_shell`.
*   [ ] **Task:** Perform Final Red Team Simulation (E2E).
    *   *Acceptance Criteria:* Complete kill chain execution (Phishing -> Domain Admin) in lab.