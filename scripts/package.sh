#!/bin/bash
# WRAITH Protocol Packaging Script
#
# This script creates distribution packages (deb, rpm, tar.gz) for WRAITH Protocol.
#
# Prerequisites:
#   - Rust toolchain installed
#   - For deb: dpkg-deb
#   - For rpm: rpmbuild
#
# Usage:
#   ./scripts/package.sh [deb|rpm|tar|all]
#
# Output:
#   - target/package/wraith_<version>_<arch>.deb
#   - target/package/wraith-<version>-1.<arch>.rpm
#   - target/package/wraith-<version>-<platform>.tar.gz

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
OUTPUT_DIR="$PROJECT_ROOT/target/package"

# Extract version from Cargo.toml
VERSION=$(grep '^version = ' "$PROJECT_ROOT/Cargo.toml" | head -1 | sed 's/.*"\(.*\)".*/\1/')
ARCH=$(uname -m)

# Map architecture names
case "$ARCH" in
    x86_64)
        DEB_ARCH="amd64"
        RPM_ARCH="x86_64"
        ;;
    aarch64|arm64)
        DEB_ARCH="arm64"
        RPM_ARCH="aarch64"
        ;;
    *)
        DEB_ARCH="$ARCH"
        RPM_ARCH="$ARCH"
        ;;
esac

echo "=== WRAITH Protocol Packaging ==="
echo "Version: $VERSION"
echo "Architecture: $ARCH (deb: $DEB_ARCH, rpm: $RPM_ARCH)"
echo "Output directory: $OUTPUT_DIR"
echo ""

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Build release binary
build_release() {
    echo "=== Building Release Binary ==="
    cd "$PROJECT_ROOT"
    cargo build --release -p wraith-cli

    BINARY="$PROJECT_ROOT/target/release/wraith"
    if [ ! -f "$BINARY" ]; then
        echo "ERROR: Binary not found at $BINARY"
        exit 1
    fi

    # Strip debug symbols
    if command -v strip &> /dev/null; then
        strip "$BINARY"
        echo "Binary stripped"
    fi

    echo "Build complete: $BINARY"
    echo ""
}

# Create tar.gz package
create_tarball() {
    echo "=== Creating Tarball ==="

    TARBALL_NAME="wraith-${VERSION}-linux-${ARCH}"
    TARBALL_DIR="$OUTPUT_DIR/$TARBALL_NAME"

    rm -rf "$TARBALL_DIR"
    mkdir -p "$TARBALL_DIR"

    # Copy binary
    cp "$PROJECT_ROOT/target/release/wraith" "$TARBALL_DIR/"

    # Copy documentation
    cp "$PROJECT_ROOT/README.md" "$TARBALL_DIR/"
    cp "$PROJECT_ROOT/LICENSE" "$TARBALL_DIR/"
    cp "$PROJECT_ROOT/CHANGELOG.md" "$TARBALL_DIR/"

    # Copy user guide and config reference
    mkdir -p "$TARBALL_DIR/docs"
    cp "$PROJECT_ROOT/docs/USER_GUIDE.md" "$TARBALL_DIR/docs/" 2>/dev/null || true
    cp "$PROJECT_ROOT/docs/CONFIG_REFERENCE.md" "$TARBALL_DIR/docs/" 2>/dev/null || true

    # Create example config
    mkdir -p "$TARBALL_DIR/examples"
    cat > "$TARBALL_DIR/examples/config.toml" << 'EOF'
# WRAITH Protocol Configuration
# Copy to ~/.config/wraith/config.toml

[node]
private_key_file = "~/.config/wraith/keypair.secret"
nickname = "my_node"

[network]
listen_addr = "0.0.0.0:41641"
max_connections = 1000

[obfuscation]
default_level = "medium"

[transfer]
chunk_size = 262144
max_parallel_chunks = 16

[logging]
level = "info"
format = "text"
EOF

    # Create tarball
    cd "$OUTPUT_DIR"
    tar czf "${TARBALL_NAME}.tar.gz" "$TARBALL_NAME"
    rm -rf "$TARBALL_DIR"

    # Generate checksum
    shasum -a 256 "${TARBALL_NAME}.tar.gz" > "${TARBALL_NAME}.tar.gz.sha256"

    echo "Tarball created: $OUTPUT_DIR/${TARBALL_NAME}.tar.gz"
    echo ""
}

