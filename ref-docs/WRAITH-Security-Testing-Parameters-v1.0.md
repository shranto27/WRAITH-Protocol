# WRAITH Protocol: Security Testing Parameters & Governance Framework

**Document Version:** 1.0  
**Classification:** Development Documentation  
**Last Updated:** November 2025  
**Author:** WRAITH Development Team

---

## 1. Executive Summary

This document defines the authorized use cases, governance framework, and operational parameters for WRAITH (Working Routing Architecture for Invisible Transfer Handling) when deployed in security testing contexts. WRAITH incorporates advanced networking capabilities that, while designed for legitimate privacy and secure communication, possess dual-use potential requiring explicit governance controls.

This documentation serves to:

- Articulate the legitimate security research and testing applications of WRAITH
- Define the authorization frameworks governing deployment
- Specify technical safeguards that constrain use to authorized contexts
- Establish audit and accountability mechanisms
- Align development practices with industry standards and legal requirements

**This document does not constitute authorization for any specific engagement.** Authorization must be obtained through the appropriate mechanisms defined herein for each deployment context.

---

## 2. WRAITH Capability Overview

### 2.1 Core Protocol Capabilities

WRAITH provides the following capabilities relevant to security testing contexts:

| Capability | Description | Security Testing Application |
|------------|-------------|------------------------------|
| **Polymorphic Framing** | Dynamic packet structure that adapts to network conditions | Evasion testing against DPI/IDS systems |
| **Protocol Mimicry** | Traffic patterns that resemble benign protocols (HTTPS, DNS, etc.) | Network security control validation |
| **Decentralized Routing** | Multi-hop relay architecture without central coordination | Infrastructure resilience testing |
| **Encrypted Payload Transport** | End-to-end encryption with forward secrecy | Data protection validation |
| **Covert Channel Establishment** | Tunnel creation through restrictive network boundaries | Egress filtering assessment |
| **Fragmented Exfiltration** | Data transfer via distributed, reassembled chunks | DLP (Data Loss Prevention) control testing |

### 2.2 Dual-Use Capability Assessment

The capabilities listed above are functionally identical to techniques employed by Advanced Persistent Threats (APTs) and malware frameworks. This is intentional—effective security testing requires tools that replicate real-world threat actor TTPs (Tactics, Techniques, and Procedures).

The distinction between legitimate and malicious use is determined entirely by:

1. **Authorization** — Explicit permission from system owners
2. **Scope** — Defined boundaries of engagement
3. **Intent** — Defensive improvement vs. exploitation
4. **Accountability** — Logging, reporting, and oversight

---

## 3. Authorized Use Cases

### 3.1 Contracted Penetration Testing Engagements

**Description:** WRAITH may be deployed during penetration testing engagements where a formal contractual relationship exists between the testing organization and the target organization.

**Authorization Requirements:**

- Signed Master Services Agreement (MSA) or Statement of Work (SOW)
- Explicit Rules of Engagement (RoE) document specifying:
  - In-scope IP ranges, domains, and systems
  - Out-of-scope systems and data categories
  - Authorized testing timeframes
  - Emergency contact procedures
  - Data handling requirements
- Written authorization from an individual with legal authority to grant access
- Liability and indemnification provisions

**Applicable WRAITH Capabilities:**

- Protocol mimicry for IDS/IPS evasion testing
- Covert channel establishment for egress control validation
- Fragmented exfiltration for DLP effectiveness assessment
- Decentralized routing for network segmentation testing

**Constraints:**

- All exfiltrated data must be synthetic test data or explicitly authorized samples
- Production data exfiltration requires explicit written authorization and secure handling procedures
- Persistent implants require explicit authorization and documented removal procedures
- All activities must be logged and provided to the client in the final report

---

### 3.2 Red Team Operations

**Description:** WRAITH may serve as a Command and Control (C2) framework component and exfiltration mechanism during authorized red team exercises simulating advanced threat actors.

**Authorization Requirements:**

- Executive-level authorization from target organization leadership
- Defined adversary emulation objectives (e.g., "simulate APT29 TTPs")
- Coordination with Blue Team leadership (for purple team exercises) or isolation protocols (for blind red team exercises)
- Incident response bypass procedures to prevent unnecessary escalation
- Legal review confirming authorization scope

**Applicable WRAITH Capabilities:**

- Full C2 channel establishment and maintenance
- Long-term persistence mechanism testing
- Lateral movement facilitation
- Data staging and exfiltration simulation
- Evasion of endpoint detection and response (EDR) tools

**Operational Framework:**

