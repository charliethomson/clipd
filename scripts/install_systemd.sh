#!/usr/bin/env bash
set -euo pipefail

SERVICE_NAME="${SERVICE_NAME:-clipd}"
BINARY_DEST="${BINARY_DEST:-/usr/local/bin/clipd}"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TEMPLATE="$REPO_ROOT/configs/systemd/clipd.service"
SYSTEMD_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"
SERVICE_FILE="$SYSTEMD_DIR/$SERVICE_NAME.service"

# --- build ---
echo "Building release binary..."
cargo build --release --manifest-path "$REPO_ROOT/Cargo.toml"

# --- install binary ---
echo "Installing binary to $BINARY_DEST..."
install -m 755 "$REPO_ROOT/target/release/clipd" "$BINARY_DEST"

# --- install service ---
mkdir -p "$SYSTEMD_DIR"
sed \
    -e "s|{{BINARY_PATH}}|$BINARY_DEST|g" \
    -e "s|{{SERVICE_NAME}}|$SERVICE_NAME|g" \
    "$TEMPLATE" > "$SERVICE_FILE"
echo "Installed service to $SERVICE_FILE"

# --- enable and start ---
systemctl --user daemon-reload
systemctl --user enable --now "$SERVICE_NAME"
echo "Service enabled and started."
echo "  Status:  systemctl --user status $SERVICE_NAME"
echo "  Logs:    journalctl --user -u $SERVICE_NAME -f"
