#!/bin/sh
# journal-cli installer and updater for Linux

set -e

# Configuration
REPO="DerSton/journal-cli"
BINARY_NAME="journal-cli"
INSTALL_DIR="$HOME/.local/bin"

echo "=== journal-cli Installer ==="

# Check OS and Architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

if [ "$OS" != "Linux" ] || [ "$ARCH" != "x86_64" ]; then
    echo "Error: Currently, prebuilt binaries are only provided for Linux (x86_64)."
    echo "If you are on macOS or a different architecture, please install via Cargo:"
    echo "  cargo install --git https://github.com/DerSton/journal-cli"
    exit 1
fi

ASSET_NAME="journal-cli-linux-x86_64"

# Fetch latest release version from GitHub API (including pre-releases)
echo "Fetching latest release version from GitHub..."
LATEST_TAG=""
if command -v curl >/dev/null 2>&1; then
    LATEST_TAG=$(curl -s "https://api.github.com/repos/${REPO}/releases" | grep -m1 '"tag_name":' | cut -d'"' -f4 || true)
elif command -v wget >/dev/null 2>&1; then
    LATEST_TAG=$(wget -qO- "https://api.github.com/repos/${REPO}/releases" | grep -m1 '"tag_name":' | cut -d'"' -f4 || true)
fi

if [ -n "$LATEST_TAG" ]; then
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST_TAG}/${ASSET_NAME}"
    echo "Latest release found: $LATEST_TAG"
else
    DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${ASSET_NAME}"
    echo "Warning: Could not fetch latest release version from API. Falling back to latest redirect."
fi

# Create install directory if it doesn't exist
mkdir -p "$INSTALL_DIR"

# Download binary
TEMP_FILE="$(mktemp)"
echo "Downloading journal-cli..."
if command -v curl >/dev/null 2>&1; then
    if ! curl -fsSL "$DOWNLOAD_URL" -o "$TEMP_FILE"; then
        echo "Error: Failed to download binary from $DOWNLOAD_URL"
        echo "Please ensure that a release has been published at https://github.com/${REPO}/releases"
        exit 1
    fi
elif command -v wget >/dev/null 2>&1; then
    if ! wget -qO "$TEMP_FILE" "$DOWNLOAD_URL"; then
        echo "Error: Failed to download binary from $DOWNLOAD_URL"
        echo "Please ensure that a release has been published at https://github.com/${REPO}/releases"
        exit 1
    fi
else
    echo "Error: curl or wget is required to run this installer script."
    exit 1
fi

# Make binary executable
chmod +x "$TEMP_FILE"

# Move binary to install directory
mv "$TEMP_FILE" "$INSTALL_DIR/$BINARY_NAME"

echo "Successfully installed/updated journal-cli to $INSTALL_DIR/$BINARY_NAME"

# Check if PATH contains install directory
case ":$PATH:" in
    *:"$INSTALL_DIR":*)
        # Already in path
        ;;
    *)
        echo ""
        echo "Warning: $INSTALL_DIR is not in your PATH."
        echo "To run journal-cli from anywhere, add it to your shell configuration file (e.g., ~/.bashrc or ~/.zshrc):"
        echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
        echo ""
        ;;
esac

echo "Run 'journal-cli --help' to verify the installation."