# Create Debian package
create_deb() {
    echo "=== Creating Debian Package ==="

    if ! command -v dpkg-deb &> /dev/null; then
        echo "WARNING: dpkg-deb not found. Skipping deb package."
        echo "Install with: sudo apt install dpkg"
        return
    fi

    DEB_NAME="wraith_${VERSION}_${DEB_ARCH}"
    DEB_DIR="$OUTPUT_DIR/$DEB_NAME"

    rm -rf "$DEB_DIR"
    mkdir -p "$DEB_DIR/DEBIAN"
    mkdir -p "$DEB_DIR/usr/bin"
    mkdir -p "$DEB_DIR/usr/share/doc/wraith"
    mkdir -p "$DEB_DIR/usr/share/man/man1"
    mkdir -p "$DEB_DIR/etc/wraith"
    mkdir -p "$DEB_DIR/lib/systemd/system"

    # Copy binary
    cp "$PROJECT_ROOT/target/release/wraith" "$DEB_DIR/usr/bin/"
    chmod 755 "$DEB_DIR/usr/bin/wraith"

    # Copy documentation
    cp "$PROJECT_ROOT/README.md" "$DEB_DIR/usr/share/doc/wraith/"
    cp "$PROJECT_ROOT/LICENSE" "$DEB_DIR/usr/share/doc/wraith/"
    cp "$PROJECT_ROOT/CHANGELOG.md" "$DEB_DIR/usr/share/doc/wraith/"
    gzip -9 -n "$DEB_DIR/usr/share/doc/wraith/CHANGELOG.md"

    # Create example config
    cat > "$DEB_DIR/etc/wraith/config.toml.example" << 'EOF'
# WRAITH Protocol Configuration
# Copy to ~/.config/wraith/config.toml or /etc/wraith/config.toml

[node]
private_key_file = "/var/lib/wraith/keypair.secret"
nickname = "wraith_node"

[network]
listen_addr = "0.0.0.0:41641"
max_connections = 1000

[obfuscation]
default_level = "medium"

[transfer]
chunk_size = 262144
max_parallel_chunks = 16

[logging]
level = "info"
format = "text"
output = "/var/log/wraith/wraith.log"
EOF

    # Create systemd service file
    cat > "$DEB_DIR/lib/systemd/system/wraith.service" << 'EOF'
[Unit]
Description=WRAITH Protocol Daemon
Documentation=https://github.com/doublegate/WRAITH-Protocol
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=wraith
Group=wraith
ExecStart=/usr/bin/wraith daemon
Restart=on-failure
RestartSec=5

# Security hardening
NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=yes
PrivateTmp=yes
PrivateDevices=yes
ProtectKernelTunables=yes
ProtectKernelModules=yes
ProtectControlGroups=yes
ReadWritePaths=/var/lib/wraith /var/log/wraith

# Resource limits
LimitNOFILE=65536
LimitNPROC=4096

[Install]
WantedBy=multi-user.target
EOF

    # Create control file
    INSTALLED_SIZE=$(du -sk "$DEB_DIR" | cut -f1)
    cat > "$DEB_DIR/DEBIAN/control" << EOF
Package: wraith
Version: $VERSION
Section: net
Priority: optional
Architecture: $DEB_ARCH
Installed-Size: $INSTALLED_SIZE
Maintainer: WRAITH Protocol Contributors <wraith@example.com>
Homepage: https://github.com/doublegate/WRAITH-Protocol
Description: Secure decentralized file transfer protocol
 WRAITH (Wire-speed Resilient Authenticated Invisible Transfer Handler)
 is a decentralized secure file transfer protocol optimized for
 high-throughput, low-latency operation with strong security guarantees
 and traffic analysis resistance.
 .
 Features include:
  - End-to-end encryption with XChaCha20-Poly1305
  - Traffic analysis resistance with obfuscation
  - NAT traversal and relay support
  - Multi-peer parallel downloads
  - Resume support for interrupted transfers
EOF

    # Create postinst script
    cat > "$DEB_DIR/DEBIAN/postinst" << 'EOF'
#!/bin/sh
set -e

# Create wraith user and group if they don't exist
if ! getent group wraith > /dev/null; then
    groupadd --system wraith
fi

if ! getent passwd wraith > /dev/null; then
    useradd --system --gid wraith --home-dir /var/lib/wraith \
        --shell /usr/sbin/nologin wraith
fi

# Create directories
mkdir -p /var/lib/wraith
mkdir -p /var/log/wraith
chown wraith:wraith /var/lib/wraith
chown wraith:wraith /var/log/wraith
chmod 750 /var/lib/wraith
chmod 750 /var/log/wraith

# Reload systemd
if [ -d /run/systemd/system ]; then
    systemctl daemon-reload
fi

echo "WRAITH Protocol installed successfully."
echo ""
echo "To start the daemon:"
echo "  sudo systemctl start wraith"
echo "  sudo systemctl enable wraith"
echo ""
echo "Configuration:"
echo "  Copy /etc/wraith/config.toml.example to /etc/wraith/config.toml"
echo "  or ~/.config/wraith/config.toml for user config"

exit 0
EOF
    chmod 755 "$DEB_DIR/DEBIAN/postinst"

    # Create prerm script
    cat > "$DEB_DIR/DEBIAN/prerm" << 'EOF'
#!/bin/sh
set -e

# Stop service if running
if [ -d /run/systemd/system ]; then
    systemctl stop wraith 2>/dev/null || true
    systemctl disable wraith 2>/dev/null || true
fi

exit 0
EOF
    chmod 755 "$DEB_DIR/DEBIAN/prerm"

    # Build deb package
    dpkg-deb --build --root-owner-group "$DEB_DIR"
    mv "$OUTPUT_DIR/${DEB_NAME}.deb" "$OUTPUT_DIR/"
    rm -rf "$DEB_DIR"

    # Generate checksum
    cd "$OUTPUT_DIR"
    shasum -a 256 "${DEB_NAME}.deb" > "${DEB_NAME}.deb.sha256"

    echo "Debian package created: $OUTPUT_DIR/${DEB_NAME}.deb"
    echo ""
}

