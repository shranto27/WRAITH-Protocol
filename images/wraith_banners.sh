#!/bin/bash
# =============================================================================
# WRAITH Terminal Banners
# Wire-speed Resilient Authenticated Invisible Transfer Handler
# =============================================================================
#
# This file contains multiple ASCII art banner variants for WRAITH.
# Includes both plain ASCII and ANSI color-coded versions.
#
# Usage:
#   source wraith_banners.sh
#   wraith_banner          # Default colored banner
#   wraith_banner_plain    # No colors
#   wraith_banner_compact  # Smaller version
#   wraith_banner_minimal  # One-liner
#   wraith_banner_ghost    # With ghost art
#
# =============================================================================

# ANSI Color Codes
RESET='\033[0m'
BOLD='\033[1m'
DIM='\033[2m'

# Foreground colors
CYAN='\033[38;5;51m'
BRIGHT_CYAN='\033[38;5;87m'
PURPLE='\033[38;5;135m'
MAGENTA='\033[38;5;165m'
WHITE='\033[38;5;255m'
GRAY='\033[38;5;245m'
DARK_GRAY='\033[38;5;240m'
CHROME='\033[38;5;252m'

# =============================================================================
# BANNER 1: Full Featured (Default)
# =============================================================================
wraith_banner() {
    echo -e "${DARK_GRAY}"
    echo '                        ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░'
    echo -e "${RESET}"
    echo -e "${CYAN}"
    echo ' ██╗    ██╗██████╗  █████╗ ██╗████████╗██╗  ██╗'
    echo ' ██║    ██║██╔══██╗██╔══██╗██║╚══██╔══╝██║  ██║'
    echo ' ██║ █╗ ██║██████╔╝███████║██║   ██║   ███████║'
    echo ' ██║███╗██║██╔══██╗██╔══██║██║   ██║   ██╔══██║'
    echo ' ╚███╔███╔╝██║  ██║██║  ██║██║   ██║   ██║  ██║'
    echo '  ╚══╝╚══╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝   ╚═╝   ╚═╝  ╚═╝'
    echo -e "${RESET}"
    echo -e "${GRAY}  Wire-speed Resilient Authenticated Invisible Transfer Handler${RESET}"
    echo ""
    echo -e "${PURPLE}                 … your ghost in the network …${RESET}"
    echo -e "${DARK_GRAY}"
    echo '                        ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░'
    echo -e "${RESET}"
}

# =============================================================================
# BANNER 2: Plain ASCII (No Colors)
# =============================================================================
wraith_banner_plain() {
    cat << 'EOF'

 ██╗    ██╗██████╗  █████╗ ██╗████████╗██╗  ██╗
 ██║    ██║██╔══██╗██╔══██╗██║╚══██╔══╝██║  ██║
 ██║ █╗ ██║██████╔╝███████║██║   ██║   ███████║
 ██║███╗██║██╔══██╗██╔══██║██║   ██║   ██╔══██║
 ╚███╔███╔╝██║  ██║██║  ██║██║   ██║   ██║  ██║
  ╚══╝╚══╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝   ╚═╝   ╚═╝  ╚═╝
  Wire-speed Resilient Authenticated Invisible Transfer Handler
                 ... your ghost in the network ...

EOF
}

# =============================================================================
# BANNER 3: Compact Version
# =============================================================================
wraith_banner_compact() {
    echo -e "${CYAN}╔══════════════════════════════════════════════════════════════╗${RESET}"
    echo -e "${CYAN}║${RESET}  ${BOLD}${WHITE}W R A I T H${RESET}  ${GRAY}│${RESET} ${DIM}Wire-speed Resilient Authenticated${RESET}     ${CYAN}║${RESET}"
    echo -e "${CYAN}║${RESET}  ${PURPLE}… your ghost in the network …${RESET}  ${GRAY}│${RESET} ${DIM}Invisible Transfer Handler${RESET}  ${CYAN}║${RESET}"
    echo -e "${CYAN}╚══════════════════════════════════════════════════════════════╝${RESET}"
}

# =============================================================================
# BANNER 4: Minimal One-Liner
# =============================================================================
wraith_banner_minimal() {
    echo -e "${BOLD}${CYAN}WRAITH${RESET} ${GRAY}│${RESET} ${DIM}Wire-speed Resilient Authenticated Invisible Transfer Handler${RESET} ${GRAY}│${RESET} ${PURPLE}… your ghost in the network …${RESET}"
}

