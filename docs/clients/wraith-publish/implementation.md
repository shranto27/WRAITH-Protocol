# WRAITH-Publish Implementation

**Document Version:** 1.0.0
**Last Updated:** 2025-11-28
**Client Version:** 1.0.0

---

## Overview

Implementation details for WRAITH-Publish, including Markdown processing, content storage, and XSS protection.

---

## Technology Stack

```json
{
  "dependencies": {
    "next": "^14.0.0",
    "react": "^18.3.0",
    "unified": "^11.0.0",
    "remark-parse": "^11.0.0",
    "rehype-sanitize": "^6.0.0",
    "dompurify": "^3.0.0",
    "@tauri-apps/api": "^2.0.0"
  }
}
```

---

## Markdown Processing

```typescript
// src/editor/MarkdownProcessor.ts
import { unified } from 'unified';
import remarkParse from 'remark-parse';
import remarkRehype from 'remark-rehype';
import rehypeSanitize from 'rehype-sanitize';
import rehypeStringify from 'rehype-stringify';
import DOMPurify from 'dompurify';

export async function processMarkdown(markdown: string): Promise<string> {
  const result = await unified()
    .use(remarkParse)
    .use(remarkRehype)
    .use(rehypeSanitize)
    .use(rehypeStringify)
    .process(markdown);

  // Additional client-side sanitization
  const sanitized = DOMPurify.sanitize(String(result), {
    ALLOWED_TAGS: ['p', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
                   'blockquote', 'code', 'pre', 'strong', 'em',
                   'ul', 'ol', 'li', 'a', 'img'],
    ALLOWED_ATTR: ['href', 'src', 'alt', 'title', 'class'],
    ALLOW_DATA_ATTR: false,
  });

  return sanitized;
}
```

---

## Content Publishing

```rust
// src-tauri/src/publishing.rs
use wraith_core::DhtNode;
use blake3;

pub struct Publisher {
    dht: DhtNode,
}

impl Publisher {
    pub async fn publish_article(&self, article: Article) -> Result<String> {
        // Serialize article
        let content = serde_json::to_vec(&article)?;

        // Calculate content hash
        let hash = blake3::hash(&content);
        let article_id = hash.to_hex().to_string();

        // Store in DHT
        self.dht.store(
            format!("article:{}", article_id),
            content,
        ).await?;

        // Update author index
        self.update_author_index(&article.author, &article_id).await?;

        Ok(article_id)
    }

    async fn update_author_index(
        &self,
        author: &str,
        article_id: &str,
    ) -> Result<()> {
        let key = format!("author-index:{}", author);

        let mut index = self.dht.retrieve(&key)
            .await
            .unwrap_or_default();

        index.push(article_id.to_string());

        self.dht.store(key, serde_json::to_vec(&index)?).await?;

        Ok(())
    }
}
```

---

## Build and Deployment

```bash
# Development
npm run dev

# Production build
npm run build
npm run tauri build
```

---

## See Also

- [Architecture](architecture.md)
- [Features](features.md)
- [Client Overview](../overview.md)