# Create RPM package
create_rpm() {
    echo "=== Creating RPM Package ==="

    if ! command -v rpmbuild &> /dev/null; then
        echo "WARNING: rpmbuild not found. Skipping rpm package."
        echo "Install with: sudo dnf install rpm-build (Fedora/RHEL)"
        echo "          or: sudo apt install rpm (Debian/Ubuntu)"
        return
    fi

    RPM_BUILD_ROOT="$OUTPUT_DIR/rpmbuild"
    rm -rf "$RPM_BUILD_ROOT"
    mkdir -p "$RPM_BUILD_ROOT"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

    # Create source tarball for rpmbuild
    TARBALL_NAME="wraith-${VERSION}"
    TARBALL_DIR="$RPM_BUILD_ROOT/SOURCES/$TARBALL_NAME"
    mkdir -p "$TARBALL_DIR"

    cp "$PROJECT_ROOT/target/release/wraith" "$TARBALL_DIR/"
    cp "$PROJECT_ROOT/README.md" "$TARBALL_DIR/"
    cp "$PROJECT_ROOT/LICENSE" "$TARBALL_DIR/"
    cp "$PROJECT_ROOT/CHANGELOG.md" "$TARBALL_DIR/"

    cd "$RPM_BUILD_ROOT/SOURCES"
    tar czf "${TARBALL_NAME}.tar.gz" "$TARBALL_NAME"
    rm -rf "$TARBALL_DIR"

    # Create spec file
    cat > "$RPM_BUILD_ROOT/SPECS/wraith.spec" << EOF
Name:           wraith
Version:        $VERSION
Release:        1%{?dist}
Summary:        Secure decentralized file transfer protocol

License:        MIT
URL:            https://github.com/doublegate/WRAITH-Protocol
Source0:        %{name}-%{version}.tar.gz

BuildArch:      $RPM_ARCH

%description
WRAITH (Wire-speed Resilient Authenticated Invisible Transfer Handler)
is a decentralized secure file transfer protocol optimized for
high-throughput, low-latency operation with strong security guarantees
and traffic analysis resistance.

Features include:
- End-to-end encryption with XChaCha20-Poly1305
- Traffic analysis resistance with obfuscation
- NAT traversal and relay support
- Multi-peer parallel downloads
- Resume support for interrupted transfers

%prep
%setup -q

%install
mkdir -p %{buildroot}%{_bindir}
mkdir -p %{buildroot}%{_sysconfdir}/wraith
mkdir -p %{buildroot}%{_unitdir}
mkdir -p %{buildroot}%{_docdir}/%{name}

install -m 755 wraith %{buildroot}%{_bindir}/wraith
install -m 644 README.md %{buildroot}%{_docdir}/%{name}/
install -m 644 LICENSE %{buildroot}%{_docdir}/%{name}/
install -m 644 CHANGELOG.md %{buildroot}%{_docdir}/%{name}/

# Create example config
cat > %{buildroot}%{_sysconfdir}/wraith/config.toml.example << 'CONFIGEOF'
# WRAITH Protocol Configuration
[node]
private_key_file = "/var/lib/wraith/keypair.secret"
nickname = "wraith_node"

[network]
listen_addr = "0.0.0.0:41641"
max_connections = 1000

[obfuscation]
default_level = "medium"

[transfer]
chunk_size = 262144
max_parallel_chunks = 16

[logging]
level = "info"
format = "text"
CONFIGEOF

# Create systemd service
cat > %{buildroot}%{_unitdir}/wraith.service << 'SERVICEEOF'
[Unit]
Description=WRAITH Protocol Daemon
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=wraith
Group=wraith
ExecStart=/usr/bin/wraith daemon
Restart=on-failure
RestartSec=5
NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=yes
PrivateTmp=yes
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
SERVICEEOF

%pre
getent group wraith >/dev/null || groupadd -r wraith
getent passwd wraith >/dev/null || useradd -r -g wraith -d /var/lib/wraith -s /sbin/nologin wraith
mkdir -p /var/lib/wraith /var/log/wraith
chown wraith:wraith /var/lib/wraith /var/log/wraith

%post
%systemd_post wraith.service

%preun
%systemd_preun wraith.service

%postun
%systemd_postun_with_restart wraith.service

%files
%license LICENSE
%doc README.md CHANGELOG.md
%{_bindir}/wraith
%config(noreplace) %{_sysconfdir}/wraith/config.toml.example
%{_unitdir}/wraith.service

%changelog
* $(date +"%a %b %d %Y") WRAITH Protocol Contributors <wraith@example.com> - $VERSION-1
- Release version $VERSION
EOF

    # Build RPM
    rpmbuild --define "_topdir $RPM_BUILD_ROOT" -bb "$RPM_BUILD_ROOT/SPECS/wraith.spec"

    # Move RPM to output directory
    find "$RPM_BUILD_ROOT/RPMS" -name "*.rpm" -exec mv {} "$OUTPUT_DIR/" \;
    rm -rf "$RPM_BUILD_ROOT"

    # Generate checksum
    cd "$OUTPUT_DIR"
    for rpm in wraith-*.rpm; do
        shasum -a 256 "$rpm" > "${rpm}.sha256"
    done

    echo "RPM package created in: $OUTPUT_DIR/"
    echo ""
}