# =============================================================================
# BANNER 5: With Ghost ASCII Art
# =============================================================================
wraith_banner_ghost() {
    echo -e "${DARK_GRAY}    ┌─────────────────────────────────────────────────────────────────┐${RESET}"
    echo -e "${DARK_GRAY}    │${RESET}                                                                 ${DARK_GRAY}│${RESET}"
    echo -e "${DARK_GRAY}    │${RESET}${DIM}       01101001                               10110010       ${RESET}${DARK_GRAY}│${RESET}"
    echo -e "${DARK_GRAY}    │${RESET}${DIM}         01110111                           01100001         ${RESET}${DARK_GRAY}│${RESET}"
    echo -e "${DARK_GRAY}    │${RESET}                       ${GRAY}▄▄▄████████▄▄▄${RESET}                       ${DARK_GRAY}│${RESET}"
    echo -e "${DARK_GRAY}    │${RESET}                    ${GRAY}▄██${RESET}${WHITE}░░░░░░░░░░░░${RESET}${GRAY}██▄${RESET}                    ${DARK_GRAY}│${RESET}"
    echo -e "${DARK_GRAY}    │${RESET}                   ${GRAY}██${RESET}${WHITE}░░░${RESET}${DARK_GRAY}▀▀▀▀▀▀▀▀${RESET}${WHITE}░░░${RESET}${GRAY}██${RESET}                   ${DARK_GRAY}│${RESET}"
    echo -e "${DARK_GRAY}    │${RESET}                  ${GRAY}██${RESET}${WHITE}░░${RESET}${DARK_GRAY}▀${RESET}  ${CYAN}◉${RESET}    ${CYAN}◉${RESET}  ${DARK_GRAY}▀${RESET}${WHITE}░░${RESET}${GRAY}██${RESET}                  ${DARK_GRAY}│${RESET}"
    echo -e "${DARK_GRAY}    │${RESET}                  ${GRAY}██${RESET}${WHITE}░░░░░░░░░░░░░░░░${RESET}${GRAY}██${RESET}                  ${DARK_GRAY}│${RESET}"
    echo -e "${DARK_GRAY}    │${RESET}                  ${GRAY}██${RESET}${WHITE}░░${RESET}${CYAN}┌──┐${RESET}${WHITE}░░░░${RESET}${CYAN}┌──┐${RESET}${WHITE}░░${RESET}${GRAY}██${RESET}                  ${DARK_GRAY}│${RESET}"
    echo -e "${DARK_GRAY}    │${RESET}                  ${GRAY}██${RESET}${WHITE}░░${RESET}${CYAN}└──┘${RESET}${WHITE}░░░░${RESET}${CYAN}└──┘${RESET}${WHITE}░░${RESET}${GRAY}██${RESET}                  ${DARK_GRAY}│${RESET}"
    echo -e "${DARK_GRAY}    │${RESET}                   ${GRAY}██${RESET}${WHITE}░░░░░░░░░░░░░░${RESET}${GRAY}██${RESET}                   ${DARK_GRAY}│${RESET}"
    echo -e "${DARK_GRAY}    │${RESET}                    ${GRAY}██${RESET}${WHITE}░░${RESET}${CYAN}┌────┐${RESET}${WHITE}░░${RESET}${GRAY}██${RESET}                    ${DARK_GRAY}│${RESET}"
    echo -e "${DARK_GRAY}    │${RESET}                     ${GRAY}▀██${RESET}${WHITE}░░░░░░${RESET}${GRAY}██▀${RESET}                     ${DARK_GRAY}│${RESET}"
    echo -e "${DARK_GRAY}    │${RESET}                   ${DIM}${GRAY}░░${RESET} ${GRAY}▀██████▀${RESET} ${DIM}${GRAY}░░${RESET}                   ${DARK_GRAY}│${RESET}"
    echo -e "${DARK_GRAY}    │${RESET}                  ${DIM}${GRAY}░░░░${RESET}  ${GRAY}░░░░${RESET}  ${DIM}${GRAY}░░░░${RESET}                  ${DARK_GRAY}│${RESET}"
    echo -e "${DARK_GRAY}    │${RESET}                                                                 ${DARK_GRAY}│${RESET}"
    echo -e "${DARK_GRAY}    │${RESET}    ${CYAN}██╗    ██╗██████╗  █████╗ ██╗████████╗██╗  ██╗${RESET}              ${DARK_GRAY}│${RESET}"
    echo -e "${DARK_GRAY}    │${RESET}    ${CYAN}██║    ██║██╔══██╗██╔══██╗██║╚══██╔══╝██║  ██║${RESET}              ${DARK_GRAY}│${RESET}"
    echo -e "${DARK_GRAY}    │${RESET}    ${CYAN}██║ █╗ ██║██████╔╝███████║██║   ██║   ███████║${RESET}              ${DARK_GRAY}│${RESET}"
    echo -e "${DARK_GRAY}    │${RESET}    ${CYAN}██║███╗██║██╔══██╗██╔══██║██║   ██║   ██╔══██║${RESET}              ${DARK_GRAY}│${RESET}"
    echo -e "${DARK_GRAY}    │${RESET}    ${CYAN}╚███╔███╔╝██║  ██║██║  ██║██║   ██║   ██║  ██║${RESET}              ${DARK_GRAY}│${RESET}"
    echo -e "${DARK_GRAY}    │${RESET}     ${CYAN}╚══╝╚══╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝   ╚═╝   ╚═╝  ╚═╝${RESET}              ${DARK_GRAY}│${RESET}"
    echo -e "${DARK_GRAY}    │${RESET}                                                                 ${DARK_GRAY}│${RESET}"
    echo -e "${DARK_GRAY}    │${RESET}  ${GRAY}Wire-speed Resilient Authenticated Invisible Transfer Handler${RESET}  ${DARK_GRAY}│${RESET}"
    echo -e "${DARK_GRAY}    │${RESET}                                                                 ${DARK_GRAY}│${RESET}"
    echo -e "${DARK_GRAY}    │${RESET}                  ${PURPLE}… your ghost in the network …${RESET}                  ${DARK_GRAY}│${RESET}"
    echo -e "${DARK_GRAY}    │${RESET}                                                                 ${DARK_GRAY}│${RESET}"
    echo -e "${DARK_GRAY}    └─────────────────────────────────────────────────────────────────┘${RESET}"
}

