# WRAITH Protocol Platform Support

**Document Version:** 1.0.0
**Last Updated:** 2025-11-28
**Status:** Integration Documentation

---

## Overview

WRAITH Protocol is designed for cross-platform compatibility while leveraging platform-specific optimizations where available. This document details supported platforms, requirements, and platform-specific features.

**Tier Classification:**
- **Tier 1:** Fully supported, tested in CI, production-ready
- **Tier 2:** Supported, tested manually, stable
- **Tier 3:** Experimental, community-maintained

---

## Platform Support Matrix

| Platform | Tier | Min Version | AF_XDP | io_uring | Notes |
|----------|------|-------------|--------|----------|-------|
| **Linux x86_64** | 1 | 6.2+ | ✓ | ✓ | Full feature set |
| **Linux aarch64** | 1 | 6.2+ | ✓ | ✓ | ARM64 servers, Raspberry Pi 4+ |
| **macOS x86_64** | 2 | 12.0+ | ✗ | ✗ | Standard UDP only |
| **macOS aarch64** | 2 | 12.0+ | ✗ | ✗ | Apple Silicon (M1/M2/M3) |
| **Windows x86_64** | 2 | 10+ | ✗ | ✗ | Standard UDP only |
| **FreeBSD x86_64** | 3 | 13.0+ | ✗ | ✗ | Community support |
| **OpenBSD x86_64** | 3 | 7.0+ | ✗ | ✗ | Community support |

---

## Linux (Tier 1)

### Requirements

**Minimum Kernel Version:**
- 6.2+ for full feature support
- 5.15+ for basic functionality (no AF_XDP zero-copy)

**Kernel Configuration:**
```bash
# Required features
CONFIG_XDP_SOCKETS=y
CONFIG_BPF=y
CONFIG_BPF_SYSCALL=y
CONFIG_IO_URING=y

# Optional but recommended
CONFIG_NET_CLS_BPF=y
CONFIG_NET_ACT_BPF=y
```

**Verify kernel support:**
```bash
# Check kernel version
uname -r

# Check XDP support
zgrep CONFIG_XDP_SOCKETS /proc/config.gz

# Check io_uring support
zgrep CONFIG_IO_URING /proc/config.gz
```

### Distribution-Specific Instructions

**Ubuntu 22.04 LTS / 24.04 LTS:**
```bash
# Update kernel (if needed)
sudo apt update
sudo apt install linux-generic-hwe-22.04

# Install dependencies
sudo apt install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libbpf-dev \
    libelf-dev \
    clang \
    llvm

# Reboot if kernel was updated
sudo reboot
```

**Fedora 38+:**
```bash
# Install dependencies
sudo dnf install -y \
    gcc \
    pkg-config \
    openssl-devel \
    libbpf-devel \
    elfutils-libelf-devel \
    clang \
    llvm

# Kernel already supports XDP/io_uring
```

**Arch Linux:**
```bash
# Install dependencies
sudo pacman -S --needed \
    base-devel \
    openssl \
    libbpf \
    libelf \
    clang \
    llvm

# Kernel already supports XDP/io_uring
```

### Capabilities

**Grant capabilities for AF_XDP (recommended):**
```bash
# Allow binary to use XDP without sudo
sudo setcap cap_net_raw,cap_net_admin,cap_bpf+ep /path/to/wraith-cli

# Verify capabilities
getcap /path/to/wraith-cli
```

**Alternative (run as root, not recommended):**
```bash
sudo ./wraith-cli transfer file.bin
```

### Performance Tuning

**Socket buffer sizes:**
```bash
# Increase UDP buffer sizes (temporary)
sudo sysctl -w net.core.rmem_max=26214400
sudo sysctl -w net.core.wmem_max=26214400

# Make permanent
echo "net.core.rmem_max=26214400" | sudo tee -a /etc/sysctl.conf
echo "net.core.wmem_max=26214400" | sudo tee -a /etc/sysctl.conf
sudo sysctl -p
```

