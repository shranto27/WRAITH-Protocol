# Wayland Compatibility Issues

## Problem: Error 71 (Protocol error) dispatching to Wayland display

### Symptoms
- Application crashes immediately after launch (0.5-1 second)
- Terminal shows: `Gdk-Message: Error 71 (Protocol error) dispatching to Wayland display.`
- Window may briefly appear then disappear
- Most common on KDE Plasma 6 with Wayland

### Root Cause

This is a **known upstream issue** in Tauri's dependencies affecting Linux Wayland environments:

1. **WebKitGTK Compatibility**: WebKitGTK (the web rendering engine used by Tauri on Linux) has compatibility issues with Wayland, particularly on KDE Plasma 6. The Tauri team has noted that "webkitgtk is getting worse/more unstable each release."

2. **Window Geometry Constraints**: According to the xdg-shell Wayland specification:
   > "A client cannot set the window geometry of a maximized or fullscreen window, as the window dimensions are determined by the compositor."

   When the application tries to restore window state or set window geometry, it violates Wayland protocol constraints, causing the compositor to terminate the connection with Error 71.

3. **tao Windowing Library**: While tao v0.30.3+ includes fixes for some Wayland issues (processing events sequentially), KDE Plasma 6 specific problems remain. Our application uses tao v0.34.5.

4. **Upstream Status**: Tauri has marked this as "status: upstream" - blocked by webkit2gtk and tao compatibility issues.

