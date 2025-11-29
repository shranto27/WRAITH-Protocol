# Protocol Naming Compendium

## Candidate Names for the Decentralized Secure File Transfer Protocol

**Document Version:** 1.0.0-DRAFT  
**Purpose:** Name Selection and Branding Reference  
**Total Candidates:** 60 names across multiple thematic categories  

---

## Selection Criteria

Names were selected based on the following criteria:

1. **Memorability** — Easy to recall and spell
2. **Relevance** — Connects to core protocol properties (stealth, speed, security, decentralization)
3. **Uniqueness** — Not already used by major projects
4. **Pronounceability** — Works across languages
5. **Domain/Package Availability** — Potential for unique namespace
6. **Acronym Potential** — Forms meaningful abbreviations

---

## Category 1: Stealth & Evasion Themes

Names inspired by the protocol's traffic obfuscation and covert channel capabilities.

| # | Name | Shorthand | Description |
|---|------|-----------|-------------|
| 1 | **Specter** | SPEC | Like a ghost in the network—present but undetectable. Traffic passes through firewalls and IDS systems as if invisible, leaving no recognizable signature. |
| 2 | **Phantom** | PHTM | Exists in the shadows of legitimate traffic. Packets appear as ordinary HTTPS or DNS while carrying encrypted file data through hostile networks. |
| 3 | **Wraith** | WRT | An ethereal presence that cannot be grasped. Elligator2-encoded handshakes and padded payloads make the protocol indistinguishable from random noise. |
| 4 | **Shade** | SHD | Operates in the shade of other protocols. Mimicry modes allow traffic to blend seamlessly with WebSocket, TLS, or DNS-over-HTTPS flows. |
| 5 | **Mirage** | MRG | What observers see isn't real. Statistical traffic analysis reveals patterns that match legitimate services, not covert file transfers. |
| 6 | **Cloak** | CLK | Wraps file transfers in layers of obfuscation. Padding, timing jitter, and cover traffic create an impenetrable cloak around actual data. |
| 7 | **Veil** | VL | A thin but effective barrier between observers and truth. Lightweight obfuscation for performance-sensitive deployments. |
| 8 | **Umbra** | UMB | The darkest part of a shadow. Maximum stealth mode with full traffic shaping, protocol mimicry, and constant-rate transmission. |
| 9 | **Penumbra** | PNUM | The partial shadow—balanced between stealth and speed. Adaptive obfuscation that scales with threat level. |
| 10 | **Eclipse** | ECLS | Blocks all visibility into transfer activity. Complete traffic analysis resistance through sophisticated timing and padding. |

---

## Category 2: Speed & Performance Themes

Names emphasizing the protocol's high-throughput, low-latency capabilities.

| # | Name | Shorthand | Description |
|---|------|-----------|-------------|
| 11 | **Flux** | FLX | Continuous, rapid flow of data. AF_XDP acceleration enables 10+ Gbps throughput with sub-millisecond latency on commodity hardware. |
| 12 | **Torrent** | TRT | An unstoppable surge of data. Parallel streams and BBR congestion control maximize bandwidth utilization across any network path. |
| 13 | **Surge** | SRG | Burst capability when needed, controlled flow otherwise. Adaptive rate control responds to network conditions in real-time. |
| 14 | **Bolt** | BLT | Lightning-fast transfers. Zero-copy packet processing and io_uring file I/O eliminate unnecessary memory operations. |
| 15 | **Pulse** | PLS | Rhythmic, efficient data heartbeat. Optimized packet pacing prevents congestion while maintaining maximum throughput. |
| 16 | **Rush** | RSH | Urgency without chaos. Priority streams ensure critical small files transfer instantly while large transfers continue in background. |
| 17 | **Swift** | SWT | Elegant speed through careful design. Every protocol decision optimizes for minimal round trips and maximum parallelism. |
| 18 | **Rapid** | RPD | Fast by default, faster when needed. Kernel bypass and CPU pinning extract every bit of performance from modern hardware. |
| 19 | **Streak** | STK | Continuous high-speed operation. Persistent connections and session resumption eliminate handshake overhead for repeated transfers. |
| 20 | **Blitz** | BLZ | Overwhelming speed for time-critical transfers. Expedited streams bypass normal flow control for sub-second small file delivery. |