**Network interface tuning:**
```bash
# Increase ring buffer size
sudo ethtool -G eth0 rx 4096 tx 4096

# Enable hardware offloads
sudo ethtool -K eth0 gso on gro on tso on
```

---

## macOS (Tier 2)

### Requirements

**Minimum Version:** macOS 12.0 (Monterey)

**Xcode Command Line Tools:**
```bash
xcode-select --install
```

**Homebrew Dependencies:**
```bash
brew install openssl@3 pkg-config
```

### Build Configuration

**Set environment variables:**
```bash
# Point to Homebrew OpenSSL
export OPENSSL_DIR=$(brew --prefix openssl@3)
export PKG_CONFIG_PATH=$(brew --prefix openssl@3)/lib/pkgconfig

# Build WRAITH
cargo build --release --no-default-features
```

### Limitations

- **No AF_XDP:** macOS doesn't support AF_XDP (Linux-specific)
- **No io_uring:** macOS uses kqueue instead
- **Performance:** 20-30% lower throughput than Linux (standard UDP only)

### Performance Optimization

**macOS-specific tuning:**
```bash
# Increase socket buffer sizes
sudo sysctl -w kern.ipc.maxsockbuf=8388608
sudo sysctl -w net.inet.udp.recvspace=2097152
sudo sysctl -w net.inet.udp.maxdgram=65535
```

---

## Windows (Tier 2)

### Requirements

**Minimum Version:** Windows 10 version 1903 or newer

**Build Tools:**
- Visual Studio 2022 or newer with C++ build tools
- Or: Build Tools for Visual Studio 2022

**Install Rust:**
```powershell
# Install rustup
winget install Rustlang.Rustup

# Verify installation
rustc --version
```

### Build Configuration

**Build WRAITH:**
```powershell
# Standard build (no kernel bypass features)
cargo build --release --no-default-features
```

### Limitations

- **No AF_XDP:** Windows doesn't support AF_XDP
- **No io_uring:** Windows uses IOCP instead
- **Performance:** 15-25% lower throughput than Linux
- **Firewall:** May require firewall exceptions

### Windows-Specific Configuration

**Firewall rule:**
```powershell
# Allow WRAITH through firewall (run as Administrator)
New-NetFirewallRule `
    -DisplayName "WRAITH Protocol" `
    -Direction Inbound `
    -Protocol UDP `
    -LocalPort 41641 `
    -Action Allow
```

**Performance tuning:**
```powershell
# Increase UDP buffer sizes (run as Administrator)
netsh int ipv4 set glob defaultcurhoplimit=64
netsh int ipv4 set glob taskoffload=enabled
```

---

## Cross-Compilation

### Linux to ARM64

**Install cross-compilation toolchain:**
```bash
# Install cross-compilation tools
rustup target add aarch64-unknown-linux-gnu
sudo apt install gcc-aarch64-linux-gnu

# Configure Cargo
mkdir -p ~/.cargo
cat >> ~/.cargo/config.toml <<'EOF'
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
EOF

# Build for ARM64
cargo build --release --target aarch64-unknown-linux-gnu
```

### Static Linking (musl)

**Build static binary:**
```bash
# Install musl target
rustup target add x86_64-unknown-linux-musl

# Install musl toolchain
sudo apt install musl-tools

# Build static binary
cargo build --release --target x86_64-unknown-linux-musl

# Verify static linking
ldd target/x86_64-unknown-linux-musl/release/wraith-cli
# Output: "not a dynamic executable"
```

---

## Mobile Platforms

### Android

**Requirements:**
- NDK r25 or newer
- Android 8.0 (API 26) or newer

**Build Configuration:**
```bash
# Install Android targets
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add x86_64-linux-android

# Set NDK path
export ANDROID_NDK_HOME=$HOME/Android/Sdk/ndk/25.2.9519653

# Build for Android
cargo ndk -t arm64-v8a -o ./jniLibs build --release
```

**Limitations:**
- No AF_XDP support
- Network permissions required in AndroidManifest.xml
- Cellular networks may have restrictive NAT

### iOS

**Requirements:**
- Xcode 14 or newer
- iOS 14.0 or newer

**Build Configuration:**
```bash
# Install iOS targets
rustup target add aarch64-apple-ios
rustup target add x86_64-apple-ios

