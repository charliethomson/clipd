#!/usr/bin/env bash
set -euo pipefail

REPO="charliethomson/clipd"
LABEL="${LABEL:-dev.thmsn.clipd}"
BINARY_DEST="${BINARY_DEST:-$HOME/.local/bin/clipd}"
LOG_DIR="${LOG_DIR:-/tmp}"

AGENT_DIR="$HOME/Library/LaunchAgents"
PLIST="$AGENT_DIR/$LABEL.plist"
RAW="https://raw.githubusercontent.com/$REPO/main"

case "$(uname -m)" in
    arm64)  ASSET="clipd-aarch64-apple-darwin" ;;
    x86_64) ASSET="clipd-x86_64-apple-darwin" ;;
    *) echo "Unsupported architecture: $(uname -m)"; exit 1 ;;
esac

# --- download latest release binary ---
echo "Fetching latest release ($ASSET)..."
DOWNLOAD_URL=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
    | grep -o '"browser_download_url": *"[^"]*'"$ASSET"'"' \
    | grep -o 'https://[^"]*')

if [ -z "$DOWNLOAD_URL" ]; then
    echo "Error: could not find release asset $ASSET"
    exit 1
fi

TMP=$(mktemp)
curl -fsSL -o "$TMP" "$DOWNLOAD_URL"
mkdir -p "$(dirname "$BINARY_DEST")"
install -m 755 "$TMP" "$BINARY_DEST"
rm "$TMP"
echo "Installed binary to $BINARY_DEST"

case ":$PATH:" in
    *":$(dirname "$BINARY_DEST"):"*) ;;
    *) echo "Note: $(dirname "$BINARY_DEST") is not in your PATH. Add the following to your shell profile:"; echo "  export PATH=\"$(dirname "$BINARY_DEST"):\$PATH\"" ;;
esac

# --- install plist ---
mkdir -p "$AGENT_DIR"
curl -fsSL "$RAW/configs/launchd/dev.thmsn.clipd.plist" \
    | sed \
        -e "s|{{LABEL}}|$LABEL|g" \
        -e "s|{{BINARY_PATH}}|$BINARY_DEST|g" \
        -e "s|{{LOG_PATH}}|$LOG_DIR/clipd.log|g" \
        -e "s|{{ERR_PATH}}|$LOG_DIR/clipd.err|g" \
    > "$PLIST"
echo "Installed plist to $PLIST"

# --- load agent ---
if launchctl list | grep -q "$LABEL" 2>/dev/null; then
    echo "Unloading existing agent..."
    launchctl unload "$PLIST"
fi
launchctl load "$PLIST"
echo "Agent loaded. Logs: $LOG_DIR/clipd.log / $LOG_DIR/clipd.err"
