# WRAITH-Share Architecture

**Document Version:** 1.0.0
**Last Updated:** 2025-11-28
**Client Version:** 1.0.0

---

## Overview

WRAITH-Share provides secure group file sharing with cryptographic access control. Users create shared folders, invite members with specific permissions, and collaborate without central servers.

**Design Goals:**
- Support 100+ member groups
- Granular permissions (read/write/admin)
- File versioning (10 versions per file)
- Activity log for audit trail
- Link sharing with expiration
- Cross-platform web and desktop access

---

## Architecture Diagram

```
┌──────────────────────────────────────────────────────┐
│               User Interface Layer                    │
│  ┌────────────────┐  ┌──────────────────────────┐   │
│  │  Desktop GUI   │  │     Web PWA              │   │
│  │  (Tauri)       │  │   (React)                │   │
│  └────────────────┘  └──────────────────────────┘   │
└──────────────────────────────────────────────────────┘
                         │
┌──────────────────────────────────────────────────────┐
│            Application Logic Layer                    │
│  ┌──────────────────────────────────────────────┐   │
│  │  Group Manager                               │   │
│  │  - Create/manage groups                      │   │
│  │  - Member permissions                        │   │
│  │  - Invitation system                         │   │
│  └──────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────┐   │
│  │  File Manager                                │   │
│  │  - Upload/download files                     │   │
│  │  - Version tracking                          │   │
│  │  - Permission enforcement                    │   │
│  └──────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────┐   │
│  │  Access Control (Capability-Based)           │   │
│  │  - Generate capabilities                     │   │
│  │  - Verify permissions                        │   │
│  │  - Revoke access                             │   │
│  └──────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────┘
                         │
┌──────────────────────────────────────────────────────┐
│         Database Layer (SQLite)                      │
│  - Groups, members, permissions                      │
│  - Files, versions, metadata                         │
│  - Activity log                                      │
└──────────────────────────────────────────────────────┘
                         │
┌──────────────────────────────────────────────────────┐
│         WRAITH Protocol Stack                        │
│  (encrypted file transfer + DHT)                     │
└──────────────────────────────────────────────────────┘
```

---

## Components

### 1. Group Manager

**Responsibilities:**
- Group lifecycle (create/delete)
- Member management (invite/remove)
- Permission assignment
- Group metadata (name, description, avatar)

**Data Structures:**
```rust
pub struct Group {
    pub id: GroupId,
    pub name: String,
    pub description: String,
    pub members: Vec<GroupMember>,
    pub created_at: SystemTime,
    pub created_by: PeerId,
}

pub struct GroupMember {
    pub peer_id: PeerId,
    pub display_name: String,
    pub role: GroupRole,
    pub capabilities: Vec<Capability>,
    pub joined_at: SystemTime,
}

pub enum GroupRole {
    Admin,   // Full control
    Write,   // Can upload/modify files
    Read,    // Can download files only
}
```

---

### 2. Access Control System

**Capability-Based Security:**
```rust
pub struct Capability {
    pub id: CapabilityId,
    pub group_id: GroupId,
    pub permissions: Permissions,
    pub issued_by: PeerId,
    pub issued_to: PeerId,
    pub expires_at: Option<SystemTime>,
    pub signature: Signature,
}

pub struct Permissions {
    pub read: bool,
    pub write: bool,
    pub delete: bool,
    pub invite: bool,
}

impl Capability {
    pub fn verify(&self, action: Action) -> Result<()> {
        // Check expiration
        if let Some(expires) = self.expires_at {
            if SystemTime::now() > expires {
                return Err(Error::CapabilityExpired);
            }
        }

        // Check permission
        match action {
            Action::Read => {
                if !self.permissions.read {
                    return Err(Error::PermissionDenied);
                }
            }
            Action::Write => {
                if !self.permissions.write {
                    return Err(Error::PermissionDenied);
                }
            }
            // ...
        }

        // Verify signature
        self.verify_signature()?;

        Ok(())
    }
}
```

---

### 3. File Version Manager

**Purpose:** Track file versions for history and rollback.

**Schema:**
```sql
CREATE TABLE file_versions (
    id INTEGER PRIMARY KEY,
    file_id INTEGER NOT NULL,
    version INTEGER NOT NULL,
    hash BLOB NOT NULL,
    size INTEGER NOT NULL,
    modified_by TEXT NOT NULL,
    modified_at INTEGER NOT NULL,
    FOREIGN KEY (file_id) REFERENCES files(id)
);
```

**Version Limit:** 10 versions per file (configurable)

---

### 4. Activity Log

**Purpose:** Audit trail for all file operations.

**Events:**
- File uploaded
- File downloaded
- File deleted
- Member added/removed
- Permissions changed

**Schema:**
```sql
CREATE TABLE activity_log (
    id INTEGER PRIMARY KEY,
    group_id TEXT NOT NULL,
    actor_peer_id TEXT NOT NULL,
    action TEXT NOT NULL,
    target_path TEXT,
    timestamp INTEGER NOT NULL,
    metadata TEXT
);
```

---

## Security Model

### Encryption Layers

1. **File Content:** XChaCha20-Poly1305 with group key
2. **Group Key:** Encrypted per-member with their public key
3. **Metadata:** Encrypted with group key
4. **Capabilities:** Signed with issuer's private key

### Permission Enforcement

**Server-Side:** Cryptographic capabilities verified before action
**Client-Side:** UI enforces permissions for UX

---

## See Also

- [Features](features.md)
- [Implementation](implementation.md)
- [Client Overview](../overview.md)
