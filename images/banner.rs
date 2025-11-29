//! # WRAITH Terminal Banner Module
//!
//! This module provides ASCII art banners and branding elements for the WRAITH
//! protocol CLI application. Multiple banner styles are available for different
//! contexts (startup, help, compact, etc.).
//!
//! ## Usage
//!
//! ```rust
//! use wraith::banner::{print_banner, BannerStyle};
//!
//! // Print the default startup banner
//! print_banner(BannerStyle::Default);
//!
//! // Print a compact banner for help screens
//! print_banner(BannerStyle::Compact);
//! ```
//!
//! ## Banner Styles
//!
//! - `Default` - Full featured banner with ASCII art
//! - `Compact` - Single box, suitable for headers
//! - `Minimal` - One-liner for tight spaces
//! - `Startup` - Version-aware startup message
//! - `Ghost` - Includes wraith figure ASCII art
//! - `Help` - Usage information banner
//!
//! ## Color Support
//!
//! Colors are automatically disabled when:
//! - `NO_COLOR` environment variable is set
//! - Output is not a TTY
//! - `--no-color` flag is passed
//!
//! ## Author
//!
//! WRAITH Protocol Development Team

use std::io::{self, Write};

// =============================================================================
// ANSI Color Constants
// =============================================================================

/// ANSI escape code for reset
pub const RESET: &str = "\x1b[0m";
/// ANSI escape code for bold
pub const BOLD: &str = "\x1b[1m";
/// ANSI escape code for dim
pub const DIM: &str = "\x1b[2m";

/// Cyan foreground (256-color: 51)
pub const CYAN: &str = "\x1b[38;5;51m";
/// Bright cyan foreground (256-color: 87)
pub const BRIGHT_CYAN: &str = "\x1b[38;5;87m";
/// Purple foreground (256-color: 135)
pub const PURPLE: &str = "\x1b[38;5;135m";
/// White foreground (256-color: 255)
pub const WHITE: &str = "\x1b[38;5;255m";
/// Gray foreground (256-color: 245)
pub const GRAY: &str = "\x1b[38;5;245m";
/// Dark gray foreground (256-color: 240)
pub const DARK_GRAY: &str = "\x1b[38;5;240m";

// =============================================================================
// Banner Style Enum
// =============================================================================

/// Available banner display styles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BannerStyle {
    /// Full featured banner with ASCII art logo
    Default,
    /// Single-line compact box
    Compact,
    /// Minimal one-liner
    Minimal,
    /// Startup banner with version
    Startup,
    /// Banner with ghost ASCII art figure
    Ghost,
    /// Slim horizontal line style
    Slim,
    /// Box-enclosed full banner
    Box,
    /// Help/usage banner
    Help,
    /// Gradient effect (256-color)
    Gradient,
}

// =============================================================================
// Color Support Detection
// =============================================================================

/// Check if colors should be enabled
pub fn colors_enabled() -> bool {
    // Check NO_COLOR environment variable
    if std::env::var("NO_COLOR").is_ok() {
        return false;
    }
    
    // Check if stdout is a TTY
    use std::io::IsTerminal;
    io::stdout().is_terminal()
}

/// Conditionally apply color code
fn color(code: &str, use_color: bool) -> &str {
    if use_color { code } else { "" }
}

// =============================================================================
// Banner Definitions
// =============================================================================

/// The main WRAITH ASCII art logo (block letters)
pub const LOGO_BLOCK: &str = r#"
 ██╗    ██╗██████╗  █████╗ ██╗████████╗██╗  ██╗
 ██║    ██║██╔══██╗██╔══██╗██║╚══██╔══╝██║  ██║
 ██║ █╗ ██║██████╔╝███████║██║   ██║   ███████║
 ██║███╗██║██╔══██╗██╔══██║██║   ██║   ██╔══██║
 ╚███╔███╔╝██║  ██║██║  ██║██║   ██║   ██║  ██║
  ╚══╝╚══╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝   ╚═╝   ╚═╝  ╚═╝
