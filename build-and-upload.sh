#!/bin/bash
set -e

if [ -z "$1" ]; then
    echo "Usage: ./build-and-upload.sh <VM_IP>"
    echo "Example: ./build-and-upload.sh 34.123.45.67"
    exit 1
fi

VM_IP="$1"
VM_USER="popplepoggle"

echo "Building backend..."
cd "$(dirname "$0")/cta backend"
cargo build --release

echo "Uploading to $VM_IP..."
scp "./target/release/cta-backend" "$VM_USER@$VM_IP:/tmp/"

echo "Installing on VM..."
ssh "$VM_USER@$VM_IP" 'sudo mkdir -p /opt/cta-backend && sudo mv /tmp/cta-backend /opt/cta-backend/ && sudo chmod +x /opt/cta-backend/cta-backend && sudo systemctl restart cta-backend'

echo "Done! Backend is running on $VM_IP:8080"