---

## Category 3: Security & Cryptography Themes

Names highlighting the protocol's strong security guarantees.

| # | Name | Shorthand | Description |
|---|------|-----------|-------------|
| 21 | **Bastion** | BSTN | An impregnable fortress for your data. XChaCha20-Poly1305 encryption with forward secrecy protects every byte in transit. |
| 22 | **Citadel** | CTD | Defense in depth. Multiple cryptographic layers—Noise handshake, AEAD transport, key ratcheting—ensure no single point of failure. |
| 23 | **Aegis** | AGS | Divine protection for digital assets. Authenticated encryption prevents tampering; replay protection defeats injection attacks. |
| 24 | **Rampart** | RMP | Strong walls against adversaries. Post-compromise security means breached keys don't expose past or future sessions. |
| 25 | **Vault** | VLT | Secure storage in transit. End-to-end encryption ensures only intended recipients can access transferred files. |
| 26 | **Sentinel** | SNTL | Vigilant guardian of data integrity. BLAKE3 tree hashing verifies every chunk; corrupted data is detected and rejected. |
| 27 | **Warden** | WDN | Strict access control. Mutual authentication via Noise_XX ensures only authorized peers can establish connections. |
| 28 | **Shield** | SHLD | First line of defense. Stateless reset tokens and path validation protect against connection hijacking and spoofing. |
| 29 | **Cipher** | CPHR | Encryption at its core. Every design decision prioritizes cryptographic security without compromising usability. |
| 30 | **Enigma** | ENGM | Indecipherable to outsiders. Elligator2 encoding makes even key exchange appear as random data to observers. |

---

## Category 4: Decentralization & Network Themes

Names reflecting the protocol's peer-to-peer, trustless architecture.

| # | Name | Shorthand | Description |
|---|------|-----------|-------------|
| 31 | **Mesh** | MSH | Interconnected without hierarchy. Privacy-enhanced DHT enables peer discovery without central servers or authorities. |
| 32 | **Nexus** | NXS | Connection point in a distributed web. Any peer can relay, discover, or transfer—no single point of failure or control. |
| 33 | **Lattice** | LTC | Structured yet flexible. Kademlia-based routing finds optimal paths through the network with logarithmic lookup complexity. |
| 34 | **Web** | WEB | Interwoven connections. Multi-path transfers and NAT traversal ensure connectivity regardless of network topology. |
| 35 | **Swarm** | SWM | Collective intelligence. Parallel downloads from multiple peers maximize throughput and provide redundancy. |
| 36 | **Hive** | HV | Collaborative by nature. Group-based discovery allows trusted peers to share files without public announcement. |
| 37 | **Colony** | CLN | Self-organizing network. Peers autonomously maintain routing tables and relay connectivity for the collective good. |
| 38 | **Grid** | GRD | Distributed infrastructure. DERP-style relays provide fallback connectivity without compromising end-to-end encryption. |
| 39 | **Node** | ND | Fundamental building block. Every participant is equal—no privileged servers, no gatekeepers, no central authority. |
| 40 | **Cluster** | CLST | Groups of cooperating peers. Locality-aware peer selection minimizes latency and maximizes throughput for nearby nodes. |

---

## Category 5: Nature & Element Themes

Names drawn from natural phenomena that parallel protocol behavior.

| # | Name | Shorthand | Description |
|---|------|-----------|-------------|
| 41 | **Obsidian** | OBS | Dark, sharp, and impenetrable. Like volcanic glass, the protocol is formed under pressure and reveals nothing of its contents. |
| 42 | **Onyx** | ONX | Deep black stone of protection. Symbolizes the protocol's role as a guardian of data confidentiality and integrity. |
| 43 | **Nebula** | NBL | Vast, diffuse, and impossible to grasp. Traffic disperses across the network like interstellar gas, defying observation. |
| 44 | **Aether** | ATHR | The medium through which data flows invisibly. Named for the classical element thought to fill the universe above Earth. |
| 45 | **Zephyr** | ZPH | A gentle but persistent wind. Steady, reliable transfers that flow around obstacles without force or detection. |
| 46 | **Tempest** | TMPS | Controlled chaos when needed. Burst mode unleashes full bandwidth capacity for time-critical large file transfers. |
| 47 | **Aurora** | AUR | Beautiful yet ephemeral. Sessions appear briefly, transfer data, and vanish—leaving no persistent trace in the network. |
| 48 | **Meridian** | MRD | The line connecting distant points. Direct peer connections span the globe with minimal latency through optimized routing. |
| 49 | **Drift** | DFT | Gradual, unnoticed movement. Data flows through the network at rates that blend with background traffic patterns. |
| 50 | **Vapor** | VPR | Present but intangible. Transfers complete and evidence evaporates—forward secrecy ensures no recoverable session history. |

