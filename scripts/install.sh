#!/usr/bin/env bash
# ====================================================================
# ⚡ DAGR Universal One-Line Installer (macOS & Linux)
# Usage: curl -fsSL https://raw.githubusercontent.com/mjzd7/dagr/main/scripts/install.sh | bash
# ====================================================================
set -e

DAGR_REPO="mjzd7/dagr"
INSTALL_DIR="${DAGR_INSTALL_DIR:-$HOME/.local/bin}"
mkdir -p "$INSTALL_DIR"

OS="$(uname -s)"
ARCH="$(uname -m)"

echo "⚡ [DAGR] Detecting platform: $OS ($ARCH)..."

case "$OS" in
    Darwin)
        if [ "$ARCH" = "arm64" ]; then
            TARGET="dagr-darwin-arm64"
        else
            TARGET="dagr-darwin-x86_64"
        fi
        ;;
    Linux)
        if [ "$ARCH" = "x86_64" ]; then
            TARGET="dagr-linux-x86_64"
        elif [ "$ARCH" = "aarch64" ] || [ "$ARCH" = "arm64" ]; then
            TARGET="dagr-linux-aarch64"
        else
            echo "❌ Unsupported Linux architecture: $ARCH"
            exit 1
        fi
        ;;
    *)
        echo "❌ Unsupported OS: $OS. On Windows, run install.ps1 in PowerShell."
        exit 1
        ;;
esac

DEST="$INSTALL_DIR/dagr"

# 1. Download release binary if available, fallback to cargo install
if command -v cargo >/dev/null 2>&1 && [ -f "Cargo.toml" ]; then
    echo "⚡ [DAGR] Building and installing local release binary with Cargo..."
    cargo install --path crates/dagr-cli --force
else
    DOWNLOAD_URL="https://github.com/$DAGR_REPO/releases/latest/download/$TARGET"
    echo "⚡ [DAGR] Downloading pre-compiled binary from $DOWNLOAD_URL..."
    curl -fsSL "$DOWNLOAD_URL" -o "$DEST" || {
        echo "⚠️  Pre-compiled binary release not reachable. Compiling via cargo..."
        cargo install --git "https://github.com/$DAGR_REPO.git" dagr --force
    }
    chmod +x "$DEST"
fi

# 2. Add to PATH if not already present
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]] && [[ ":$PATH:" != *":$HOME/.cargo/bin:"* ]]; then
    export PATH="$INSTALL_DIR:$PATH"
fi

echo "🔌 [DAGR] Auto-configuring MCP and Agent Skills across all IDEs..."
if command -v dagr >/dev/null 2>&1; then
    dagr mcp install --client all
    dagr skills install --target all
elif [ -f "$DEST" ]; then
    "$DEST" mcp install --client all
    "$DEST" skills install --target all
fi

echo ""
echo "✅ [DAGR] Installation successful! Restart Cursor / Claude Desktop to connect."