"#;

/// The acronym expansion
pub const ACRONYM: &str = "Wire-speed Resilient Authenticated Invisible Transfer Handler";

/// The tagline
pub const TAGLINE: &str = "… your ghost in the network …";

// =============================================================================
// Banner Printing Functions
// =============================================================================

/// Print a banner in the specified style
pub fn print_banner(style: BannerStyle) {
    let use_color = colors_enabled();
    
    match style {
        BannerStyle::Default => print_default_banner(use_color),
        BannerStyle::Compact => print_compact_banner(use_color),
        BannerStyle::Minimal => print_minimal_banner(use_color),
        BannerStyle::Startup => print_startup_banner(env!("CARGO_PKG_VERSION"), use_color),
        BannerStyle::Ghost => print_ghost_banner(use_color),
        BannerStyle::Slim => print_slim_banner(use_color),
        BannerStyle::Box => print_box_banner(use_color),
        BannerStyle::Help => print_help_banner(use_color),
        BannerStyle::Gradient => print_gradient_banner(use_color),
    }
}

/// Print the default full-featured banner
pub fn print_default_banner(use_color: bool) {
    let c = |code| color(code, use_color);
    
    println!("{}", c(DARK_GRAY));
    println!("                        ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░");
    println!("{}", c(RESET));
    println!("{}", c(CYAN));
    println!(" ██╗    ██╗██████╗  █████╗ ██╗████████╗██╗  ██╗");
    println!(" ██║    ██║██╔══██╗██╔══██╗██║╚══██╔══╝██║  ██║");
    println!(" ██║ █╗ ██║██████╔╝███████║██║   ██║   ███████║");
    println!(" ██║███╗██║██╔══██╗██╔══██║██║   ██║   ██╔══██║");
    println!(" ╚███╔███╔╝██║  ██║██║  ██║██║   ██║   ██║  ██║");
    println!("  ╚══╝╚══╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝   ╚═╝   ╚═╝  ╚═╝");
    println!("{}", c(RESET));
    println!("{}  {}{}", c(GRAY), ACRONYM, c(RESET));
    println!();
    println!("{}                 {}{}", c(PURPLE), TAGLINE, c(RESET));
    println!("{}", c(DARK_GRAY));
    println!("                        ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░");
    println!("{}", c(RESET));
}

/// Print compact single-box banner
pub fn print_compact_banner(use_color: bool) {
    let c = |code| color(code, use_color);
    
    println!("{}╔══════════════════════════════════════════════════════════════╗{}", c(CYAN), c(RESET));
    println!("{}║{}  {}{}W R A I T H{}  {}│{} {}Wire-speed Resilient Authenticated{}     {}║{}",
        c(CYAN), c(RESET), c(BOLD), c(WHITE), c(RESET), c(GRAY), c(RESET), c(DIM), c(RESET), c(CYAN), c(RESET));
    println!("{}║{}  {}{}{}  {}│{} {}Invisible Transfer Handler{}  {}║{}",
        c(CYAN), c(RESET), c(PURPLE), TAGLINE, c(RESET), c(GRAY), c(RESET), c(DIM), c(RESET), c(CYAN), c(RESET));
    println!("{}╚══════════════════════════════════════════════════════════════╝{}", c(CYAN), c(RESET));
}

/// Print minimal one-liner banner
pub fn print_minimal_banner(use_color: bool) {
    let c = |code| color(code, use_color);
    
    println!("{}{}WRAITH{} {}│{} {}{}{}│{} {}{}{}",
        c(BOLD), c(CYAN), c(RESET),
        c(GRAY), c(RESET),
        c(DIM), ACRONYM, c(RESET),
        c(GRAY), c(RESET),
        c(PURPLE), TAGLINE, c(RESET));
}

