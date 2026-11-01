#!/usr/bin/env bash
# ==============================================================================
# Mizan · High-Assurance NIST OSCAL Compliance Kernel & Workbench
# Zero-Dependency Automated Installer for macOS & Linux
# ==============================================================================
set -euo pipefail

REPO="ckodex-labs/ckodex-oscal-cli"
DEFAULT_VERSION="v0.1.0"
VERSION="${MIZAN_VERSION:-${DEFAULT_VERSION}}"

# Terminal styling
BOLD="\033[1m"
GREEN="\033[32m"
BLUE="\033[34m"
YELLOW="\033[33m"
RED="\033[31m"
RESET="\033[0m"

log_info()  { printf "${BLUE}${BOLD}==>${RESET} %s\n" "$*"; }
log_ok()    { printf "${GREEN}${BOLD}==>${RESET} %s\n" "$*"; }
log_warn()  { printf "${YELLOW}${BOLD}==>${RESET} %s\n" "$*"; }
log_error() { printf "${RED}${BOLD}==>${RESET} %s\n" "$*"; }

printf "${BOLD}Mizan · High-Assurance OSCAL v1.2.3 Compliance Kernel Installer${RESET}\n\n"

# 1. Detect Operating System and Architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}" in
    Darwin)
        case "${ARCH}" in
            arm64|aarch64) TARGET="aarch64-apple-darwin" ;;
            x86_64)       TARGET="x86_64-apple-darwin" ;;
            *) log_error "Unsupported macOS architecture: ${ARCH}"; exit 1 ;;
        esac
        ;;
    Linux)
        case "${ARCH}" in
            x86_64)       TARGET="x86_64-unknown-linux-gnu" ;;
            aarch64|arm64) TARGET="aarch64-unknown-linux-gnu" ;;
            *) log_error "Unsupported Linux architecture: ${ARCH}"; exit 1 ;;
        esac
        ;;
    *)
        log_error "Unsupported operating system: ${OS}. Supported: macOS (Darwin), Linux."
        exit 1
        ;;
esac

log_info "Detected Platform: ${OS} (${ARCH}) -> Target: ${TARGET}"
log_info "Target Version:   ${VERSION}"

# 2. Determine Installation Directory
if [[ -w "/usr/local/bin" ]]; then
    INSTALL_DIR="/usr/local/bin"
elif [[ -d "$HOME/.local/bin" ]] || mkdir -p "$HOME/.local/bin" 2>/dev/null; then
    INSTALL_DIR="$HOME/.local/bin"
else
    INSTALL_DIR="/usr/local/bin"
fi

# 3. Download Release Asset or Fallback to Cargo
TAR_NAME="mizan-${VERSION}-${TARGET}.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${TAR_NAME}"
TMP_DIR="$(mktemp -d -t mizan-install-XXXXXX)"
trap 'rm -rf "${TMP_DIR}"' EXIT

log_info "Fetching release binary from GitHub..."
DOWNLOADED=false
if curl -fsSL "${DOWNLOAD_URL}" -o "${TMP_DIR}/${TAR_NAME}" 2>/dev/null; then
    DOWNLOADED=true
fi

if [[ "${DOWNLOADED}" == "true" ]]; then
    log_info "Extracting binary..."
    tar -xzf "${TMP_DIR}/${TAR_NAME}" -C "${TMP_DIR}"
    
    # Locate mizan executable in extracted files
    BIN_SRC="$(find "${TMP_DIR}" -type f -name "mizan" | head -n 1)"
    if [[ -z "${BIN_SRC}" ]]; then
        log_error "Executable 'mizan' not found in downloaded archive."
        exit 1
    fi
else
    log_warn "Pre-built binary archive not reachable at ${DOWNLOAD_URL}."
    if command -v cargo >/dev/null 2>&1; then
        log_info "Cargo detected. Building and installing Mizan from source via cargo..."
        cargo install --git "https://github.com/${REPO}" --bin mizan --locked
        log_ok "Successfully compiled and installed mizan with Cargo!"
        exit 0
    else
        log_error "Could not download pre-built release and 'cargo' is not installed."
        log_error "Please install Rust (https://rustup.rs) or download binaries from:"
        log_error "https://github.com/${REPO}/releases"
        exit 1
    fi
fi

# 4. Install Binary to Target Location
chmod +x "${BIN_SRC}"

if [[ -w "${INSTALL_DIR}" ]]; then
    cp "${BIN_SRC}" "${INSTALL_DIR}/mizan"
else
    log_info "Elevated permissions required to install to ${INSTALL_DIR}."
    sudo cp "${BIN_SRC}" "${INSTALL_DIR}/mizan"
    sudo chmod 755 "${INSTALL_DIR}/mizan"
fi

# Also link oscal-cli alias if possible
if [[ -w "${INSTALL_DIR}" ]]; then
    ln -sf "${INSTALL_DIR}/mizan" "${INSTALL_DIR}/oscal-cli" 2>/dev/null || true
else
    sudo ln -sf "${INSTALL_DIR}/mizan" "${INSTALL_DIR}/oscal-cli" 2>/dev/null || true
fi

# 5. Verification
if command -v mizan >/dev/null 2>&1; then
    INSTALLED_PATH="$(command -v mizan)"
    log_ok "Mizan installed successfully at: ${INSTALLED_PATH}"
else
    log_ok "Mizan binary installed at: ${INSTALL_DIR}/mizan"
    if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
        log_warn "Note: ${INSTALL_DIR} is not in your \$PATH."
        log_warn "Add it to your profile: export PATH=\"${INSTALL_DIR}:\$PATH\""
    fi
fi

printf "\n"
printf "${BOLD}Quick Verification:${RESET}\n"
printf "  mizan doctor\n"
printf "  mizan validate examples/sample-ssp.json\n\n"
printf "${BOLD}Shell Completions:${RESET}\n"
printf "  mizan completions zsh > ~/.zfunc/_mizan\n"