```
┌─────────────────────────────────────────────────────────────────┐
│                    RED TEAM ENGAGEMENT FLOW                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  [Authorization]──▶[Scoping]──▶[Infrastructure Setup]          │
│         │              │                  │                     │
│         ▼              ▼                  ▼                     │
│  Legal Review     Define RoE      Deploy WRAITH Nodes           │
│  Executive Sign   Target List     Configure Logging             │
│  Emergency POC    Timeframes      Establish C2 Channels         │
│                                                                 │
│  [Execution]──▶[Documentation]──▶[Remediation Support]         │
│         │              │                  │                     │
│         ▼              ▼                  ▼                     │
│  Phased Ops       Activity Logs    Finding Debrief              │
│  Check-ins        Screenshots      Detection Gap Analysis       │
│  Abort Criteria   Artifacts        Cleanup Verification         │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Constraints:**

- Implant persistence must not survive beyond engagement end date (time-limited execution or manual removal)
- All C2 infrastructure must be decommissioned within 48 hours of engagement conclusion
- No access to systems outside defined scope, even if discovered
- Immediate notification if critical vulnerabilities affecting safety are discovered

---

### 3.3 Capture The Flag (CTF) Competitions

**Description:** WRAITH may be used as an offensive tool within sanctioned CTF competition environments where network exploitation, data exfiltration, and evasion are explicit competition objectives.

**Authorization Requirements:**

- Competition rules explicitly permitting the use of custom C2/exfiltration tools
- Confirmation that competition infrastructure is isolated from production systems
- Acknowledgment of competition Code of Conduct

**Applicable WRAITH Capabilities:**

- All capabilities may be used within competition scope
- No restrictions on evasion, persistence, or exfiltration techniques within competition boundaries

**Constraints:**

- WRAITH deployment must be limited to competition-designated networks
- No attacks against competition infrastructure itself (scoring servers, VPN endpoints, etc.) unless explicitly in scope
- No use against other competitors' personal systems outside competition environment
- Compliance with competition-specific tool disclosure requirements (if any)

---

### 3.4 Security Research & Academic Testing

**Description:** WRAITH capabilities may be studied, tested, and documented in controlled laboratory environments for the purpose of advancing defensive security knowledge.

**Authorization Requirements:**

- Institutional Review Board (IRB) approval for research involving human subjects or production data
- Laboratory environment isolation verification
- Research ethics compliance documentation
- For vulnerability research: Coordinated disclosure commitment

**Applicable WRAITH Capabilities:**

- All capabilities may be tested in isolated laboratory environments
- Analysis of evasion effectiveness against security controls
- Development of detection signatures and behavioral indicators
- Performance benchmarking under various network conditions

**Research Outputs:**

Security research using WRAITH should contribute to the defensive community through:

- Detection signature development (YARA, Suricata, Sigma rules)
- Behavioral analysis documentation
- Network traffic pattern characterization
- Recommendations for control improvements

**Constraints:**

- No testing against production systems without explicit authorization
- No release of functional exploit code without coordinated disclosure
- Dual-use research findings must be reviewed for responsible disclosure considerations
- Student researchers require faculty advisor oversight

---

## 4. Prohibited Uses

The following uses of WRAITH are explicitly prohibited regardless of claimed justification:

| Prohibited Activity | Rationale |
|---------------------|-----------|
| Unauthorized access to any system | Violation of CFAA and equivalent statutes |
| Deployment against systems without explicit owner authorization | No implicit authorization exists |
| Exfiltration of real sensitive data without explicit data handling authorization | Privacy violations, potential regulatory breach |
| Use in ransomware, destructive malware, or wiper attacks | Causes irreversible harm |
| Mass targeting or indiscriminate scanning | Scope violations, potential legal exposure |
| Supply chain compromise | Affects parties outside authorization scope |
| Attacks against critical infrastructure without government coordination | National security implications |
| Targeted attacks against individuals (stalking, harassment, doxing) | Criminal activity |
| Sale or distribution to parties without vetting | Loss of accountability chain |

---

## 5. Technical Safeguards

### 5.1 Scope Enforcement Mechanisms

WRAITH implementations intended for security testing should incorporate the following technical controls:

**Target Whitelisting:**
```
[scope]
allowed_targets = [
    "192.168.1.0/24",
    "10.0.50.0/24",
    "*.target-company.local"
]
enforcement = "hard"  # Refuse connections outside scope
```

**Time-Bounded Execution:**
```
[engagement]
start_time = "2025-12-01T00:00:00Z"
end_time = "2025-12-15T23:59:59Z"
action_on_expiry = "self_terminate"
```

**Kill Switch Implementation:**
```
[safety]
kill_switch_enabled = true
kill_switch_endpoints = [
    "https://c2.operator-domain.com/killswitch",
    "dns:kill.operator-domain.com"
]
check_interval_seconds = 300
```

### 5.2 Logging and Audit Requirements

All WRAITH deployments in security testing contexts must maintain comprehensive logs:

| Log Category | Contents | Retention |
|--------------|----------|-----------|
| Connection Log | Timestamps, source/destination IPs, ports, protocol used | Duration of engagement + 1 year |
| Command Log | All commands issued through C2 channel | Duration of engagement + 1 year |
| Data Transfer Log | Hashes of all transferred files, sizes, timestamps | Duration of engagement + 1 year |
| Error Log | Failed operations, blocked attempts, exceptions | Duration of engagement + 1 year |

**Log Integrity:**
- Logs must be cryptographically signed to prevent tampering
- Logs should be transmitted to secure, off-node storage in real-time where possible
- Log encryption at rest is mandatory

### 5.3 Data Handling Controls

For engagements involving data exfiltration testing:

- **Synthetic Data Preference:** Use generated test data that mimics production data patterns without containing real sensitive information
- **Production Data Authorization:** If real data exfiltration is required for testing, obtain explicit written authorization specifying:
  - Data categories authorized for exfiltration
  - Maximum volume limits
  - Secure handling requirements
  - Destruction timeline and verification method
- **In-Transit Encryption:** All exfiltrated data must be encrypted with keys controlled by authorized operators
- **Data Segregation:** Exfiltrated test data must be stored separately from operator production systems

---

## 6. Legal and Regulatory Framework

### 6.1 Applicable Laws and Regulations

WRAITH operators must ensure compliance with applicable legal frameworks, including but not limited to:

**United States:**
- Computer Fraud and Abuse Act (CFAA), 18 U.S.C. § 1030
- Electronic Communications Privacy Act (ECPA)
- State-specific computer crime statutes
- Sector-specific regulations (HIPAA, GLBA, etc.) if applicable data is involved

**International:**
- UK Computer Misuse Act 1990
- EU Directive on Attacks Against Information Systems (2013/40/EU)
- Local equivalents in jurisdiction of operation

**Key Legal Principle:** Authorization from the system owner is the primary legal defense against computer crime charges. This authorization must be:
- Explicit (not implied)
- Documented (written preferred)
- Scoped (specific systems and timeframes)
- Granted by an authorized party (someone with legal authority to consent)

### 6.2 Regulatory Considerations

Penetration testing involving regulated data requires additional considerations:

| Regulation | Consideration |
|------------|---------------|
| PCI-DSS | Pentester must be qualified; testing must follow PCI penetration testing guidance |
| HIPAA | BAA may be required if PHI is accessed; minimum necessary principle applies |
| GDPR | Data subject rights apply to any personal data accessed during testing |
| ITAR/EAR | WRAITH itself may be subject to export controls depending on cryptographic implementation |

---

## 7. Alignment with Industry Standards

### 7.1 Penetration Testing Frameworks

WRAITH deployment should align with established penetration testing methodologies:

- **PTES (Penetration Testing Execution Standard):** Pre-engagement, intelligence gathering, threat modeling, vulnerability analysis, exploitation, post-exploitation, reporting
- **OWASP Testing Guide:** For web application components
- **NIST SP 800-115:** Technical Guide to Information Security Testing and Assessment
- **MITRE ATT&CK:** Adversary TTP mapping for red team operations

### 7.2 Responsible Disclosure

If WRAITH testing reveals vulnerabilities in third-party software or services:

1. Document the vulnerability with sufficient detail for reproduction
2. Notify the vendor through established security contact channels
3. Provide reasonable time for remediation (typically 90 days)
4. Coordinate public disclosure timing with vendor
5. Do not weaponize vulnerabilities for unauthorized use

---

## 8. Operator Qualifications and Accountability

### 8.1 Operator Requirements

Individuals deploying WRAITH in security testing contexts should possess:

- Demonstrated expertise in network security and penetration testing
- Understanding of applicable legal frameworks
- Familiarity with industry-standard testing methodologies
- Commitment to ethical conduct and responsible disclosure

**Recommended Credentials (not mandatory):**
- OSCP, OSCE, OSEP (Offensive Security)
- GPEN, GXPN (GIAC)
- CEH, LPT (EC-Council)
- CREST certifications

### 8.2 Accountability Chain

All WRAITH deployments must maintain a clear accountability chain:

```
┌─────────────────────────────────────────────────────────────────┐
│                     ACCOUNTABILITY CHAIN                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  [Authorizing Party]                                            │
│         │                                                       │
│         │ Grants authorization via signed RoE/Contract          │
│         ▼                                                       │
│  [Engagement Lead]                                              │
│         │                                                       │
│         │ Responsible for scope compliance, operator oversight  │
│         ▼                                                       │
│  [Operator(s)]                                                  │
│         │                                                       │
│         │ Execute testing within authorized scope               │
│         ▼                                                       │
│  [WRAITH Instance]                                              │
│         │                                                       │
│         │ Logs all activity, enforces technical constraints     │
│         ▼                                                       │
│  [Audit Trail]                                                  │
│         │                                                       │
│         └──▶ Available for review, incident response, legal    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 9. Incident Response Provisions

