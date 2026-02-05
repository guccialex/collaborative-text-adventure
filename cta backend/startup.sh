#!/bin/bash
set -e

# ============================================
# CONFIGURE THIS: Set your domain name
# ============================================
DOMAIN="yourdomain.com"

BINARY_URL="https://storage.googleapis.com/compiled_rust_for_text_adventure_server_to_run/cta-backend"
APP_DIR="/opt/cta-backend"

# Skip if already running
systemctl is-active --quiet cta-backend && systemctl is-active --quiet caddy && exit 0

# ----- Install Caddy -----
if ! command -v caddy &> /dev/null; then
    apt-get update
    apt-get install -y debian-keyring debian-archive-keyring apt-transport-https curl
    curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
    curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' > /etc/apt/sources.list.d/caddy-stable.list
    apt-get update
    apt-get install -y caddy
fi

# ----- Configure Caddy -----
cat > /etc/caddy/Caddyfile <<EOF
$DOMAIN {
    reverse_proxy localhost:8080
}
EOF

# ----- Setup Backend -----
mkdir -p "$APP_DIR"
curl -L -o "$APP_DIR/cta-backend" "$BINARY_URL"
chmod +x "$APP_DIR/cta-backend"

cat > /etc/systemd/system/cta-backend.service <<EOF
[Unit]
Description=CTA Backend
After=network.target

[Service]
ExecStart=$APP_DIR/cta-backend
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
