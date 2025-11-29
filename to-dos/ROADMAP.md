# WRAITH Protocol Development Roadmap

**Version:** 1.0.0
**Last Updated:** 2025-11-28
**Status:** Planning

---

## Executive Summary

This roadmap outlines the development plan for WRAITH Protocol and its ecosystem of client applications. The project is divided into two major tracks:

1. **Protocol Track:** Core protocol implementation (7 phases, ~12-18 months)
2. **Client Track:** 8 client applications (parallel development, ongoing)

---

## Protocol Development Timeline

### Phase Overview

| Phase | Focus | Duration | Story Points | Status |
|-------|-------|----------|--------------|--------|
| **Phase 1** | Foundation & Core Types | 4-6 weeks | 89 | Not Started |
| **Phase 2** | Cryptographic Layer | 4-6 weeks | 102 | Not Started |
| **Phase 3** | Transport & Kernel Bypass | 6-8 weeks | 156 | Not Started |
| **Phase 4** | Obfuscation & Stealth | 3-4 weeks | 76 | Not Started |
| **Phase 5** | Discovery & NAT Traversal | 5-7 weeks | 123 | Not Started |
| **Phase 6** | Integration & Testing | 4-5 weeks | 98 | Not Started |
| **Phase 7** | Hardening & Optimization | 6-8 weeks | 145 | Not Started |
| **Total** | | **32-44 weeks** | **789 points** | |

### Dependency Chart

```
Phase 1 (Foundation)
    ↓
Phase 2 (Crypto) ──────┐
    ↓                   │
Phase 3 (Transport) ────┤
    ↓                   │
Phase 4 (Obfuscation)───┤
    ↓                   │
Phase 5 (Discovery) ────┤
    ↓                   │
Phase 6 (Integration)←──┘
    ↓
Phase 7 (Hardening)
```

---

## Phase 1: Foundation (Weeks 1-6)

**Goal:** Establish core protocol types, frame encoding, and basic session management.

### Deliverables
- [ ] Frame encoding/decoding
- [ ] Session state machine
- [ ] Stream multiplexing
- [ ] Error handling framework
- [ ] Logging infrastructure
- [ ] Unit test framework

### Success Criteria
- Frame parsing benchmarks: >1M frames/sec
- Zero-copy frame parsing
- All frame types encodable/decodable
- Session transitions validated
- Test coverage >80%

**Story Points:** 89
**Risk Level:** Low (foundational work)

---

## Phase 2: Cryptographic Layer (Weeks 7-12)

**Goal:** Implement Noise_XX handshake, AEAD encryption, and key ratcheting.

### Deliverables
- [ ] X25519 key exchange
- [ ] Elligator2 encoding/decoding
- [ ] Noise_XX handshake (3 phases)
- [ ] XChaCha20-Poly1305 AEAD
- [ ] Symmetric ratchet
- [ ] DH ratchet
- [ ] BLAKE3 hashing
- [ ] Constant-time operations
- [ ] Memory zeroization
- [ ] Crypto test vectors

### Success Criteria
- Handshake completes in <50ms (LAN)
- Encryption throughput >3 GB/s (single core, x86_64 AVX2)
- All operations constant-time (verified)
- Forward secrecy validated
- Test coverage >90%

**Story Points:** 102
**Risk Level:** Medium (cryptographic correctness critical)

---

## Phase 3: Transport & Kernel Bypass (Weeks 13-20)

**Goal:** Implement AF_XDP sockets, XDP programs, and io_uring file I/O.

### Deliverables
- [ ] XDP/eBPF packet filter
- [ ] AF_XDP socket management
- [ ] UMEM allocation (NUMA-aware, huge pages)
- [ ] Ring buffer operations
- [ ] io_uring file I/O
- [ ] Worker thread model
- [ ] CPU pinning
- [ ] UDP fallback (non-XDP systems)
- [ ] MTU discovery
- [ ] Performance benchmarks