### 9.1 Unintended Impact Procedures

If WRAITH testing causes unintended system impact:

1. **Immediate:** Cease all testing activities
2. **Notify:** Contact emergency POC defined in RoE within 15 minutes
3. **Document:** Preserve all logs and artifacts related to the incident
4. **Assist:** Provide support for incident response and recovery as needed
5. **Review:** Conduct post-incident analysis to prevent recurrence

### 9.2 Scope Violation Response

If operators discover they have inadvertently accessed out-of-scope systems:

1. **Disconnect:** Immediately terminate connection to out-of-scope system
2. **Document:** Record how the access occurred
3. **Notify:** Inform engagement lead and client contact
4. **Purge:** Delete any data obtained from out-of-scope systems
5. **Assess:** Determine if the issue represents a finding (e.g., inadequate network segmentation)

---

## 10. Development Considerations

### 10.1 Secure Development Practices

WRAITH development should incorporate:

- **Code Review:** All capability additions reviewed for unintended consequences
- **Access Control:** Development builds restricted to authorized developers
- **Version Control:** Full git history maintained for accountability
- **Build Integrity:** Reproducible builds with cryptographic verification
- **Distribution Control:** Release binaries signed and distributed through controlled channels

### 10.2 Defensive Contribution

To support the defensive security community, WRAITH development should include:

