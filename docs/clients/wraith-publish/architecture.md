# WRAITH-Publish Architecture

**Document Version:** 1.0.0
**Last Updated:** 2025-11-28
**Client Version:** 1.0.0

---

## Overview

WRAITH-Publish is a censorship-resistant publishing platform for blogs and long-form content, distributed peer-to-peer with no central servers.

**Design Goals:**
- Publish articles without centralized hosting
- Content addressed by cryptographic hash
- Censorship resistance
- Beautiful reading experience
- Built-in monetization (optional)

---

## Architecture Diagram

```
┌──────────────────────────────────────────────────────┐
│            Publishing UI                             │
│  ┌────────────────┐  ┌──────────────────────────┐   │
│  │  Markdown      │  │     Reader UI            │   │
│  │  Editor        │  │   (Next.js)              │   │
│  └────────────────┘  └──────────────────────────┘   │
└──────────────────────────────────────────────────────┘
                         │
┌──────────────────────────────────────────────────────┐
│         Content Processing                           │
│  - Markdown → HTML conversion                        │
│  - XSS sanitization (DOMPurify)                      │
│  - Image optimization                                │
│  - Metadata extraction                               │
└──────────────────────────────────────────────────────┘
                         │
┌──────────────────────────────────────────────────────┐
│         DHT Storage                                  │
│  - Content-addressed articles                        │
│  - Metadata index                                    │
│  - Author identity verification                      │
└──────────────────────────────────────────────────────┘
                         │
┌──────────────────────────────────────────────────────┐
│         WRAITH Protocol Stack                        │
│  (DHT storage + retrieval)                           │
└──────────────────────────────────────────────────────┘
```

---

## Components

### 1. Markdown Editor

**Features:**
- Live preview
- Syntax highlighting
- Image upload
- Draft auto-save
- XSS protection

**Sanitization:**
```typescript
const sanitized = DOMPurify.sanitize(html, {
  ALLOWED_TAGS: ['p', 'h1', 'h2', 'h3', 'h4', 'blockquote',
                 'code', 'pre', 'strong', 'em', 'ul', 'ol',
                 'li', 'a', 'img'],
  ALLOWED_ATTR: ['href', 'src', 'alt', 'title', 'class'],
  ALLOW_DATA_ATTR: false,
});
```

---

### 2. Content Storage

**DHT Storage:**
```
wraith://article/<content-hash>
```

**Article Structure:**
```json
{
  "title": "Article Title",
  "content": "<sanitized HTML>",
  "markdown": "# Original markdown...",
  "author": "peer-id",
  "published_at": 1700000000,
  "tags": ["tech", "tutorial"],
  "images": ["image-hash-1", "image-hash-2"]
}
```

---

### 3. Discovery and Search

**Article Index:**
- Author index (all articles by author)
- Tag index (articles by tag)
- Full-text search (client-side)

**RSS Feed Generation:**
```xml
<?xml version="1.0"?>
<rss version="2.0">
  <channel>
    <title>Author Name</title>
    <link>wraith://author/peer-id</link>
    <item>
      <title>Article Title</title>
      <link>wraith://article/hash</link>
      <pubDate>...</pubDate>
    </item>
  </channel>
</rss>
```

---

## Performance Characteristics

**Publishing:**
- Article publish: <5 seconds
- Image upload: depends on size

**Reading:**
- Article load: <2 seconds
- Full-text search: <200ms for 1000 articles

---

## See Also

- [Features](features.md)
- [Implementation](implementation.md)
- [Client Overview](../overview.md)
