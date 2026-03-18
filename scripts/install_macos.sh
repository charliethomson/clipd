#!/usr/bin/env bash
set -euo pipefail

LABEL="${LABEL:-dev.thmsn.clipd}"
BINARY_DEST="${BINARY_DEST:-/usr/local/bin/clipd}"
LOG_DIR="${LOG_DIR:-/tmp}"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TEMPLATE="$REPO_ROOT/configs/launchd/dev.thmsn.clipd.plist"
AGENT_DIR="$HOME/Library/LaunchAgents"
PLIST="$AGENT_DIR/$LABEL.plist"

if [ -f "$BINARY_DEST" ]; then
    echo "Updating clipd..."
else
    echo "Installing clipd..."
fi

# --- stop existing agent before replacing binary ---
if launchctl list | grep -q "$LABEL" 2>/dev/null; then
    echo "Stopping existing agent..."
    launchctl bootout "gui/$(id -u)/$LABEL" 2>/dev/null || true
fi

# --- build ---
echo "Building release binary..."
cargo build --release --manifest-path "$REPO_ROOT/Cargo.toml"

# --- install binary ---
install -m 755 "$REPO_ROOT/target/release/clipd" "$BINARY_DEST"
echo "Installed binary to $BINARY_DEST"

# --- install plist ---
mkdir -p "$AGENT_DIR"
sed \
    -e "s|{{LABEL}}|$LABEL|g" \
    -e "s|{{BINARY_PATH}}|$BINARY_DEST|g" \
    -e "s|{{LOG_PATH}}|$LOG_DIR/clipd.log|g" \
    -e "s|{{ERR_PATH}}|$LOG_DIR/clipd.err|g" \
    "$TEMPLATE" > "$PLIST"

# --- start agent ---
launchctl bootstrap "gui/$(id -u)" "$PLIST"
echo "Done. Logs: $LOG_DIR/clipd.log / $LOG_DIR/clipd.err"
