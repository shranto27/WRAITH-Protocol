# WRAITH Protocol Release Guide

This guide explains how to create official releases for the WRAITH Protocol project.

## Overview

The WRAITH Protocol uses an automated GitHub Actions workflow to build cross-platform release binaries, run comprehensive tests, and publish releases to GitHub. The release workflow supports both automated tag-based releases and manual releases.

## Release Workflow Features

### Platforms Supported
- **Linux x86_64 (glibc)** - Standard Linux binary
- **Linux x86_64 (musl)** - Static Linux binary (no runtime dependencies)
- **Linux aarch64** - ARM64 Linux systems
- **macOS x86_64** - Intel-based Macs
- **macOS aarch64** - Apple Silicon Macs (M1/M2/M3)
- **Windows x86_64** - 64-bit Windows systems

### Workflow Stages
1. **Validation** - Verify semantic version format and CHANGELOG entry
2. **Testing** - Run full test suite, clippy, and formatting checks
3. **Building** - Build release binaries for all platforms with cross-compilation
4. **Checksums** - Generate SHA256 checksums for all artifacts
5. **Release** - Create GitHub Release with all artifacts and release notes

### Security Features
- All GitHub Actions inputs are properly sanitized using environment variables
- No command injection vulnerabilities (follows GitHub security best practices)
- Checksums provided for artifact verification
- Binaries are stripped to remove debug symbols

## Creating a Release

### Method 1: Tag-Based Release (Recommended)

This is the standard way to create releases. When you push a semantic version tag, the workflow automatically triggers.

#### Prerequisites
1. **Update CHANGELOG.md**
   ```bash
   # Add your changes under the version header
   nano CHANGELOG.md
   ```

   Ensure your version has a section like:
   ```markdown
   ## [0.1.0] - 2025-11-29

   ### Added
   - Initial release with core protocol implementation
   - Cross-platform support for Linux, macOS, Windows

   ### Security
   - XChaCha20-Poly1305 AEAD encryption
   - X25519 key exchange with Elligator2
   ```

2. **Update version in Cargo.toml**
   ```toml
   [workspace.package]
   version = "0.1.0"
   ```

3. **Commit changes**
   ```bash
   git add CHANGELOG.md Cargo.toml
   git commit -m "chore: prepare release v0.1.0"
   ```

#### Create and Push Tag
```bash
# Create a semantic version tag
git tag -a v0.1.0 -m "WRAITH Protocol v0.1.0"

# Push the tag to GitHub (this triggers the release workflow)
git push origin v0.1.0
```

#### Tag Naming Convention
- **Stable release:** `v1.0.0`, `v1.2.3`
- **Pre-release:** `v0.1.0-alpha.1`, `v1.0.0-beta.2`, `v2.0.0-rc.1`

Tags matching `v*.*.*` (with optional pre-release suffix) will trigger the workflow.

### Method 2: Manual Release

Use this method for testing the release workflow or creating special releases.

1. **Navigate to Actions tab** on GitHub
2. **Select "Release" workflow** from the left sidebar
3. **Click "Run workflow"** button
4. **Fill in the parameters:**
   - **Branch:** main (or your release branch)
   - **Release version:** e.g., `v0.1.0`
   - **Mark as pre-release:** Check if this is alpha/beta/rc
   - **Create as draft:** Check to review before publishing
5. **Click "Run workflow"**

### Pre-release vs Stable Release

The workflow automatically detects pre-releases based on:
- Version contains `alpha`, `beta`, or `rc` (e.g., `v1.0.0-beta.1`)
- Manual workflow with "Mark as pre-release" checked

Pre-releases are marked differently on GitHub and don't trigger "latest release" notifications.

## Monitoring the Release

### Check Workflow Progress

1. Navigate to **Actions** tab on GitHub
2. Click on the **Release** workflow run
3. Monitor the four jobs:
   - **Validate Release** - Checks version format and CHANGELOG
   - **Test Suite** - Runs tests, clippy, formatting
   - **Build (6 jobs)** - Builds binaries for all platforms
   - **Create GitHub Release** - Publishes the release

### Build Matrix

The build job runs in parallel for all platforms:
```
✓ Build x86_64-unknown-linux-gnu
✓ Build x86_64-unknown-linux-musl
✓ Build aarch64-unknown-linux-gnu
✓ Build x86_64-apple-darwin
✓ Build aarch64-apple-darwin
✓ Build x86_64-pc-windows-msvc
```

### Typical Runtime
- **Validation:** ~30 seconds
- **Tests:** ~2-5 minutes
- **Builds:** ~10-15 minutes (parallel)
- **Release:** ~30 seconds
- **Total:** ~15-20 minutes

## Release Artifacts

### Files Generated

Each release includes:
- **6 platform-specific archives:**
  - `wraith-x86_64-linux-gnu.tar.gz`
  - `wraith-x86_64-linux-musl.tar.gz`
  - `wraith-aarch64-linux-gnu.tar.gz`
  - `wraith-x86_64-macos.tar.gz`
  - `wraith-aarch64-macos.tar.gz`
  - `wraith-x86_64-windows.zip`

