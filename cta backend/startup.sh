#!/bin/bash

LOG="/var/log/cta-startup.log"
exec > >(tee -a "$LOG") 2>&1
echo "========== CTA startup $(date) =========="
set -ex

BINARY_URL="https://storage.googleapis.com/compiled_rust_for_text_adventure_server_to_run/cta-backend"
APP_DIR="/opt/cta-backend"

# ----- Detect public IP and build nip.io domain -----
PUBLIC_IP=$(curl -s http://metadata.google.internal/computeMetadata/v1/instance/network-interfaces/0/access-configs/0/external-ip -H "Metadata-Flavor: Google")
DOMAIN="${PUBLIC_IP}.nip.io"
echo "Public IP: $PUBLIC_IP"
echo "Domain: $DOMAIN"

# ----- Install Caddy (first time only) -----
if ! command -v caddy &> /dev/null; then
    echo "Waiting for apt lock..."
    while fuser /var/lib/dpkg/lock-frontend >/dev/null 2>&1; do
        echo "  apt is locked, waiting 5s..."
        sleep 5
    done

    echo "Installing Caddy..."
    apt-get update
    apt-get install -y debian-keyring debian-archive-keyring apt-transport-https curl
    curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
    curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' > /etc/apt/sources.list.d/caddy-stable.list
    apt-get update
    apt-get install -y caddy
    echo "Caddy installed"
else
    echo "Caddy already installed"
fi

# ----- Configure Caddy -----
echo "Writing Caddyfile..."
cat > /etc/caddy/Caddyfile <<EOF
$DOMAIN {
    reverse_proxy localhost:8080
}
EOF
cat /etc/caddy/Caddyfile

# ----- Open firewall ports -----
if command -v ufw &> /dev/null; then
    ufw allow 80/tcp
    ufw allow 443/tcp
fi

# ----- Download backend binary (always, in case it was updated) -----
echo "Downloading backend binary..."
mkdir -p "$APP_DIR"
systemctl stop cta-backend 2>/dev/null || true
curl -L -o "$APP_DIR/cta-backend" "$BINARY_URL"
chmod +x "$APP_DIR/cta-backend"
ls -la "$APP_DIR/cta-backend"

# ----- Create systemd service -----
cat > /etc/systemd/system/cta-backend.service <<EOF
[Unit]
Description=CTA Backend
After=network.target

[Service]
ExecStart=$APP_DIR/cta-backend
WorkingDirectory=$APP_DIR
Restart=always
Environment=RUST_LOG=info
Environment=HOST=127.0.0.1
Environment=PORT=8080

[Install]
WantedBy=multi-user.target
EOF

# ----- Start Services -----
systemctl daemon-reload
systemctl enable --now cta-backend
systemctl restart caddy

echo "Waiting for backend to start..."
sleep 2
curl -s http://localhost:8080/health || echo "Backend not responding yet"

echo "========== DONE =========="
echo "Backend running at https://$DOMAIN"