# Print summary
print_summary() {
    echo "=== Packaging Summary ==="
    echo ""
    echo "Output directory: $OUTPUT_DIR"
    echo ""
    echo "Generated packages:"
    ls -lh "$OUTPUT_DIR"/*.{deb,rpm,tar.gz} 2>/dev/null || echo "(none)"
    echo ""
    echo "Checksums:"
    ls -lh "$OUTPUT_DIR"/*.sha256 2>/dev/null || echo "(none)"
    echo ""
    echo "Installation instructions:"
    echo ""
    echo "  Debian/Ubuntu:"
    echo "    sudo dpkg -i wraith_${VERSION}_${DEB_ARCH}.deb"
    echo ""
    echo "  Fedora/RHEL/CentOS:"
    echo "    sudo rpm -i wraith-${VERSION}-1.${RPM_ARCH}.rpm"
    echo ""
    echo "  Generic Linux:"
    echo "    tar xzf wraith-${VERSION}-linux-${ARCH}.tar.gz"
    echo "    sudo cp wraith-${VERSION}-linux-${ARCH}/wraith /usr/local/bin/"
    echo ""
}

# Main
case "${1:-all}" in
    tar)
        build_release
        create_tarball
        ;;
    deb)
        build_release
        create_deb
        ;;
    rpm)
        build_release
        create_rpm
        ;;
    all)
        build_release
        create_tarball
        create_deb
        create_rpm
        print_summary
        ;;
    *)
        echo "Usage: $0 [deb|rpm|tar|all]"
        exit 1
        ;;
esac

echo "Packaging complete!"
