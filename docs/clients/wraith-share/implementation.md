# WRAITH-Share Implementation

**Document Version:** 1.0.0
**Last Updated:** 2025-11-28
**Client Version:** 1.0.0

---

## Overview

Implementation details for WRAITH-Share, including capability-based access control, file versioning, and activity logging.

---

## Technology Stack

```json
{
  "dependencies": {
    "react": "^18.3.0",
    "@tanstack/react-query": "^5.0.0",
    "@tauri-apps/api": "^2.0.0",
    "indexed-db": "^1.0.0"
  }
}
```

```toml
[dependencies]
wraith-core = { path = "../../crates/wraith-core" }
wraith-files = { path = "../../crates/wraith-files" }
rusqlite = { version = "0.31", features = ["bundled"] }
tokio = { version = "1.40", features = ["full"] }
```

---

## Capability System Implementation

```rust
// src/access/capability.rs
use ed25519_dalek::{Keypair, Signature, Signer, Verifier};

pub struct CapabilityManager {
    keypair: Keypair,
    db: Arc<Database>,
}

impl CapabilityManager {
    pub fn issue_capability(
        &self,
        group_id: GroupId,
        recipient: PeerId,
        permissions: Permissions,
        expires_in: Duration,
    ) -> Result<Capability> {
        let cap = Capability {
            id: CapabilityId::new(),
            group_id,
            permissions,
            issued_by: self.local_peer_id(),
            issued_to: recipient,
            expires_at: Some(SystemTime::now() + expires_in),
            signature: Signature::default(),
        };

        // Sign capability
        let message = cap.to_bytes();
        let signature = self.keypair.sign(&message);

        Ok(Capability {
            signature,
            ..cap
        })
    }

    pub fn verify_capability(&self, cap: &Capability, action: Action) -> Result<()> {
        // Check expiration
        if let Some(expires) = cap.expires_at {
            if SystemTime::now() > expires {
                return Err(Error::CapabilityExpired);
            }
        }

        // Check permission
        if !cap.has_permission(action) {
            return Err(Error::PermissionDenied);
        }

        // Verify signature
        let message = cap.to_bytes_without_signature();
        let public_key = self.get_public_key(cap.issued_by)?;

        public_key.verify(&message, &cap.signature)?;

        Ok(())
    }
}
```

---

## File Version Management

```typescript
// src/files/VersionManager.ts
export class VersionManager {
  private db: Database;
  private maxVersions = 10;

  async addVersion(fileId: string, data: Uint8Array): Promise<Version> {
    const hash = await this.computeHash(data);

    const version: Version = {
      id: generateId(),
      fileId,
      version: await this.getNextVersionNumber(fileId),
      hash,
      size: data.length,
      modifiedBy: this.localPeerId,
      modifiedAt: Date.now(),
    };

    await this.db.insertVersion(version);
    await this.store(hash, data);

    // Clean up old versions
    await this.pruneOldVersions(fileId);

    return version;
  }

  async restoreVersion(fileId: string, versionNumber: number): Promise<void> {
    const version = await this.db.getVersion(fileId, versionNumber);
    const data = await this.load(version.hash);

    // Create new version from restored data
    await this.addVersion(fileId, data);
  }

  private async pruneOldVersions(fileId: string): Promise<void> {
    const versions = await this.db.getVersions(fileId);

    if (versions.length > this.maxVersions) {
      const toDelete = versions.slice(this.maxVersions);

      for (const version of toDelete) {
        await this.db.deleteVersion(version.id);
        await this.deleteData(version.hash);
      }
    }
  }
}
```

---

## Activity Logging

```rust
// src/audit/activity_log.rs
pub struct ActivityLogger {
    db: Arc<Database>,
}

impl ActivityLogger {
    pub async fn log_event(&self, event: ActivityEvent) -> Result<()> {
        self.db.insert_activity(LogEntry {
            id: LogId::new(),
            group_id: event.group_id,
            actor: event.actor,
            action: event.action,
            target: event.target,
            timestamp: SystemTime::now(),
            metadata: event.metadata,
        }).await?;

        Ok(())
    }

    pub async fn get_recent_activity(
        &self,
        group_id: GroupId,
        limit: usize,
    ) -> Result<Vec<LogEntry>> {
        self.db.query_activity(group_id, limit).await
    }
}

pub struct ActivityEvent {
    pub group_id: GroupId,
    pub actor: PeerId,
    pub action: ActivityAction,
    pub target: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

pub enum ActivityAction {
    FileUploaded,
    FileDownloaded,
    FileDeleted,
    MemberAdded,
    MemberRemoved,
    PermissionsChanged,
}
```

---

## Build and Deployment

```bash
# Build desktop app
npm run tauri build

# Build PWA
npm run build

# Deploy PWA
npx serve -s dist
```

---

## See Also

- [Architecture](architecture.md)
- [Features](features.md)
- [Client Overview](../overview.md)
