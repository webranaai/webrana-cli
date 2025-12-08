#!/bin/bash
# Webrana CLI Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/webranaai/webrana-cli/master/install.sh | bash

set -e

REPO="webranaai/webrana-cli"
INSTALL_DIR="${WEBRANA_INSTALL_DIR:-/usr/local/bin}"
BINARY_NAME="webrana"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

info() {
    echo -e "${CYAN}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Detect OS and architecture
detect_platform() {
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)

    case "$OS" in
        linux)
            OS="linux"
            ;;
        darwin)
            OS="darwin"
            ;;
        mingw*|msys*|cygwin*)
            OS="windows"
            ;;
        *)
            error "Unsupported OS: $OS"
            ;;
    esac

    case "$ARCH" in
        x86_64|amd64)
            ARCH="x86_64"
            ;;
        aarch64|arm64)
            ARCH="aarch64"
            ;;
        *)
            error "Unsupported architecture: $ARCH"
            ;;
    esac

    PLATFORM="${OS}-${ARCH}"
    info "Detected platform: $PLATFORM"
}

# Get latest release version
get_latest_version() {
    info "Fetching latest version..."
    VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    
    if [ -z "$VERSION" ]; then
        error "Failed to get latest version"
    fi
    
    info "Latest version: $VERSION"
}

# Download and install
download_and_install() {
    local FILENAME
    
    case "$OS" in
        windows)
            FILENAME="webrana-${PLATFORM}.exe"
            ;;
        *)
            FILENAME="webrana-${PLATFORM}"
            ;;
    esac

    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${FILENAME}"
    
    info "Downloading from: $DOWNLOAD_URL"
    
    # Create temp directory
    TMP_DIR=$(mktemp -d)
    trap "rm -rf $TMP_DIR" EXIT
    
    # Download
    if command -v curl &> /dev/null; then
        curl -fsSL "$DOWNLOAD_URL" -o "$TMP_DIR/$BINARY_NAME" || error "Download failed"
    elif command -v wget &> /dev/null; then
        wget -q "$DOWNLOAD_URL" -O "$TMP_DIR/$BINARY_NAME" || error "Download failed"
    else
        error "Neither curl nor wget found. Please install one of them."
    fi
    
    # Make executable
    chmod +x "$TMP_DIR/$BINARY_NAME"
    
    # Install
    info "Installing to $INSTALL_DIR..."
    
    if [ -w "$INSTALL_DIR" ]; then
        mv "$TMP_DIR/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
    else
        warn "Need sudo to install to $INSTALL_DIR"
        sudo mv "$TMP_DIR/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
    fi
    
    success "Installed to $INSTALL_DIR/$BINARY_NAME"
}

# Verify installation
verify_installation() {
    if command -v webrana &> /dev/null; then
        success "Webrana CLI installed successfully!"
        echo ""
        webrana --version
        echo ""
        echo "Get started:"
        echo "  webrana --help"
        echo "  webrana chat 'Hello, Webrana!'"
        echo ""
        echo "Configure API key:"
        echo "  export OPENAI_API_KEY=sk-..."
        echo "  # or"
        echo "  export ANTHROPIC_API_KEY=sk-ant-..."
    else
        warn "Installation complete, but 'webrana' not found in PATH"
        warn "Make sure $INSTALL_DIR is in your PATH"
        echo ""
        echo "Add to your shell profile:"
        echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
    fi
}

# Main
main() {
    echo ""
    echo "  ╦ ╦╔═╗╔╗ ╦═╗╔═╗╔╗╔╔═╗  ╔═╗╦  ╦"
    echo "  ║║║║╣ ╠╩╗╠╦╝╠═╣║║║╠═╣  ║  ║  ║"
    echo "  ╚╩╝╚═╝╚═╝╩╚═╩ ╩╝╚╝╩ ╩  ╚═╝╩═╝╩"
    echo "  Autonomous CLI Agent Installer"
    echo ""
    
    detect_platform
    get_latest_version
    download_and_install
    verify_installation
}

main "$@"