/// Print startup banner with version
pub fn print_startup_banner(version: &str, use_color: bool) {
    let c = |code| color(code, use_color);
    
    println!();
    println!("{} ██╗    ██╗██████╗  █████╗ ██╗████████╗██╗  ██╗{}", c(CYAN), c(RESET));
    println!("{} ██║    ██║██╔══██╗██╔══██╗██║╚══██╔══╝██║  ██║{}", c(CYAN), c(RESET));
    println!("{} ██║ █╗ ██║██████╔╝███████║██║   ██║   ███████║{}   {}v{}{}", 
        c(CYAN), c(RESET), c(GRAY), version, c(RESET));
    println!("{} ██║███╗██║██╔══██╗██╔══██║██║   ██║   ██╔══██║{}", c(CYAN), c(RESET));
    println!("{} ╚███╔███╔╝██║  ██║██║  ██║██║   ██║   ██║  ██║{}", c(CYAN), c(RESET));
    println!("{}  ╚══╝╚══╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝   ╚═╝   ╚═╝  ╚═╝{}", c(CYAN), c(RESET));
    println!();
    println!("{}  {}{}", c(PURPLE), TAGLINE, c(RESET));
    println!();
    println!("{}  Starting WRAITH daemon...{}", c(GRAY), c(RESET));
    println!();
}

