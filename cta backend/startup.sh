#!/bin/bash
set -e

REPO_URL="https://github.com/YOUR_USERNAME/collaborative-text-adventure.git"
APP_DIR="/opt/cta-backend"
RUST_USER="cta"

# Create user if doesn't exist
if ! id "$RUST_USER" &>/dev/null; then
    useradd -m -s /bin/bash "$RUST_USER"
fi

# Install dependencies
apt-get update
apt-get install -y build-essential pkg-config libssl-dev git

# Install Rust for the user
if [ ! -d "/home/$RUST_USER/.cargo" ]; then
    su - "$RUST_USER" -c 'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y'
fi

# Clone or pull repo
if [ -d "$APP_DIR" ]; then
    cd "$APP_DIR"
    git pull
else
    git clone "$REPO_URL" "$APP_DIR"
fi

chown -R "$RUST_USER:$RUST_USER" "$APP_DIR"

# Build
cd "$APP_DIR/cta backend"
su - "$RUST_USER" -c "cd '$APP_DIR/cta backend' && ~/.cargo/bin/cargo build --release"

# Create .env if missing
if [ ! -f ".env" ]; then
    cp .env.example .env
fi

# Create systemd service
cat > /etc/systemd/system/cta-backend.service <<EOF
[Unit]
Description=CTA Backend
After=network.target

[Service]
Type=simple
User=$RUST_USER
WorkingDirectory=$APP_DIR/cta backend
ExecStart=$APP_DIR/cta backend/target/release/cta-backend
Restart=always
RestartSec=5
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
EOF

# Start service
systemctl daemon-reload
systemctl enable cta-backend
systemctl restart cta-backend

echo "CTA Backend is running!"
