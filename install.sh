#!/bin/bash
set -e

# py-license-auditor installer
REPO="yayami3/py-license-auditor"
VERSION="v0.1.1"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Installing py-license-auditor...${NC}"

# Detect platform
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
    linux)
        case "$ARCH" in
            x86_64|amd64) TARGET="x86_64-unknown-linux-gnu" ;;
            *) echo -e "${RED}Unsupported architecture: $ARCH${NC}"; exit 1 ;;
        esac
        BINARY="py-license-auditor"
        ;;
    darwin)
        case "$ARCH" in
            x86_64|amd64) TARGET="x86_64-apple-darwin" ;;
            arm64|aarch64) TARGET="aarch64-apple-darwin" ;;
            *) echo -e "${RED}Unsupported architecture: $ARCH${NC}"; exit 1 ;;
        esac
        BINARY="py-license-auditor"
        ;;
    mingw*|msys*|cygwin*)
        case "$ARCH" in
            x86_64|amd64) TARGET="x86_64-pc-windows-msvc" ;;
            *) echo -e "${RED}Unsupported architecture: $ARCH${NC}"; exit 1 ;;
        esac
        BINARY="py-license-auditor.exe"
        ;;
    *)
        echo -e "${RED}Unsupported OS: $OS${NC}"
        exit 1
        ;;
esac

echo "Detected platform: $OS $ARCH -> $TARGET"

# Download URL
URL="https://github.com/$REPO/releases/download/$VERSION/py-license-auditor-$TARGET.tar.gz"

# Install directory
INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"

# Temporary directory
TMP_DIR=$(mktemp -d)
cd "$TMP_DIR"

echo "Downloading from $URL..."
if command -v curl >/dev/null 2>&1; then
    curl -sSL "$URL" -o archive.tar.gz
elif command -v wget >/dev/null 2>&1; then
    wget -q "$URL" -O archive.tar.gz
else
    echo -e "${RED}Error: curl or wget required${NC}"
    exit 1
fi

echo "Extracting..."
tar -xzf archive.tar.gz

# Find and install binary
if [ -f "$BINARY" ]; then
    chmod +x "$BINARY"
    mv "$BINARY" "$INSTALL_DIR/"
    echo -e "${GREEN}✅ Installed to $INSTALL_DIR/$BINARY${NC}"
else
    echo -e "${RED}Error: Binary $BINARY not found in archive${NC}"
    exit 1
fi

# Cleanup
cd /
rm -rf "$TMP_DIR"

# Check PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo -e "${YELLOW}⚠️  Add $INSTALL_DIR to your PATH:${NC}"
    echo "export PATH=\"\$PATH:$INSTALL_DIR\""
    echo ""
    echo "Or add this line to your shell profile (~/.bashrc, ~/.zshrc, etc.)"
fi

echo -e "${GREEN}🎉 Installation complete!${NC}"
echo "Run: py-license-auditor --help"