### Success Criteria
- XDP redirect rate: >24M pps (single core)
- AF_XDP zero-copy validated
- Throughput: >9 Gbps (10GbE hardware)
- Latency: <1μs (NIC to userspace)
- Fallback to UDP works seamlessly

**Story Points:** 156
**Risk Level:** High (kernel interaction, platform-specific)

---

## Phase 4: Obfuscation (Weeks 21-24)

**Goal:** Implement traffic obfuscation and protocol mimicry.

### Deliverables
- [ ] Packet padding (6 size classes)
- [ ] Timing obfuscation
- [ ] Cover traffic generator
- [ ] TLS record wrapper
- [ ] WebSocket wrapper
- [ ] DNS-over-HTTPS tunnel
- [ ] Padding mode selection
- [ ] Timing distribution sampling
- [ ] Obfuscation benchmarks

### Success Criteria
- Padding overhead: <20% (privacy mode)
- TLS mimicry passes DPI inspection
- Cover traffic maintains baseline rate
- Configurable obfuscation levels
- Performance impact <10% (privacy mode)

**Story Points:** 76
**Risk Level:** Medium (effectiveness difficult to validate)

---

## Phase 5: Discovery & NAT Traversal (Weeks 25-31)

**Goal:** Implement peer discovery, DHT, relays, and NAT hole punching.

### Deliverables
- [ ] Privacy-enhanced Kademlia DHT
- [ ] Encrypted announcements
- [ ] DHT query/store operations
- [ ] DERP-style relay protocol
- [ ] Relay client implementation
- [ ] NAT type detection
- [ ] STUN-like endpoint discovery
- [ ] Hole punching (simultaneous open)
- [ ] Birthday attack (symmetric NAT)
- [ ] Connection migration
- [ ] Path validation

### Success Criteria
- DHT lookup: <500ms (typical)
- Relay connection established: <200ms
- NAT traversal success rate: >90%
- Hole punching timeout: <5 seconds
- Graceful relay fallback

**Story Points:** 123
**Risk Level:** High (network complexity, NAT diversity)

---

## Phase 6: Integration (Weeks 32-36)

**Goal:** Integrate all components, file transfer engine, and comprehensive testing.

### Deliverables
- [ ] File chunking (256 KiB)
- [ ] BLAKE3 tree hashing
- [ ] Transfer state machine
- [ ] Resume/seek support
- [ ] Multi-peer parallel download
- [ ] Progress tracking
- [ ] BBR congestion control
- [ ] Flow control
- [ ] Loss detection & recovery
- [ ] CLI implementation
- [ ] Configuration system
- [ ] Integration tests

### Success Criteria
- Complete file transfer (1GB): <10 seconds (1 Gbps LAN)
- Resume works after interruption
- Multi-peer speedup: ~linear up to 5 peers
- BBR achieves >95% bandwidth utilization
- CLI functional for send/receive

**Story Points:** 98
**Risk Level:** Medium (integration complexity)

---

## Phase 7: Hardening & Optimization (Weeks 37-44)

**Goal:** Security audit, fuzzing, performance tuning, and production readiness.

### Deliverables
- [ ] Security audit (code review)
- [ ] Fuzzing (packet parsing, crypto)
- [ ] Property-based testing
- [ ] Performance profiling
- [ ] Memory profiling
- [ ] Bottleneck optimization
- [ ] Documentation (API, architecture)
- [ ] Deployment guide
- [ ] Monitoring/metrics
- [ ] Error recovery testing
- [ ] Cross-platform testing (Linux, macOS)
- [ ] Packaging (deb, rpm, cargo)

### Success Criteria
- No critical security issues
- Fuzz testing: 72 hours without crashes
- Performance targets met (see roadmap)
- Memory usage predictable
- Cross-platform builds succeed
- Documentation complete

**Story Points:** 145
**Risk Level:** Medium (security critical, time-consuming)

---

## Client Application Timeline

### Priority Tiers

**Tier 1 (High Priority):** Weeks 20-36 (parallel with protocol phases 4-6)
- WRAITH-Transfer (direct P2P file transfer)
- WRAITH-Chat (secure messaging)

