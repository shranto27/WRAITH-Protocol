# WRAITH-Publish Features

**Document Version:** 1.0.0
**Last Updated:** 2025-11-28
**Client Version:** 1.0.0

---

## Overview

WRAITH-Publish provides a censorship-resistant platform for publishing articles, blogs, and long-form content with no central authority.

---

## Core Features

### 1. Markdown Editor

**User Stories:**
- As a writer, I can write articles in Markdown
- As a writer, I see live preview while writing
- As a writer, drafts auto-save every 30 seconds

**Features:**
- Syntax highlighting
- Live preview
- Auto-save drafts
- Image drag-and-drop
- Code block support

---

### 2. Content Publishing

**User Stories:**
- As a writer, I can publish articles to DHT
- As a writer, I get a permanent link to my article
- As a writer, I can update published articles (creates new version)

**Publication Process:**
1. Write article in Markdown
2. Preview and edit
3. Add metadata (title, tags)
4. Publish to DHT
5. Receive `wraith://article/<hash>` link

---

### 3. Reader Experience

**User Stories:**
- As a reader, I can read articles offline after first load
- As a reader, I can save articles for later
- As a reader, I can search across all articles

**Reading Features:**
- Clean typography
- Dark/light mode
- Adjustable font size
- Bookmark articles
- Reading progress indicator

---

### 4. Discovery

**User Stories:**
- As a reader, I can browse articles by tag
- As a reader, I can follow authors
- As a reader, I can search articles by keyword

**Discovery Methods:**
- Tag browsing
- Author pages
- Full-text search
- RSS feed subscription

---

## Advanced Features

### Custom Domains

**User Stories:**
- As a writer, I can map my domain to my articles
- Readers can access via `https://mysite.com` → `wraith://`

**DNS TXT Record:**
```
_wraith TXT "v=1;peer=<peer-id>"
```

### Comments

**User Stories:**
- As a reader, I can comment on articles
- As a writer, I can moderate comments
- Comments are also decentralized

---

## See Also

- [Architecture](architecture.md)
- [Implementation](implementation.md)
- [Client Overview](../overview.md)