### References
- [Tauri Issue #10702: Error 71 on Wayland display](https://github.com/tauri-apps/tauri/issues/10702)
- [tao Issue #977: Wayland protocol error 71](https://github.com/tauri-apps/tao/issues/977)
- [Tauri Issue #13414: Can't run tauri dev using Wayland](https://github.com/tauri-apps/tauri/issues/13414)
- [Tauri Issue #12361: Rendering not working correctly with GDK_BACKEND=wayland](https://github.com/tauri-apps/tauri/issues/12361)
- [KDE Plasma Wayland Known Issues](https://community.kde.org/Plasma/Wayland_Known_Significant_Issues)

## Solutions

### Automatic Fix (Implemented)

As of version 0.1.0, WRAITH Transfer **automatically detects** and fixes both Wayland issues:

#### 1. Wayland Error 71 Fix
Detects KDE Plasma on Wayland and forces X11 backend via XWayland:

```rust
// Automatically fallback to X11 on KDE Plasma + Wayland
#[cfg(target_os = "linux")]
{
    if env::var("GDK_BACKEND").is_err() {
        if let Ok(session_type) = env::var("XDG_SESSION_TYPE") {
            if session_type == "wayland" {
                let is_kde = env::var("KDE_SESSION_VERSION").is_ok()
                    || env::var("KDE_FULL_SESSION").is_ok();
                if is_kde {
                    unsafe { env::set_var("GDK_BACKEND", "x11"); }
                }
            }
        }
    }
}
```

The application will print a message when this workaround activates:
```
Detected KDE Plasma on Wayland - forcing X11 backend to avoid Error 71
See: https://github.com/tauri-apps/tauri/issues/10702
```

#### 2. GBM Buffer Fix
Automatically disables WebKit compositing mode to avoid "Failed to create GBM buffer" errors:

```rust
// Workaround for GBM buffer errors
if env::var("WEBKIT_DISABLE_COMPOSITING_MODE").is_err() {
    unsafe { env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1"); }
}
```

This fix addresses:
- Graphics incompatibility between WebKitGTK, Mesa, and GPU drivers
- NVIDIA GPU issues with hardware-accelerated compositing
- Older hardware lacking full acceleration support

**References:**
- [Tauri Issue #13493: Failed to create GBM buffer](https://github.com/tauri-apps/tauri/issues/13493)
- [Tauri Issue #9304: App window fails under Linux with NVIDIA GPU](https://github.com/tauri-apps/tauri/issues/9304)
- [winfunc/opcode Issue #26: GBM buffer error on Arch Linux](https://github.com/winfunc/opcode/issues/26)

### Manual Workarounds

If you encounter issues or want to override the automatic behavior:

#### 1. Force X11 Backend
```bash
GDK_BACKEND=x11 wraith-transfer
```

Or create a desktop entry with the environment variable:
```ini
[Desktop Entry]
Name=WRAITH Transfer
Exec=env GDK_BACKEND=x11 /path/to/wraith-transfer
Type=Application
```

#### 2. Disable WebKit Compositing (for GBM errors)
```bash
WEBKIT_DISABLE_COMPOSITING_MODE=1 wraith-transfer
```

Forces WebKit to use a simpler rendering path that avoids GBM buffer creation.

#### 3. Combine Both Fixes
```bash
GDK_BACKEND=x11 WEBKIT_DISABLE_COMPOSITING_MODE=1 wraith-transfer
```

#### 4. Try Wayland with X11 Fallback
```bash
GDK_BACKEND=wayland,x11 wraith-transfer
```

This tells GDK to prefer Wayland but fallback to X11 if issues occur.

#### 5. Switch to X11 Session
The most reliable solution is to use an X11 session instead of Wayland:
- Log out of KDE Plasma
- At the login screen, select "Plasma (X11)" session
- Log back in and run WRAITH Transfer normally

### Environment Detection

The automatic fix detects KDE Plasma by checking these environment variables:
- `KDE_SESSION_VERSION` - KDE Plasma version
- `KDE_FULL_SESSION` - Set to "true" in KDE sessions
- `DESKTOP_SESSION` - Contains "plasma" or "kde"

If any of these are set and `XDG_SESSION_TYPE=wayland`, the application will use `GDK_BACKEND=x11`.

## Other Wayland Compositors

While the automatic fix targets KDE Plasma 6 (the most problematic), other Wayland compositors may also have issues:

### Known Compatible
- **GNOME Wayland**: Generally works well with Tauri applications
- **Sway**: May work with `GDK_BACKEND=wayland,x11`

### Known Issues
- **Hyprland**: Some users report crashes (see GitHub issues)
- **wlroots-based compositors**: Variable compatibility

For non-KDE compositors, the application sets `GDK_BACKEND=wayland,x11` to allow GDK to try Wayland first with X11 fallback.

## Development Mode

When running in development mode (`cargo tauri dev`):

```bash
# If you encounter "Failed to initialize gtk backend!" errors:
GDK_BACKEND=x11 cargo tauri dev

# Or set in your shell profile:
export GDK_BACKEND=x11
```

## Testing the Fix

After rebuilding with the automatic fix:

1. **Check for detection message**:
   ```bash
   cargo build --release
   ./target/release/wraith-transfer 2>&1 | grep "Detected KDE"
   ```

   Should print: "Detected KDE Plasma on Wayland - forcing X11 backend to avoid Error 71"

2. **Verify GDK_BACKEND is not set before running**:
   ```bash
   unset GDK_BACKEND
   ./target/release/wraith-transfer
   ```

3. **Test manual override** (respects user preference):
   ```bash
   GDK_BACKEND=wayland ./target/release/wraith-transfer
   ```

   Should NOT print detection message and should use Wayland (may still crash with Error 71).

## Future Improvements

The Wayland compatibility issues are being tracked upstream:

1. **Tauri v3**: May switch to different rendering backends (Chromium via CEF or QtWebEngine)
2. **WebKitGTK improvements**: Ongoing work on GTK4 support
3. **KDE Plasma 6.6+**: May include fixes for protocol errors

We will revisit this workaround as upstream dependencies mature.

## Impact on Features

Using XWayland (X11 backend on Wayland) has minimal impact:

### Still Works
- All application features
- File transfers
- Window management
- System tray (if implemented)
- Multi-monitor support

### Potential Limitations
- **Fractional scaling**: May have slight rendering differences vs native Wayland
- **Touchpad gestures**: Some Wayland-specific gestures may not work
- **Screen recording**: Some Wayland screen recording tools may not capture XWayland windows
- **Variable refresh rate**: XWayland doesn't support VRR (yet)

For most users, these limitations are acceptable trade-offs for a stable, working application.

## Reporting Issues

If you still encounter Wayland-related crashes after these fixes:

1. Check Tauri version: `cargo tree -p tauri`
2. Check tao version: `cargo tree -p tao`
3. Check WebKitGTK version: `pacman -Q webkit2gtk` (Arch) or equivalent
4. Collect diagnostic info:
   ```bash
   echo "XDG_SESSION_TYPE=$XDG_SESSION_TYPE"
   echo "DESKTOP_SESSION=$DESKTOP_SESSION"
   echo "KDE_SESSION_VERSION=$KDE_SESSION_VERSION"
   echo "GDK_BACKEND=$GDK_BACKEND"
   wraith-transfer --version
   ```
5. Open an issue at: https://github.com/doublegate/WRAITH-Protocol/issues

Include the diagnostic output and describe your compositor/desktop environment.