---

## Category 6: Mythological & Abstract Themes

Names with deeper symbolic meaning relating to protocol properties.

| # | Name | Shorthand | Description |
|---|------|-----------|-------------|
| 51 | **Hermes** | HRMS | Messenger of the gods, guide of souls. Swift, secure delivery of data between any two points in the network. |
| 52 | **Charon** | CHRN | Ferryman across the river Styx. Transports data across hostile network boundaries that would block ordinary protocols. |
| 53 | **Nyx** | NYX | Primordial goddess of night. Embodies the protocol's operation in darkness, invisible to all observers. |
| 54 | **Lethe** | LTH | River of forgetfulness. Forward secrecy ensures past sessions are forgotten—even by those who participated. |
| 55 | **Proteus** | PRTS | Shape-shifting sea god. Protocol mimicry transforms traffic to appear as HTTPS, WebSocket, or DNS at will. |
| 56 | **Argus** | ARGS | Hundred-eyed giant. Comprehensive integrity verification watches every byte for tampering or corruption. |
| 57 | **Styx** | STX | Unbreakable oath river. Cryptographic authentication creates binding proof of peer identity that cannot be forged. |
| 58 | **Erebus** | ERBS | Primordial darkness. The deepest level of obfuscation where traffic is indistinguishable from cosmic background noise. |
| 59 | **Prometheus** | PMTH | Bringer of fire to humanity. Democratizes secure communication—military-grade cryptography for everyone. |
| 60 | **Atlas** | ATLS | Titan bearing the weight of the sky. Robust infrastructure carrying the burden of global secure file transfer. |

---

## Recommended Primary Candidates

Based on comprehensive evaluation against selection criteria:

### Top 5 Recommendations

| Rank | Name | Rationale |
|------|------|-----------|
| **1** | **Specter** | Perfectly captures stealth aspect; memorable; short; good CLI feel (`specter send file.dat`); available namespace |
| **2** | **Flux** | Emphasizes speed/flow; modern sound; 4 letters; works as verb ("flux the file over"); tech-forward connotation |
| **3** | **Obsidian** | Strong imagery; implies both darkness (stealth) and hardness (security); distinctive; memorable |
| **4** | **Nebula** | Evokes distributed, hard-to-observe nature; scientifically cool; unique in protocol space |
| **5** | **Wraith** | Gaming/tech culture resonance; implies invisibility; short; punchy; good acronym potential (WRAITH) |

### Honorable Mentions

| Name | Strength |
|------|----------|
| **Umbra** | Perfect stealth metaphor; Latin elegance |
| **Cipher** | Direct security reference; classic feel |
| **Drift** | Subtle; implies undetectable movement |
| **Zephyr** | Gentle but persistent; poetic |
| **Proteus** | Shape-shifting captures mimicry perfectly |

---

## Acronym Expansions

For names that could serve as acronyms:

| Acronym | Expansion | Notes |
|---------|-----------|-------|
| **SPECTER** | **S**ecure **P**rivate **E**ncrypted **C**overt **T**ransfer with **E**vasion and **R**esilience | Primary recommendation |
| **FLUX** | **F**ast **L**ow-latency **U**ndetectable e**X**change | Emphasizes speed + stealth |
| **WRAITH** | **W**ire-speed **R**esilient **A**uthenticated **I**nvisible **T**ransfer **H**andler | Technical accuracy |
| **SHADE** | **S**ecure **H**idden **A**synchronous **D**ecentralized **E**xchange | Covers all bases |
| **DRIFT** | **D**ecentralized **R**esilient **I**nvisible **F**ile **T**ransfer | Clean and descriptive |
| **CLOAK** | **C**overt **L**ow-latency **O**bfuscated **A**uthenticated **K**ernel-accelerated | Technical focus |
| **PULSE** | **P**rivate **U**ndetectable **L**ightweight **S**ecure **E**xchange | Lightweight variant |
| **MESH** | **M**utual **E**ncryption **S**tealth **H**andoff | P2P emphasis |
| **AEGIS** | **A**uthenticated **E**ncrypted **G**lobal **I**nvisible **S**haring | Security focus |
| **NEBULA** | **N**etwork-**E**vasive **B**urst-capable **U**niversal **L**ow-latency **A**rchitecture | Full technical scope |

