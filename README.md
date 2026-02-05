# Collaborative Text Adventure

A web-based collaborative text adventure game built entirely in Rust. Players explore a shared world where their choices affect the story for everyone.

## Architecture

```
┌─────────────────────┐         ┌─────────────────────┐
│   Frontend (WASM)   │  HTTP   │   Backend (Actix)   │
│   Leptos + Trunk    │ ◄─────► │   Rust on GCP VM    │
└─────────────────────┘         └─────────────────────┘
```

- **Frontend**: Leptos framework compiled to WebAssembly, runs in the browser
- **Backend**: Actix-web server running on a Google Cloud VM

## Project Structure

```
├── cta frontend/          # Leptos WASM frontend
│   ├── src/
│   │   ├── components/    # UI components
│   │   ├── pages/         # Page views
│   │   ├── domain/        # Game domain logic
│   │   ├── state/         # State management
│   │   └── api.rs         # Backend API client
│   └── Cargo.toml
├── cta backend/           # Actix-web backend
│   ├── src/main.rs
│   ├── startup.sh         # GCP VM startup script
│   └── Cargo.toml
└── build-and-upload.sh    # Deploy backend to VM
```

## Frontend Development

### Prerequisites

```bash
# Install Rust nightly
rustup toolchain install nightly --allow-downgrade

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install Trunk (build tool)
cargo install trunk
```

### Run locally

```bash
cd "cta frontend"
trunk serve --port 3000 --open
```

### Build for production

```bash
cd "cta frontend"
trunk build --release
# Output in dist/ - deploy to any static host
```

## Backend Deployment (GCP)

### Overview

The backend runs on a Google Compute Engine VM. There are two deployment methods:

1. **GCS Bucket method**: Build locally, upload binary to a public GCS bucket, VM pulls it on startup
2. **Direct SCP method**: Build locally, upload directly to VM via SSH

### Method 1: GCS Bucket

This method uses a startup script that automatically downloads and runs the binary when the VM boots.

#### Step 1: Build the backend

```bash
cd "cta backend"
cargo build --release
# Binary at: target/release/cta-backend
```

#### Step 2: Upload to GCS

```bash
# Create a bucket (one time)
gsutil mb gs://your-bucket-name

# Make it publicly readable (one time)
gsutil iam ch allUsers:objectViewer gs://your-bucket-name

# Upload the binary
gsutil cp target/release/cta-backend gs://your-bucket-name/cta-backend
```

#### Step 3: Update the startup script

Edit `cta backend/startup.sh` and set your bucket URL:

```bash
BINARY_URL="https://storage.googleapis.com/your-bucket-name/cta-backend"
```

#### Step 4: Create the GCP VM

1. Go to GCP Console → Compute Engine → VM Instances → Create Instance
2. Choose a machine type (e2-micro works for small loads)
3. Under "Firewall", check "Allow HTTP traffic"
4. Expand "Advanced options" → "Management"
5. Paste the contents of `startup.sh` into the "Startup script" field
6. Create the instance

The VM will automatically download and start the backend on boot.

### Method 2: Direct SCP (Quick iteration)

For faster iteration during development:

```bash
./build-and-upload.sh <VM_IP>
```

This builds locally and uploads directly to the VM via SSH.

### SSH into the VM

```bash
gcloud compute ssh <instance-name> --zone <zone>
# or
ssh <username>@<vm-ip>
```

### Check backend status

```bash
# Service status
sudo systemctl status cta-backend

# Live logs
sudo journalctl -u cta-backend -f

# Test locally on VM
curl http://localhost:8080
curl http://localhost:8080/health
```

### How the startup script works

The `startup.sh` script:

1. Checks if the service is already running (skips setup if so)
2. Downloads the binary from GCS
3. Sets up iptables to redirect port 80 → 8080 (so the app doesn't need root)
4. Creates a systemd service
5. Starts the service

The app runs on port 8080, but external requests to port 80 are automatically redirected.

### Updating the backend

After making changes:

1. Rebuild: `cargo build --release`
2. Upload new binary to GCS: `gsutil cp target/release/cta-backend gs://your-bucket-name/cta-backend`
3. On the VM, restart the service:
   ```bash
   sudo curl -L -o /opt/cta-backend/cta-backend "https://storage.googleapis.com/your-bucket-name/cta-backend"
   sudo chmod +x /opt/cta-backend/cta-backend
   sudo systemctl restart cta-backend
   ```

Or just use the quick method:
```bash
./build-and-upload.sh <VM_IP>
```

## HTTPS Setup

For HTTPS, you need a domain name. The easiest approach is Caddy:

```bash
# On the VM
sudo apt install -y caddy

# Configure (replace yourdomain.com)
echo 'yourdomain.com {
    reverse_proxy localhost:8080
}' | sudo tee /etc/caddy/Caddyfile

sudo systemctl restart caddy
```

Caddy automatically obtains and renews SSL certificates from Let's Encrypt.

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | GET | Welcome message |
| `/health` | GET | Health check |
| `/api/counter` | GET | Get current counter value |
| `/api/counter/increment` | POST | Increment counter |

## Development

### Environment variables

The backend reads these environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `HOST` | `0.0.0.0` | Bind address |
| `PORT` | `8080` | Listen port |
| `RUST_LOG` | `info` | Log level |