/// Print ghost figure banner
pub fn print_ghost_banner(use_color: bool) {
    let c = |code| color(code, use_color);
    
    println!("{}    ┌─────────────────────────────────────────────────────────────────┐{}", c(DARK_GRAY), c(RESET));
    println!("{}    │{}                                                                 {}│{}", c(DARK_GRAY), c(RESET), c(DARK_GRAY), c(RESET));
    println!("{}    │{}{}       01101001                               10110010       {}{}│{}", c(DARK_GRAY), c(RESET), c(DIM), c(RESET), c(DARK_GRAY), c(RESET));
    println!("{}    │{}{}         01110111                           01100001         {}{}│{}", c(DARK_GRAY), c(RESET), c(DIM), c(RESET), c(DARK_GRAY), c(RESET));
    println!("{}    │{}                       {}▄▄▄████████▄▄▄{}                       {}│{}", c(DARK_GRAY), c(RESET), c(GRAY), c(RESET), c(DARK_GRAY), c(RESET));
    println!("{}    │{}                    {}▄██{}░░░░░░░░░░░░{}██▄{}                    {}│{}", c(DARK_GRAY), c(RESET), c(GRAY), c(WHITE), c(GRAY), c(RESET), c(DARK_GRAY), c(RESET));
    println!("{}    │{}                   {}██{}░░░{}▀▀▀▀▀▀▀▀{}░░░{}██{}                   {}│{}", c(DARK_GRAY), c(RESET), c(GRAY), c(WHITE), c(DARK_GRAY), c(WHITE), c(GRAY), c(RESET), c(DARK_GRAY), c(RESET));
    println!("{}    │{}                  {}██{}░░{}▀{}  {}◉{}    {}◉{}  {}▀{}░░{}██{}                  {}│{}", 
        c(DARK_GRAY), c(RESET), c(GRAY), c(WHITE), c(DARK_GRAY), c(RESET), c(CYAN), c(RESET), c(CYAN), c(RESET), c(DARK_GRAY), c(WHITE), c(GRAY), c(RESET), c(DARK_GRAY), c(RESET));
    println!("{}    │{}                  {}██{}░░░░░░░░░░░░░░░░{}██{}                  {}│{}", c(DARK_GRAY), c(RESET), c(GRAY), c(WHITE), c(GRAY), c(RESET), c(DARK_GRAY), c(RESET));
    println!("{}    │{}                  {}██{}░░{}┌──┐{}░░░░{}┌──┐{}░░{}██{}                  {}│{}", 
        c(DARK_GRAY), c(RESET), c(GRAY), c(WHITE), c(CYAN), c(WHITE), c(CYAN), c(WHITE), c(GRAY), c(RESET), c(DARK_GRAY), c(RESET));
    println!("{}    │{}                  {}██{}░░{}└──┘{}░░░░{}└──┘{}░░{}██{}                  {}│{}", 
        c(DARK_GRAY), c(RESET), c(GRAY), c(WHITE), c(CYAN), c(WHITE), c(CYAN), c(WHITE), c(GRAY), c(RESET), c(DARK_GRAY), c(RESET));
    println!("{}    │{}                   {}██{}░░░░░░░░░░░░░░{}██{}                   {}│{}", c(DARK_GRAY), c(RESET), c(GRAY), c(WHITE), c(GRAY), c(RESET), c(DARK_GRAY), c(RESET));
    println!("{}    │{}                    {}██{}░░{}┌────┐{}░░{}██{}                    {}│{}", 
        c(DARK_GRAY), c(RESET), c(GRAY), c(WHITE), c(CYAN), c(WHITE), c(GRAY), c(RESET), c(DARK_GRAY), c(RESET));
    println!("{}    │{}                     {}▀██{}░░░░░░{}██▀{}                     {}│{}", c(DARK_GRAY), c(RESET), c(GRAY), c(WHITE), c(GRAY), c(RESET), c(DARK_GRAY), c(RESET));
    println!("{}    │{}                   {}░░{} {}▀██████▀{} {}░░{}                   {}│{}", 
        c(DARK_GRAY), c(RESET), c(DIM), c(GRAY), c(RESET), c(DIM), c(GRAY), c(RESET), c(DARK_GRAY), c(RESET));
    println!("{}    │{}                  {}░░░░{}  {}░░░░{}  {}░░░░{}                  {}│{}", 
        c(DARK_GRAY), c(RESET), c(DIM), c(GRAY), c(RESET), c(DIM), c(GRAY), c(RESET), c(DARK_GRAY), c(RESET));
    println!("{}    │{}                                                                 {}│{}", c(DARK_GRAY), c(RESET), c(DARK_GRAY), c(RESET));
    println!("{}    │{}    {}██╗    ██╗██████╗  █████╗ ██╗████████╗██╗  ██╗{}              {}│{}", c(DARK_GRAY), c(RESET), c(CYAN), c(RESET), c(DARK_GRAY), c(RESET));
    println!("{}    │{}    {}██║    ██║██╔══██╗██╔══██╗██║╚══██╔══╝██║  ██║{}              {}│{}", c(DARK_GRAY), c(RESET), c(CYAN), c(RESET), c(DARK_GRAY), c(RESET));
    println!("{}    │{}    {}██║ █╗ ██║██████╔╝███████║██║   ██║   ███████║{}              {}│{}", c(DARK_GRAY), c(RESET), c(CYAN), c(RESET), c(DARK_GRAY), c(RESET));
    println!("{}    │{}    {}██║███╗██║██╔══██╗██╔══██║██║   ██║   ██╔══██║{}              {}│{}", c(DARK_GRAY), c(RESET), c(CYAN), c(RESET), c(DARK_GRAY), c(RESET));
    println!("{}    │{}    {}╚███╔███╔╝██║  ██║██║  ██║██║   ██║   ██║  ██║{}              {}│{}", c(DARK_GRAY), c(RESET), c(CYAN), c(RESET), c(DARK_GRAY), c(RESET));
    println!("{}    │{}     {}╚══╝╚══╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝   ╚═╝   ╚═╝  ╚═╝{}              {}│{}", c(DARK_GRAY), c(RESET), c(CYAN), c(RESET), c(DARK_GRAY), c(RESET));
    println!("{}    │{}                                                                 {}│{}", c(DARK_GRAY), c(RESET), c(DARK_GRAY), c(RESET));
    println!("{}    │{}  {}{}{}  {}│{}", c(DARK_GRAY), c(RESET), c(GRAY), ACRONYM, c(RESET), c(DARK_GRAY), c(RESET));
    println!("{}    │{}                                                                 {}│{}", c(DARK_GRAY), c(RESET), c(DARK_GRAY), c(RESET));
    println!("{}    │{}                  {}{}{}                  {}│{}", c(DARK_GRAY), c(RESET), c(PURPLE), TAGLINE, c(RESET), c(DARK_GRAY), c(RESET));
    println!("{}    │{}                                                                 {}│{}", c(DARK_GRAY), c(RESET), c(DARK_GRAY), c(RESET));
    println!("{}    └─────────────────────────────────────────────────────────────────┘{}", c(DARK_GRAY), c(RESET));
}

