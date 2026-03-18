#!/usr/bin/env bash
set -euo pipefail

REPO="charliethomson/clipd"
SERVICE_NAME="${SERVICE_NAME:-clipd}"
BINARY_DEST="${BINARY_DEST:-/usr/local/bin/clipd}"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TEMPLATE="$REPO_ROOT/configs/systemd/clipd.service"
SYSTEMD_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"
SERVICE_FILE="$SYSTEMD_DIR/$SERVICE_NAME.service"

case "$(uname -m)" in
    x86_64) ASSET="clipd-x86_64-unknown-linux-gnu" ;;
    *) echo "Unsupported architecture: $(uname -m)"; exit 1 ;;
esac

# --- download latest release ---
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
install -m 755 "$TMP" "$BINARY_DEST"
rm "$TMP"
echo "Installed binary to $BINARY_DEST"

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