---

## CLI Command Examples

How each top candidate would feel in practical use:

```bash
# Specter
specter send document.pdf alice@keybase
specter receive --output ./downloads
specter daemon --config /etc/specter/config.toml
specter status
specter peers list

# Flux  
flux push report.docx peer:abc123def456
flux pull file:hash789 --parallel 8
fluxd --interface eth0 --port 0
flux transfers --active

# Obsidian
obsidian transfer secret.zip --to peer.key --encrypt
obsidian serve --stealth full
obsidian discover "project-files" --group teamkey
obd status --json

# Nebula
nebula send --file archive.tar.gz --recipient 0x1234...
nebula watch ./incoming
nebulad --relay wss://relay.example.com
nebula network stats

# Wraith
wraith tx data.bin --peer pubkey.txt --mimicry https
wraith rx --bind 0.0.0.0:0 --output /data
wraithd --xdp --interface ens192
wraith conn list
```

---

## Logo Concept Associations

Visual concepts that pair well with each name:

| Name | Visual Concept |
|------|----------------|
| **Specter** | Translucent figure; fading edges; ghost outline merging with circuit patterns |
| **Flux** | Flowing lines; liquid metal aesthetic; continuous motion blur |
| **Obsidian** | Sharp angular black crystal; reflective dark surface; volcanic glass shard |
| **Nebula** | Swirling cosmic clouds; purple/blue gas with embedded stars; diffuse edges |
| **Wraith** | Hooded figure dissolving into mist; smoke-like tendrils; shadow emerging from network nodes |
| **Umbra** | Perfect black circle (eclipse shadow); gradient from light to absolute dark |
| **Cipher** | Interlocking geometric shapes; encrypted text fragments; key/lock fusion |
| **Drift** | Smooth curved lines; floating particles; gentle wave patterns |
| **Aegis** | Classical shield outline; protective dome; layered defensive rings |
| **Mesh** | Interconnected nodes; geodesic structure; web of equal connections |

---

## Domain Availability Check (Suggested)

Domains to investigate for availability:

```
specter-protocol.io / specterxfer.io / getspecter.dev
flux-transfer.io / fluxproto.dev / useflux.io  
obsidian-xfer.io / obsidianproto.dev
nebula-transfer.io / nebulaproto.io
wraith-protocol.io / wraithxfer.dev / getwraith.io
```

**Package Registry Names:**
```
crates.io: specter-proto, flux-xfer, obsidian-net, nebula-p2p, wraith-fs
npm: @specter/cli, @flux-protocol/core, @obsidian-xfer/client
pypi: specter-protocol, flux-transfer, obsidian-p2p
```

---

## Final Recommendation

**Primary Name: SPECTER**

**Rationale:**

1. **Stealth First:** The core differentiator of this protocol is traffic invisibility—"Specter" captures this perfectly
2. **Memorable:** Single word, easy to spell, universally understood concept
3. **CLI Ergonomics:** `specter send file.dat` feels natural and professional
4. **Acronym Ready:** SPECTER = Secure Private Encrypted Covert Transfer with Evasion and Resilience
5. **Namespace Available:** Likely available on major package registries
6. **Cultural Resonance:** Familiar from film/gaming but not overused in tech
7. **Scalable Brand:** Works for both serious security tools and consumer-friendly apps

**Tagline Options:**
- "Your files, invisible in transit"
- "Secure. Fast. Undetectable."
- "The ghost in the network"
- "Military-grade stealth for everyone"
- "Transfer without a trace"

---

*End of Protocol Naming Compendium*