**Tier 2 (Medium Priority):** Weeks 30-50 (starts during protocol phase 6)
- WRAITH-Sync (backup synchronization)
- WRAITH-Share (distributed file sharing)

**Tier 3 (Lower Priority):** Weeks 40-60 (after protocol complete)
- WRAITH-Stream (media streaming)
- WRAITH-Mesh (IoT networking)
- WRAITH-Publish (censorship-resistant publishing)
- WRAITH-Vault (distributed secret storage)

**Tier 3 (Security Testing - Specialized):** Weeks 44-70 (post-protocol hardening)
- WRAITH-Recon (reconnaissance & data transfer assessment)
- WRAITH-RedOps (red team operations platform)

**Note:** Security testing clients require completed protocol (Phase 7) and governance framework. See [Security Testing Parameters](../ref-docs/WRAITH-Security-Testing-Parameters-v1.0.md) for authorized use requirements.

### Client Development Phases

Each client follows 4-phase development:
1. **Design** (1-2 weeks): Architecture, API design
2. **Implementation** (3-6 weeks): Core functionality
3. **Testing** (2-3 weeks): Integration, UX testing
4. **Polish** (1-2 weeks): Documentation, packaging

### Security Testing Clients Detail

**WRAITH-Recon: Reconnaissance & Data Transfer Assessment**

**Purpose:** Authorized network reconnaissance and data exfiltration assessment platform.

**Governance Framework:** Requires signed Rules of Engagement (RoE), scope enforcement, kill switch capability, and tamper-evident audit logging.

**Development Timeline:** Weeks 44-56 (post-Phase 7)
- **Prerequisites:** Complete protocol implementation, wraith-crypto, wraith-transport, wraith-obfuscation
- **Duration:** 12 weeks (~55 story points)
- **Key Dependencies:** AF_XDP kernel bypass, protocol mimicry, governance controls

**Key Milestones:**
1. Governance & Safety Controller (cryptographic authorization, scope enforcement)
2. Reconnaissance Module (passive/active scanning, asset enumeration)
3. Transfer & Exfiltration Module (protocol mimicry, traffic shaping)
4. Audit & Reporting System (tamper-evident logs, compliance reporting)

**WRAITH-RedOps: Red Team Operations Platform**

**Purpose:** Comprehensive adversary emulation platform for authorized red team engagements.

**Governance Framework:** Executive authorization required, scope configuration, multi-operator audit trails, emergency kill switch.

**Development Timeline:** Weeks 56-70 (follows WRAITH-Recon)
- **Prerequisites:** Complete protocol + WRAITH-Recon governance patterns
- **Duration:** 14 weeks (~89 story points)
- **Key Dependencies:** Noise_XX handshake, multi-transport support, MITRE ATT&CK mapping

**Key Milestones:**
1. Team Server (multi-user C2, PostgreSQL state management, listener bus)
2. Operator Client (Tauri GUI, real-time session management, campaign tracking)
3. Spectre Implant (memory-resident agent, evasion techniques, P2P chaining)
4. Governance & Audit System (chain of custody, operations logging, reporting)

**MITRE ATT&CK Integration:** RedOps includes comprehensive technique mapping for:
- Initial Access, Execution, Persistence, Privilege Escalation
- Defense Evasion, Credential Access, Discovery, Lateral Movement
- Collection, Command and Control, Exfiltration, Impact

**Combined Timeline:** 26 weeks total for both security testing clients (Weeks 44-70)

---

## Performance Targets

### Protocol Layer

| Metric | Target | Stretch Goal |
|--------|--------|--------------|
| **Handshake Latency** | <50 ms (LAN) | <20 ms |
| **Throughput (1 Gbps)** | >800 Mbps | >950 Mbps |
| **Throughput (10 Gbps)** | >9 Gbps | >9.5 Gbps |
| **CPU @ 10 Gbps** | <80% (8 cores) | <50% |
| **Memory (per session)** | <10 MB | <5 MB |
| **Latency (NIC→App)** | <1 μs | <500 ns |