- **Individual checksums:**
  - `wraith-x86_64-linux-gnu.sha256`
  - (one for each platform)

- **Combined checksums:**
  - `SHA256SUMS.txt` - All checksums in one file

### Verifying Checksums

**Linux/macOS:**
```bash
# Download the archive and checksum
wget https://github.com/doublegate/WRAITH-Protocol/releases/download/v0.1.0/wraith-x86_64-linux-gnu.tar.gz
wget https://github.com/doublegate/WRAITH-Protocol/releases/download/v0.1.0/SHA256SUMS.txt

# Verify checksum
sha256sum -c SHA256SUMS.txt --ignore-missing
```

**Windows (PowerShell):**
```powershell
# Download the archive
Invoke-WebRequest -Uri "https://github.com/doublegate/WRAITH-Protocol/releases/download/v0.1.0/wraith-x86_64-windows.zip" -OutFile "wraith.zip"

# Verify checksum (compare with SHA256SUMS.txt)
Get-FileHash wraith.zip -Algorithm SHA256
```

## Release Notes

### Automatic Generation

The workflow automatically extracts release notes from `CHANGELOG.md` using the version number. For example, for `v0.1.0`, it extracts the section between:
```markdown
## [0.1.0] - 2025-11-29
```
and the next version header.

### Fallback Behavior

If no CHANGELOG entry is found:
- **Warning is logged** during validation (workflow continues)
- **Generic release notes** are created with a link to CHANGELOG.md
- **Consider updating CHANGELOG.md** before creating the release

### Manual Editing

After release creation, you can:
1. Navigate to the **Releases** page on GitHub
2. Click **Edit** on the release
3. Modify the release notes in the web editor
4. Save changes

## Troubleshooting

### Validation Failures

**Error: Invalid version format**
```
❌ Invalid version format: v1.0
Expected format: vX.Y.Z or vX.Y.Z-prerelease
```

**Solution:** Use semantic versioning with three numbers:
- Correct: `v1.0.0`, `v1.0.0-beta.1`
- Incorrect: `v1.0`, `v1`, `1.0.0` (missing 'v')

**Warning: No CHANGELOG entry found**
```
⚠️  WARNING: No CHANGELOG.md entry found for version v0.1.0
```

**Solution:** Add a section to CHANGELOG.md:
```markdown
## [0.1.0] - 2025-11-29

### Added
- Your changes here
```

### Build Failures

**Cross-compilation errors**

If cross-compilation fails for a specific target:
1. Check the **Build** job logs for that platform
2. Common issues:
   - Missing dependencies for target
   - Platform-specific code issues
   - Cross-compilation tool errors

**Solution:** The workflow uses `cross` for musl/aarch64 targets. Ensure your code compiles for all targets locally:
```bash
# Install cross
cargo install cross

# Test cross-compilation
cross build --target x86_64-unknown-linux-musl --release
cross build --target aarch64-unknown-linux-gnu --release
```

### Test Failures

If tests fail during the release workflow:
1. The release will **not be created** (fail-fast behavior)
2. Check the **Test Suite** job logs
3. Fix the failing tests locally
4. Commit the fixes
5. Re-create the tag:
   ```bash
   # Delete the local tag
   git tag -d v0.1.0

   # Delete the remote tag
   git push origin :refs/tags/v0.1.0

   # Create a new tag after fixes
   git tag -a v0.1.0 -m "WRAITH Protocol v0.1.0"
   git push origin v0.1.0
   ```

### Release Already Exists

**Error:** If a release already exists for a tag, the workflow will fail.

**Solution:**
1. **Option A:** Delete the existing release on GitHub, then re-run the workflow
2. **Option B:** Create a new patch version (e.g., `v0.1.1` instead of `v0.1.0`)

## Best Practices

### Pre-Release Checklist

Before creating a release:

- [ ] All tests pass locally: `cargo test --workspace`
- [ ] Clippy checks pass: `cargo clippy --workspace -- -D warnings`
- [ ] Code is formatted: `cargo fmt --all -- --check`
- [ ] CHANGELOG.md is updated with version section
- [ ] Version is updated in `Cargo.toml`
- [ ] All changes are committed to main branch
- [ ] CI workflow passes on main branch

### Version Numbering