# =============================================================================
# BANNER 6: Slim Horizontal
# =============================================================================
wraith_banner_slim() {
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
    echo -e "  ${BOLD}${WHITE}W${CYAN}R${WHITE}A${CYAN}I${WHITE}T${CYAN}H${RESET}  ${GRAY}│${RESET}  ${DIM}Wire-speed Resilient Authenticated Invisible Transfer Handler${RESET}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
}

# =============================================================================
# BANNER 7: Startup Banner (with version)
# =============================================================================
wraith_banner_startup() {
    local VERSION="${1:-0.1.0}"
    echo ""
    echo -e "${CYAN} ██╗    ██╗██████╗  █████╗ ██╗████████╗██╗  ██╗${RESET}"
    echo -e "${CYAN} ██║    ██║██╔══██╗██╔══██╗██║╚══██╔══╝██║  ██║${RESET}"
    echo -e "${CYAN} ██║ █╗ ██║██████╔╝███████║██║   ██║   ███████║${RESET}   ${GRAY}v${VERSION}${RESET}"
    echo -e "${CYAN} ██║███╗██║██╔══██╗██╔══██║██║   ██║   ██╔══██║${RESET}"
    echo -e "${CYAN} ╚███╔███╔╝██║  ██║██║  ██║██║   ██║   ██║  ██║${RESET}"
    echo -e "${CYAN}  ╚══╝╚══╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝   ╚═╝   ╚═╝  ╚═╝${RESET}"
    echo ""
    echo -e "${PURPLE}  … your ghost in the network …${RESET}"
    echo ""
    echo -e "${GRAY}  Starting WRAITH daemon...${RESET}"
    echo ""
}

# =============================================================================
# BANNER 8: Box Style
# =============================================================================
wraith_banner_box() {
    echo -e "${CYAN}╔════════════════════════════════════════════════════════════════════════╗${RESET}"
    echo -e "${CYAN}║${RESET}                                                                        ${CYAN}║${RESET}"
    echo -e "${CYAN}║${RESET}  ${WHITE}██╗    ██╗██████╗  █████╗ ██╗████████╗██╗  ██╗${RESET}                      ${CYAN}║${RESET}"
    echo -e "${CYAN}║${RESET}  ${WHITE}██║    ██║██╔══██╗██╔══██╗██║╚══██╔══╝██║  ██║${RESET}                      ${CYAN}║${RESET}"
    echo -e "${CYAN}║${RESET}  ${WHITE}██║ █╗ ██║██████╔╝███████║██║   ██║   ███████║${RESET}                      ${CYAN}║${RESET}"
    echo -e "${CYAN}║${RESET}  ${WHITE}██║███╗██║██╔══██╗██╔══██║██║   ██║   ██╔══██║${RESET}                      ${CYAN}║${RESET}"
    echo -e "${CYAN}║${RESET}  ${WHITE}╚███╔███╔╝██║  ██║██║  ██║██║   ██║   ██║  ██║${RESET}                      ${CYAN}║${RESET}"
    echo -e "${CYAN}║${RESET}   ${WHITE}╚══╝╚══╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝   ╚═╝   ╚═╝  ╚═╝${RESET}                      ${CYAN}║${RESET}"
    echo -e "${CYAN}║${RESET}                                                                        ${CYAN}║${RESET}"
    echo -e "${CYAN}║${RESET}  ${GRAY}Wire-speed Resilient Authenticated Invisible Transfer Handler${RESET}        ${CYAN}║${RESET}"
    echo -e "${CYAN}║${RESET}                                                                        ${CYAN}║${RESET}"
    echo -e "${CYAN}║${RESET}                    ${PURPLE}… your ghost in the network …${RESET}                    ${CYAN}║${RESET}"
    echo -e "${CYAN}║${RESET}                                                                        ${CYAN}║${RESET}"
    echo -e "${CYAN}╚════════════════════════════════════════════════════════════════════════╝${RESET}"
}

