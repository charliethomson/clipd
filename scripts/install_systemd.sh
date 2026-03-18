#!/usr/bin/env bash
set -euo pipefail

SERVICE_NAME="${SERVICE_NAME:-clipd}"
BINARY_DEST="${BINARY_DEST:-/usr/local/bin/clipd}"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TEMPLATE="$REPO_ROOT/configs/systemd/clipd.service"
SYSTEMD_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"
SERVICE_FILE="$SYSTEMD_DIR/$SERVICE_NAME.service"

if [ -f "$BINARY_DEST" ]; then
    echo "Updating clipd..."
else
    echo "Installing clipd..."
fi

# --- stop existing service before replacing binary ---
if systemctl --user is-active --quiet "$SERVICE_NAME" 2>/dev/null; then
    echo "Stopping existing service..."
    systemctl --user stop "$SERVICE_NAME"
fi

# --- build ---
echo "Building release binary..."
cargo build --release --manifest-path "$REPO_ROOT/Cargo.toml"

# --- install binary ---
install -m 755 "$REPO_ROOT/target/release/clipd" "$BINARY_DEST"
echo "Installed binary to $BINARY_DEST"

# --- install service ---
mkdir -p "$SYSTEMD_DIR"
sed \
    -e "s|{{BINARY_PATH}}|$BINARY_DEST|g" \
    -e "s|{{SERVICE_NAME}}|$SERVICE_NAME|g" \
    "$TEMPLATE" > "$SERVICE_FILE"

# --- enable and start ---
systemctl --user daemon-reload
systemctl --user enable --now "$SERVICE_NAME"
echo "Done."
echo "  Status:  systemctl --user status $SERVICE_NAME"
echo "  Logs:    journalctl --user -u $SERVICE_NAME -f"