Follow [Semantic Versioning 2.0.0](https://semver.org/):

- **MAJOR** (X.0.0) - Incompatible API changes
- **MINOR** (0.X.0) - Backwards-compatible new features
- **PATCH** (0.0.X) - Backwards-compatible bug fixes
- **Pre-release** (0.1.0-alpha.1) - Unstable releases

Examples:
- `v0.1.0` - Initial development release
- `v0.2.0` - Added new features (backwards-compatible)
- `v0.2.1` - Bug fixes only
- `v1.0.0` - First stable release
- `v1.0.0-rc.1` - Release candidate before v1.0.0
- `v2.0.0` - Breaking changes from v1.x.x

### CHANGELOG.md Format

Use [Keep a Changelog](https://keepachangelog.com/) format:

```markdown
## [0.2.0] - 2025-12-15

### Added
- New feature: DHT peer discovery
- Support for NAT traversal

### Changed
- Updated cryptographic library to latest version
- Improved error messages

### Fixed
- Fixed memory leak in session management
- Resolved race condition in stream multiplexing

### Security
- Patched timing attack vulnerability in handshake
```

Categories:
- **Added** - New features
- **Changed** - Changes to existing functionality
- **Deprecated** - Soon-to-be-removed features
- **Removed** - Removed features
- **Fixed** - Bug fixes
- **Security** - Security fixes

## Advanced Usage

### Creating Draft Releases

Draft releases allow you to prepare a release without publishing:

**Manual Workflow:**
1. Run workflow with **"Create as draft"** checked
2. Review the draft release on GitHub
3. Edit release notes if needed
4. **Publish** when ready

**Use cases:**
- Review release notes before announcement
- Test artifact downloads
- Coordinate release timing

### Hotfix Releases

For urgent bug fixes:

1. Create a hotfix branch from the release tag:
   ```bash
   git checkout -b hotfix/v0.1.1 v0.1.0
   ```

2. Make and commit the fix:
   ```bash
   git commit -am "fix: critical security vulnerability"
   ```

3. Update CHANGELOG.md with hotfix section
4. Create and push the new tag:
   ```bash
   git tag -a v0.1.1 -m "Hotfix: Security vulnerability"
   git push origin hotfix/v0.1.1
   git push origin v0.1.1
   ```

### Managing Multiple Release Channels

For projects with stable and beta channels:

**Stable releases:**
```bash
git tag v1.0.0
```

**Beta releases:**
```bash
git tag v1.1.0-beta.1
git tag v1.1.0-beta.2
git tag v1.1.0  # Final release
```

Users can install specific channels:
```bash
# Latest stable
wget https://github.com/doublegate/WRAITH-Protocol/releases/latest/download/wraith-x86_64-linux-gnu.tar.gz

# Specific beta
wget https://github.com/doublegate/WRAITH-Protocol/releases/download/v1.1.0-beta.1/wraith-x86_64-linux-gnu.tar.gz
```

## CI/CD Integration

### Automatic Deployments

The release workflow integrates with the existing CI workflow:

1. **CI Workflow** (`.github/workflows/ci.yml`)
   - Runs on every push/PR
   - Validates code quality
   - Runs tests

2. **Release Workflow** (`.github/workflows/release.yml`)
   - Runs on version tags
   - Reuses CI patterns (caching, toolchain setup)
   - Extends CI with cross-platform builds

### Caching Strategy

The release workflow uses cargo caching to speed up builds:
- **Cache key includes:** OS, target, and Cargo.lock hash
- **Shared with CI:** Same cache strategy as ci.yml
- **Invalidation:** Automatic when dependencies change

## Future Enhancements

Planned improvements to the release workflow:

- [ ] **Cargo publish** - Publish crates to crates.io
- [ ] **Docker images** - Build and push Docker images
- [ ] **Homebrew tap** - Create Homebrew formula
- [ ] **APT/RPM packages** - Linux package manager support
- [ ] **Code signing** - Sign macOS/Windows binaries
- [ ] **Release announcements** - Automatic Discord/Twitter posts

## Security Considerations

### Workflow Security

The release workflow follows GitHub Actions security best practices:

1. **No command injection vulnerabilities**
   - All user inputs passed through environment variables
   - No direct interpolation of GitHub context in `run:` commands

2. **Minimal permissions**
   - Only `contents: write` for release creation
   - No access to secrets beyond GITHUB_TOKEN

3. **Supply chain security**
   - Uses pinned action versions (e.g., `@v6`, `@v4`)
   - Verifies checksums of all artifacts
   - Strips binaries to remove debug information

### Artifact Verification

Always verify downloaded artifacts:
```bash
# Download the combined checksums
wget https://github.com/doublegate/WRAITH-Protocol/releases/download/v0.1.0/SHA256SUMS.txt

# Verify your download
sha256sum -c SHA256SUMS.txt --ignore-missing
```

Expected output:
```
wraith-x86_64-linux-gnu.tar.gz: OK
```

## References

- [GitHub Actions: Publishing Releases](https://docs.github.com/en/actions/advanced-guides/storing-workflow-data-as-artifacts)
- [Semantic Versioning 2.0.0](https://semver.org/)
- [Keep a Changelog](https://keepachangelog.com/)
- [Cross-compilation with `cross`](https://github.com/cross-rs/cross)
- [GitHub Actions Security Best Practices](https://docs.github.com/en/actions/security-guides/security-hardening-for-github-actions)

## Support

For issues with the release workflow:
1. Check the **Actions** tab for detailed logs
2. Review this guide for common issues
3. Open an issue on GitHub with workflow run link
4. Include relevant log excerpts

---

**Last Updated:** 2025-11-29
**Workflow Version:** 1.0.0
**Maintainer:** WRAITH Protocol Team