# =============================================================================
# BANNER 9: Gradient Effect (256-color terminals)
# =============================================================================
wraith_banner_gradient() {
    echo -e "\033[38;5;51m ██╗    ██╗\033[38;5;50m██████╗ \033[38;5;49m █████╗ \033[38;5;48m██╗\033[38;5;47m████████╗\033[38;5;46m██╗  ██╗\033[0m"
    echo -e "\033[38;5;51m ██║    ██║\033[38;5;50m██╔══██╗\033[38;5;49m██╔══██╗\033[38;5;48m██║\033[38;5;47m╚══██╔══╝\033[38;5;46m██║  ██║\033[0m"
    echo -e "\033[38;5;45m ██║ █╗ ██║\033[38;5;44m██████╔╝\033[38;5;43m███████║\033[38;5;42m██║\033[38;5;41m   ██║   \033[38;5;40m███████║\033[0m"
    echo -e "\033[38;5;39m ██║███╗██║\033[38;5;38m██╔══██╗\033[38;5;37m██╔══██║\033[38;5;36m██║\033[38;5;35m   ██║   \033[38;5;34m██╔══██║\033[0m"
    echo -e "\033[38;5;33m ╚███╔███╔╝\033[38;5;32m██║  ██║\033[38;5;31m██║  ██║\033[38;5;30m██║\033[38;5;29m   ██║   \033[38;5;28m██║  ██║\033[0m"
    echo -e "\033[38;5;27m  ╚══╝╚══╝ \033[38;5;26m╚═╝  ╚═╝\033[38;5;25m╚═╝  ╚═╝\033[38;5;24m╚═╝\033[38;5;23m   ╚═╝   \033[38;5;22m╚═╝  ╚═╝\033[0m"
    echo ""
    echo -e "\033[38;5;245m  Wire-speed Resilient Authenticated Invisible Transfer Handler\033[0m"
    echo -e "\033[38;5;135m                 … your ghost in the network …\033[0m"
}

# =============================================================================
# BANNER 10: Help/Usage Banner
# =============================================================================
wraith_banner_help() {
    echo -e "${CYAN}WRAITH${RESET} - ${GRAY}Wire-speed Resilient Authenticated Invisible Transfer Handler${RESET}"
    echo -e "${PURPLE}… your ghost in the network …${RESET}"
    echo ""
    echo -e "${WHITE}USAGE:${RESET}"
    echo -e "  ${CYAN}wraith${RESET} <command> [options]"
    echo ""
    echo -e "${WHITE}COMMANDS:${RESET}"
    echo -e "  ${CYAN}send${RESET}      Send a file to a peer"
    echo -e "  ${CYAN}receive${RESET}   Listen for incoming transfers"
    echo -e "  ${CYAN}daemon${RESET}    Start the WRAITH daemon"
    echo -e "  ${CYAN}peers${RESET}     List connected peers"
    echo -e "  ${CYAN}status${RESET}    Show transfer status"
    echo ""
    echo -e "${WHITE}OPTIONS:${RESET}"
    echo -e "  ${GRAY}-h, --help${RESET}       Show this help message"
    echo -e "  ${GRAY}-v, --version${RESET}    Show version information"
    echo -e "  ${GRAY}-c, --config${RESET}     Path to config file"
    echo ""
}

# =============================================================================
# Export all functions
# =============================================================================
export -f wraith_banner
export -f wraith_banner_plain
export -f wraith_banner_compact
export -f wraith_banner_minimal
export -f wraith_banner_ghost
export -f wraith_banner_slim
export -f wraith_banner_startup
export -f wraith_banner_box
export -f wraith_banner_gradient
export -f wraith_banner_help

# If script is run directly, show the default banner
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    wraith_banner
fi