/// Print slim horizontal banner
pub fn print_slim_banner(use_color: bool) {
    let c = |code| color(code, use_color);
    
    println!("{}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{}", c(CYAN), c(RESET));
    println!("  {}{}W{}R{}A{}I{}T{}H{}  {}│{}  {}{}{}", 
        c(BOLD), c(WHITE), c(CYAN), c(WHITE), c(CYAN), c(WHITE), c(CYAN), c(RESET),
        c(GRAY), c(RESET), c(DIM), ACRONYM, c(RESET));
    println!("{}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{}", c(CYAN), c(RESET));
}

/// Print box-enclosed banner
pub fn print_box_banner(use_color: bool) {
    let c = |code| color(code, use_color);
    
    println!("{}╔════════════════════════════════════════════════════════════════════════╗{}", c(CYAN), c(RESET));
    println!("{}║{}                                                                        {}║{}", c(CYAN), c(RESET), c(CYAN), c(RESET));
    println!("{}║{}  {}██╗    ██╗██████╗  █████╗ ██╗████████╗██╗  ██╗{}                      {}║{}", c(CYAN), c(RESET), c(WHITE), c(RESET), c(CYAN), c(RESET));
    println!("{}║{}  {}██║    ██║██╔══██╗██╔══██╗██║╚══██╔══╝██║  ██║{}                      {}║{}", c(CYAN), c(RESET), c(WHITE), c(RESET), c(CYAN), c(RESET));
    println!("{}║{}  {}██║ █╗ ██║██████╔╝███████║██║   ██║   ███████║{}                      {}║{}", c(CYAN), c(RESET), c(WHITE), c(RESET), c(CYAN), c(RESET));
    println!("{}║{}  {}██║███╗██║██╔══██╗██╔══██║██║   ██║   ██╔══██║{}                      {}║{}", c(CYAN), c(RESET), c(WHITE), c(RESET), c(CYAN), c(RESET));
    println!("{}║{}  {}╚███╔███╔╝██║  ██║██║  ██║██║   ██║   ██║  ██║{}                      {}║{}", c(CYAN), c(RESET), c(WHITE), c(RESET), c(CYAN), c(RESET));
    println!("{}║{}   {}╚══╝╚══╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝   ╚═╝   ╚═╝  ╚═╝{}                      {}║{}", c(CYAN), c(RESET), c(WHITE), c(RESET), c(CYAN), c(RESET));
    println!("{}║{}                                                                        {}║{}", c(CYAN), c(RESET), c(CYAN), c(RESET));
    println!("{}║{}  {}{}{}        {}║{}", c(CYAN), c(RESET), c(GRAY), ACRONYM, c(RESET), c(CYAN), c(RESET));
    println!("{}║{}                                                                        {}║{}", c(CYAN), c(RESET), c(CYAN), c(RESET));
    println!("{}║{}                    {}{}{}                    {}║{}", c(CYAN), c(RESET), c(PURPLE), TAGLINE, c(RESET), c(CYAN), c(RESET));
    println!("{}║{}                                                                        {}║{}", c(CYAN), c(RESET), c(CYAN), c(RESET));
    println!("{}╚════════════════════════════════════════════════════════════════════════╝{}", c(CYAN), c(RESET));
}