- Publication of network traffic indicators for detection development
- Documentation of behavioral patterns for EDR/XDR detection
- Collaboration with security vendors on detection capabilities (under NDA if needed)
- Contribution to threat intelligence sharing communities (with appropriate sanitization)

---

## 11. Document Control

### 11.1 Review and Updates

This document should be reviewed and updated:

- Annually at minimum
- Upon significant capability additions to WRAITH
- Upon changes to applicable legal or regulatory frameworks
- Following any incident requiring lessons-learned incorporation

### 11.2 Distribution

This document may be distributed to:

- WRAITH development team members
- Authorized operators
- Clients and partners evaluating WRAITH for authorized use
- Legal counsel for compliance review

---

## Appendix A: Sample Rules of Engagement Template

```markdown
# Rules of Engagement
## [Client Name] Penetration Test

**Engagement ID:** [UNIQUE-ID]
**Effective Dates:** [START] to [END]

### Authorization
I, [NAME], [TITLE] of [ORGANIZATION], authorize [TESTING COMPANY] to 
perform penetration testing against the systems defined below.

### In-Scope Systems
- [IP RANGES]
- [DOMAINS]
- [APPLICATIONS]

### Out-of-Scope Systems
- [CRITICAL SYSTEMS]
- [THIRD-PARTY HOSTED]
- [SPECIFIC EXCLUSIONS]

### Authorized Activities
- [ ] Network scanning and enumeration
- [ ] Vulnerability exploitation
- [ ] Social engineering (specify types)
- [ ] Physical security testing
- [ ] Data exfiltration simulation
- [ ] Persistence testing
- [ ] Evasion testing

### Prohibited Activities
- [ ] Denial of service
- [ ] Destructive actions
- [ ] Production data exfiltration (unless separately authorized)
- [ ] [OTHER RESTRICTIONS]

### Emergency Contacts
- Primary: [NAME], [PHONE], [EMAIL]
- Secondary: [NAME], [PHONE], [EMAIL]

### Signatures
_________________________    _________________________
Authorizing Party            Testing Lead
Date:                        Date:
```

---

## Appendix B: Glossary

| Term | Definition |
|------|------------|
| **APT** | Advanced Persistent Threat — sophisticated, long-term targeted attack campaigns |
| **C2** | Command and Control — infrastructure used to communicate with implants |
| **CFAA** | Computer Fraud and Abuse Act — primary US federal computer crime statute |
| **CTF** | Capture The Flag — competitive security exercise |
| **DLP** | Data Loss Prevention — controls to prevent unauthorized data exfiltration |
| **DPI** | Deep Packet Inspection — network analysis examining packet contents |
| **EDR** | Endpoint Detection and Response — endpoint security monitoring |
| **IDS/IPS** | Intrusion Detection/Prevention System — network security monitoring |
| **RoE** | Rules of Engagement — documented scope and authorization for testing |
| **TTP** | Tactics, Techniques, and Procedures — adversary behavioral patterns |

---

*End of Document*