### Client Applications

| Client | First Byte Latency | Throughput | Concurrent Ops |
|--------|-------------------|------------|----------------|
| Transfer | <100 ms | Wire speed | 256 transfers |
| Chat | <50 ms | N/A | 10K messages/sec |
| Share | <500 ms (discovery) | Wire speed | 1000 swarms |
| Sync | <200 ms | Wire speed | 100 files |
| Stream | <200 ms | Wire speed | 100 streams |
| Mesh | <100 ms | 100 Mbps | 1000 devices |
| Publish | <1 sec (propagation) | N/A | 10K reads/sec |
| Vault | <500 ms | N/A | 1000 secrets |
| **Recon** | <10 ms (scan) | 300+ Mbps | 10K hosts/sec |
| **RedOps** | <50 ms (C2) | 300+ Mbps | 1000 beacons |

---

## Risk Management

### High-Risk Areas

**1. Kernel Bypass (Phase 3)**
- **Risk:** Platform-specific bugs, driver incompatibility
- **Mitigation:** Extensive testing, UDP fallback
- **Contingency:** Ship without XDP, optimize later

**2. NAT Traversal (Phase 5)**
- **Risk:** Low success rate on symmetric NAT
- **Mitigation:** Relay network, birthday attack optimization
- **Contingency:** Document known limitations

**3. Security Audit (Phase 7)**
- **Risk:** Critical vulnerabilities discovered late
- **Mitigation:** Early code review, fuzzing in Phase 6
- **Contingency:** Delay release, fix issues

**4. Performance Targets**
- **Risk:** Cannot achieve wire-speed throughput
- **Mitigation:** Profiling throughout, early benchmarks
- **Contingency:** Document actual performance, optimize post-release

### Staffing Risks

**Assumptions:**
- 2-3 full-time developers (protocol)
- 1-2 developers (clients, parallel work)
- 1 security reviewer (part-time, Phase 7)

**Contingency:**
- If understaffed: Cut Tier 3 clients, extend timeline
- If overstaffed: Parallelize clients, earlier completion

---

## Resource Requirements

### Development Environment

**Minimum:**
- Linux workstation (Ubuntu 22.04+, Fedora 38+)
- 4-core CPU, 16 GB RAM
- 1 Gbps network interface

**Recommended:**
- Linux workstation (kernel 6.6+)
- 8-16 core CPU (AMD Ryzen 9 / Intel i9)
- 32-64 GB RAM
- 10 Gbps NIC (Intel X710 / Mellanox ConnectX-5)
- NVMe SSD (2+ TB)

**Testing Hardware:**
- Multiple systems with different NICs
- Various NAT routers
- WiFi access points
- VPN servers for NAT testing

### External Dependencies

**Critical:**
- Rust toolchain (1.75+)
- Linux kernel 6.2+ (AF_XDP)
- libbpf, clang (XDP compilation)

**Optional:**
- Hardware security module (HSM) for key storage
- Cloud relay servers (for NAT traversal testing)

---

## Milestones & Release Strategy

### Alpha Release (End of Phase 3, Week 20)

**Features:**
- Basic send/receive functionality
- Encryption working
- UDP transport only (no XDP yet)
- Single-peer transfers
- CLI interface

**Audience:** Internal testing only

### Beta Release (End of Phase 6, Week 36)

**Features:**
- Full protocol implementation
- AF_XDP kernel bypass
- DHT peer discovery
- Relay/NAT traversal
- Multi-peer transfers
- WRAITH-Transfer client (Tier 1)

**Audience:** Early adopters, security researchers

### 1.0 Release (End of Phase 7, Week 44)

**Features:**
- Security audited
- Production-ready
- Cross-platform (Linux, macOS)
- WRAITH-Transfer + WRAITH-Chat clients
- Full documentation
- Deployment guides

**Audience:** General public

### Post-1.0 Roadmap

