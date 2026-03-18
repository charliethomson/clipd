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

# --- build ---
echo "Building release binary..."
cargo build --release --manifest-path "$REPO_ROOT/Cargo.toml"

# --- install binary ---
echo "Installing binary to $BINARY_DEST..."
install -m 755 "$REPO_ROOT/target/release/clipd" "$BINARY_DEST"

# --- install plist ---
mkdir -p "$AGENT_DIR"
sed \
    -e "s|{{LABEL}}|$LABEL|g" \
    -e "s|{{BINARY_PATH}}|$BINARY_DEST|g" \
    -e "s|{{LOG_PATH}}|$LOG_DIR/clipd.log|g" \
    -e "s|{{ERR_PATH}}|$LOG_DIR/clipd.err|g" \
    "$TEMPLATE" > "$PLIST"
echo "Installed plist to $PLIST"

# --- load agent ---
if launchctl list | grep -q "$LABEL" 2>/dev/null; then
    echo "Unloading existing agent..."
    launchctl unload "$PLIST"
fi
launchctl load "$PLIST"
echo "Agent loaded. Logs: $LOG_DIR/clipd.log / $LOG_DIR/clipd.err"
