# WRAITH-Share Features

**Document Version:** 1.0.0
**Last Updated:** 2025-11-28
**Client Version:** 1.0.0

---

## Overview

WRAITH-Share enables secure group file sharing with cryptographic access control, version history, and activity logging—all without central servers.

---

## Core Features

### 1. Group Creation and Management

**User Stories:**
- As a user, I can create shared folders with custom names
- As an admin, I can add/remove members
- As an admin, I can assign roles (admin/write/read)
- As a user, I can leave groups I'm a member of

**Group Roles:**
- **Admin:** Full control (add/remove members, change permissions, delete group)
- **Write:** Upload, modify, delete own files
- **Read:** Download files only

---

### 2. Member Invitation

**User Stories:**
- As an admin, I can invite members via QR code
- As an admin, I can generate invitation links with expiration
- As a user, I can accept invitations and join groups

**Invitation Methods:**
1. **QR Code:** Scan to join instantly
2. **Link:** Click to join (with optional password)
3. **Direct:** Send invite to known peer ID

**Invitation Expiration:** 7 days (default, configurable)

---

### 3. Granular Permissions

**Permission Types:**
- **Read:** Download files
- **Write:** Upload/modify files
- **Delete:** Delete files
- **Invite:** Add new members
- **Admin:** All permissions + group management

**Per-File Permissions:** Optional fine-grained control on specific files/folders

---

### 4. File Versioning

**User Stories:**
- As a user, I can view file version history
- As a user, I can restore previous versions
- As a user, I see who modified each version

**Version Retention:**
- Keep 10 versions per file (default)
- Versions older than 90 days automatically deleted
- Admin can configure retention policy

**Version Viewer:**
```
document.docx - Version History
┌─────────────────────────────────────────┐
│  v10  Today 3:45 PM   Alice    1.5 MB   │
│  v9   Today 2:30 PM   Bob      1.4 MB   │
│  v8   Yesterday       Alice    1.3 MB   │
│  ...                                    │
│  [Restore] [Download] [Compare]         │
└─────────────────────────────────────────┘
```

---

### 5. Activity Log

**User Stories:**
- As an admin, I can view all file operations
- As a user, I can see who accessed my files
- As an admin, I can export activity logs

**Logged Events:**
- File uploads/downloads/deletions
- Member additions/removals
- Permission changes
- Group settings changes

**Log Retention:** 90 days (configurable)

---

### 6. Link Sharing

**User Stories:**
- As a user, I can generate shareable links for files
- As a user, I can set link expiration dates
- As a user, I can password-protect links

**Link Options:**
- Expiration: 1 hour, 1 day, 1 week, 1 month, never
- Password protection (optional)
- Download limit (e.g., 10 downloads max)

---

## Advanced Features

### Search and Filtering

**Search:**
- Full-text filename search
- Filter by file type
- Filter by uploader
- Filter by date range

### Bulk Operations

**User Stories:**
- As a user, I can select multiple files for download
- As a user, I can upload entire folders
- As an admin, I can delete multiple files at once

### Offline Support (PWA)

**Features:**
- View cached files offline
- Queue uploads for when online
- Background sync when connection restored

---

## User Interface

### Group Dashboard

```
┌─────────────────────────────────────────┐
│  Team Documents                         │
│  👥 15 members  📁 127 files  💾 5.2 GB│
├─────────────────────────────────────────┤
│  Recent Activity                        │
│  • Alice uploaded report.pdf (2m ago)   │
│  • Bob modified budget.xlsx (15m ago)   │
│  • Charlie joined the group (1h ago)    │
│                                         │
│  Files                                  │
│  ┌───────────────────────────────────┐  │
│  │ 📄 report.pdf      2.5 MB  Today  │  │
│  │ 📊 budget.xlsx     1.2 MB  Today  │  │
│  │ 📁 Archive/       52 files        │  │
│  └───────────────────────────────────┘  │
│                                         │
│  [Upload] [New Folder] [Invite Member] │
└─────────────────────────────────────────┘
```

---

## See Also

- [Architecture](architecture.md)
- [Implementation](implementation.md)
- [Client Overview](../overview.md)