**v1.1 (Q1 2026):**
- Windows support (limited, no AF_XDP)
- WRAITH-Sync client
- Performance improvements

**v1.2 (Q2 2026):**
- WRAITH-Share client
- Post-quantum cryptography (hybrid mode)
- Mobile clients (Android/iOS)

**v2.0 (Q4 2026):**
- All Tier 2 & 3 clients
- Multipath transport
- Advanced obfuscation (ML-based)

---

## Success Metrics

### Technical Metrics
- [ ] All performance targets met
- [ ] Security audit passed (zero critical issues)
- [ ] Test coverage >85% (protocol), >70% (clients)
- [ ] Cross-platform compatibility (Linux, macOS)
- [ ] Fuzz testing: 72+ hours stable
- [ ] NAT traversal >90% success rate

### Adoption Metrics (Post-Launch)
- [ ] 10K+ downloads (first 3 months)
- [ ] 100+ active relay nodes
- [ ] 1K+ stars on GitHub
- [ ] Community contributions (PRs, issues)
- [ ] Production deployments (case studies)

### Community Metrics
- [ ] Documentation completeness: 100%
- [ ] Active discussion (Discord/Matrix)
- [ ] Third-party integrations
- [ ] Security researchers engaged
- [ ] Academic citations

---

## Dependencies & Blockers

### External Dependencies
- **Rust Language:** Stable 1.75+ (for all features)
- **Linux Kernel:** 6.2+ (for AF_XDP)
- **Cryptographic Libraries:** audited crates (dalek, RustCrypto)
- **DHT Implementation:** libp2p or custom
- **Build Tools:** xtask, cross-compilation support

### Potential Blockers
1. **Kernel API Changes:** AF_XDP API breaking changes in newer kernels
   - **Mitigation:** Track kernel development, maintain compatibility layers

2. **Cryptographic Vulnerabilities:** Discovered flaws in primitives (Curve25519, ChaCha20)
   - **Mitigation:** Follow IETF/CFRG announcements, prepare upgrade path

3. **Platform Restrictions:** XDP not supported on target hardware/drivers
   - **Mitigation:** UDP fallback mode, document requirements

4. **NAT Evolution:** New NAT types resistant to hole punching
   - **Mitigation:** Relay network expansion, alternative techniques

---

## Budget Estimate (Time/Resources)

### Development Time
- **Protocol:** 32-44 weeks (2-3 FTE developers)
- **Tier 1 Clients:** 16-24 weeks (1-2 FTE developers, parallel)
- **Tier 2 Clients:** 20-30 weeks (1-2 FTE developers, parallel)
- **Tier 3 Clients:** 24-36 weeks (1-2 FTE developers, deferred)
- **Total Project:** ~18-24 months to v2.0

### Infrastructure Costs (Annual)
- **Relay Servers:** 10-20 servers × $50/mo = $6,000-$12,000
- **DHT Bootstrap Nodes:** Included in relay servers
- **CI/CD:** GitHub Actions (free tier), self-hosted runners
- **Testing Hardware:** One-time $10,000-$20,000
- **Code Signing:** $300-$500/year
- **Domain/Hosting:** $500/year

**Total Annual (post-launch):** $7,000-$15,000

---

## Conclusion

This roadmap provides a structured path from protocol foundation to a complete ecosystem of client applications. The phased approach allows for:
- **Early validation:** Alpha/Beta releases for feedback
- **Risk mitigation:** Parallel development, fallback options
- **Flexibility:** Adjust timeline based on progress
- **Quality:** Security and performance baked in from start

**Target Completion:** 18-24 months to v2.0 with full ecosystem.

**Next Steps:**
1. Review and approve roadmap
2. Set up development environment
3. Begin Phase 1 (Foundation)
4. Establish CI/CD pipeline
5. Create project tracking (GitHub Projects)

---

**See Also:**
- [Phase 1 Sprint Planning](protocol/phase-1-foundation.md)
- [Phase 2 Sprint Planning](protocol/phase-2-crypto.md)
- [Client Application Plans](clients/)