# Build for iOS (requires macOS)
cargo build --release --target aarch64-apple-ios
```

**Limitations:**
- No background transfers (iOS restrictions)
- Network permissions required in Info.plist
- App Store review considerations for P2P functionality

---

## Embedded Systems

### Raspberry Pi

**Supported Models:**
- Raspberry Pi 4 (Tier 2)
- Raspberry Pi 5 (Tier 1, kernel 6.6+)

**OS:** Raspberry Pi OS (64-bit) or Ubuntu Server 22.04

**Installation:**
```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install dependencies
sudo apt install -y build-essential pkg-config libssl-dev

# Build WRAITH
cargo build --release --no-default-features
```

**Performance Notes:**
- Pi 4: ~500 Mbps UDP throughput
- Pi 5: ~1 Gbps UDP throughput (Gigabit Ethernet)
- No AF_XDP support (driver limitation)

---

## Container Platforms

### Docker

**Dockerfile:**
```dockerfile
FROM rust:1.75 AS builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libbpf-dev \
    libelf-dev

# Copy source
WORKDIR /wraith
COPY . .

# Build release binary
RUN cargo build --release

# Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy binary
COPY --from=builder /wraith/target/release/wraith-cli /usr/local/bin/

# Expose default port
EXPOSE 41641/udp

ENTRYPOINT ["wraith-cli"]
```

**Build and run:**
```bash
# Build image
docker build -t wraith:latest .

# Run container
docker run -p 41641:41641/udp wraith:latest
```

### Kubernetes

**Deployment example:**
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: wraith
spec:
  replicas: 3
  selector:
    matchLabels:
      app: wraith
  template:
    metadata:
      labels:
        app: wraith
    spec:
      containers:
      - name: wraith
        image: wraith:latest
        ports:
        - containerPort: 41641
          protocol: UDP
        securityContext:
          capabilities:
            add:
              - NET_RAW
              - NET_ADMIN
              - BPF
---
apiVersion: v1
kind: Service
metadata:
  name: wraith-service
spec:
  type: LoadBalancer
  ports:
  - port: 41641
    protocol: UDP
    targetPort: 41641
  selector:
    app: wraith
```

---

## Platform-Specific Features

### Linux-Only Features

**AF_XDP (Zero-Copy Networking):**
```rust
#[cfg(target_os = "linux")]
use wraith_transport::XdpTransport;

#[cfg(target_os = "linux")]
async fn use_xdp() -> Result<()> {
    let transport = XdpTransport::new("eth0", 0)?;
    // ... use zero-copy transport
    Ok(())
}

#[cfg(not(target_os = "linux"))]
async fn use_xdp() -> Result<()> {
    Err("AF_XDP only supported on Linux".into())
}
```

**io_uring (High-Performance I/O):**
```rust
#[cfg(target_os = "linux")]
use wraith_files::IoUringFileReader;

#[cfg(target_os = "linux")]
async fn read_file_fast(path: &Path) -> Result<Vec<u8>> {
    let reader = IoUringFileReader::open(path)?;
    reader.read_all().await
}

#[cfg(not(target_os = "linux"))]
async fn read_file_fast(path: &Path) -> Result<Vec<u8>> {
    tokio::fs::read(path).await.map_err(Into::into)
}
```

---

## Testing Across Platforms

### CI Configuration

**GitHub Actions example:**
```yaml
name: Cross-Platform Tests

on: [push, pull_request]

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        include:
          - os: ubuntu-latest
            features: af-xdp,io-uring
          - os: macos-latest
            features: default
          - os: windows-latest
            features: default

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true

      - name: Build
        run: cargo build --features ${{ matrix.features }}

      - name: Test
        run: cargo test --features ${{ matrix.features }}
```

---

## See Also

- [Embedding Guide](embedding-guide.md)
- [Interoperability](interoperability.md)
- [Development Guide](../engineering/development-guide.md)
