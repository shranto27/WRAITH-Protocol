# WRAITH-Stream Implementation

**Document Version:** 1.0.0
**Last Updated:** 2025-11-28
**Client Version:** 1.0.0

---

## Overview

Implementation details for WRAITH-Stream, including FFmpeg transcoding, HLS segment encryption, and adaptive bitrate logic.

---

## Technology Stack

```typescript
// Frontend
"dependencies": {
  "video.js": "^8.0.0",
  "hls.js": "^1.5.0"
}
```

```toml
# Backend
[dependencies]
wraith-core = { path = "../../crates/wraith-core" }
tokio = { version = "1.40", features = ["full"] }
```

---

## FFmpeg Transcoding

```typescript
// src/transcoder/VideoTranscoder.ts
import ffmpeg from 'fluent-ffmpeg';

export class VideoTranscoder {
  async transcode(inputPath: string, outputDir: string): Promise<string[]> {
    const profiles = [
      { name: '240p', width: 426, height: 240, bitrate: '400k' },
      { name: '480p', width: 854, height: 480, bitrate: '1000k' },
      { name: '720p', width: 1280, height: 720, bitrate: '2500k' },
      { name: '1080p', width: 1920, height: 1080, bitrate: '5000k' },
    ];

    const masterPlaylist = `${outputDir}/master.m3u8`;
    let masterContent = '#EXTM3U\n#EXT-X-VERSION:3\n';

    for (const profile of profiles) {
      const outputPath = `${outputDir}/${profile.name}.m3u8`;

      await new Promise<void>((resolve, reject) => {
        ffmpeg(inputPath)
          .outputOptions([
            '-c:v libx264',
            '-c:a aac',
            `-b:v ${profile.bitrate}`,
            `-s ${profile.width}x${profile.height}`,
            '-hls_time 6',
            '-hls_playlist_type vod',
            `-hls_segment_filename ${outputDir}/${profile.name}_%03d.ts`,
            '-f hls',
          ])
          .output(outputPath)
          .on('end', () => resolve())
          .on('error', reject)
          .run();
      });

      masterContent += `#EXT-X-STREAM-INF:BANDWIDTH=${parseInt(profile.bitrate) * 1000}\n`;
      masterContent += `${profile.name}.m3u8\n`;
    }

    await fs.promises.writeFile(masterPlaylist, masterContent);
    return [masterPlaylist];
  }
}
```

---

## Segment Encryption

```rust
// src/stream/encryption.rs
use chacha20poly1305::XChaCha20Poly1305;

pub fn encrypt_segment(
    segment: &[u8],
    stream_key: &[u8; 32],
    segment_id: &str,
) -> Vec<u8> {
    let nonce = blake3::derive_key(
        "segment-nonce",
        &[stream_key, segment_id.as_bytes()].concat()
    );

    let cipher = XChaCha20Poly1305::new(stream_key.into());
    cipher.encrypt(&nonce[..24].try_into().unwrap(), segment)
        .expect("encryption failed")
}
```

---

## Build and Deployment

```bash
npm run build
npm run tauri build
```

---

## See Also

- [Architecture](architecture.md)
- [Features](features.md)
- [Client Overview](../overview.md)