/// Print help/usage banner
pub fn print_help_banner(use_color: bool) {
    let c = |code| color(code, use_color);
    
    println!("{}WRAITH{} - {}{}{}", c(CYAN), c(RESET), c(GRAY), ACRONYM, c(RESET));
    println!("{}{}{}", c(PURPLE), TAGLINE, c(RESET));
    println!();
    println!("{}USAGE:{}", c(WHITE), c(RESET));
    println!("  {}wraith{} <command> [options]", c(CYAN), c(RESET));
    println!();
    println!("{}COMMANDS:{}", c(WHITE), c(RESET));
    println!("  {}send{}      Send a file to a peer", c(CYAN), c(RESET));
    println!("  {}receive{}   Listen for incoming transfers", c(CYAN), c(RESET));
    println!("  {}daemon{}    Start the WRAITH daemon", c(CYAN), c(RESET));
    println!("  {}peers{}     List connected peers", c(CYAN), c(RESET));
    println!("  {}status{}    Show transfer status", c(CYAN), c(RESET));
    println!();
    println!("{}OPTIONS:{}", c(WHITE), c(RESET));
    println!("  {}-h, --help{}       Show this help message", c(GRAY), c(RESET));
    println!("  {}-v, --version{}    Show version information", c(GRAY), c(RESET));
    println!("  {}-c, --config{}     Path to config file", c(GRAY), c(RESET));
    println!();
}

/// Print gradient effect banner (256-color)
pub fn print_gradient_banner(use_color: bool) {
    if !use_color {
        print_default_banner(false);
        return;
    }
    
    println!("\x1b[38;5;51m ██╗    ██╗\x1b[38;5;50m██████╗ \x1b[38;5;49m █████╗ \x1b[38;5;48m██╗\x1b[38;5;47m████████╗\x1b[38;5;46m██╗  ██╗\x1b[0m");
    println!("\x1b[38;5;51m ██║    ██║\x1b[38;5;50m██╔══██╗\x1b[38;5;49m██╔══██╗\x1b[38;5;48m██║\x1b[38;5;47m╚══██╔══╝\x1b[38;5;46m██║  ██║\x1b[0m");
    println!("\x1b[38;5;45m ██║ █╗ ██║\x1b[38;5;44m██████╔╝\x1b[38;5;43m███████║\x1b[38;5;42m██║\x1b[38;5;41m   ██║   \x1b[38;5;40m███████║\x1b[0m");
    println!("\x1b[38;5;39m ██║███╗██║\x1b[38;5;38m██╔══██╗\x1b[38;5;37m██╔══██║\x1b[38;5;36m██║\x1b[38;5;35m   ██║   \x1b[38;5;34m██╔══██║\x1b[0m");
    println!("\x1b[38;5;33m ╚███╔███╔╝\x1b[38;5;32m██║  ██║\x1b[38;5;31m██║  ██║\x1b[38;5;30m██║\x1b[38;5;29m   ██║   \x1b[38;5;28m██║  ██║\x1b[0m");
    println!("\x1b[38;5;27m  ╚══╝╚══╝ \x1b[38;5;26m╚═╝  ╚═╝\x1b[38;5;25m╚═╝  ╚═╝\x1b[38;5;24m╚═╝\x1b[38;5;23m   ╚═╝   \x1b[38;5;22m╚═╝  ╚═╝\x1b[0m");
    println!();
    println!("\x1b[38;5;245m  {}\x1b[0m", ACRONYM);
    println!("\x1b[38;5;135m                 {}\x1b[0m", TAGLINE);
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_banner_styles() {
        // Just verify they don't panic
        for style in [
            BannerStyle::Default,
            BannerStyle::Compact,
            BannerStyle::Minimal,
            BannerStyle::Startup,
            BannerStyle::Ghost,
            BannerStyle::Slim,
            BannerStyle::Box,
            BannerStyle::Help,
            BannerStyle::Gradient,
        ] {
            print_banner(style);
        }
    }

    #[test]
    fn test_constants() {
        assert!(!ACRONYM.is_empty());
        assert!(!TAGLINE.is_empty());
        assert!(!LOGO_BLOCK.is_empty());
    }
}
