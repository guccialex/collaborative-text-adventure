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
│   │   ├── components/    # UI components (game/, server_counter)
│   │   ├── pages/         # Page views
│   │   ├── domain/        # Game domain logic
│   │   ├── state/         # State management
│   │   ├── api/           # Backend API client modules
│   │   └── config.rs      # Centralized configuration
│   └── Cargo.toml
├── cta backend/           # Actix-web backend
│   ├── src/main.rs
│   ├── startup.sh         # GCP VM startup script (installs Caddy, configures HTTPS)
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

### Prerequisites

- A GCP project with Compute Engine enabled
- `gcloud` CLI installed and authenticated
- A GCS bucket with the compiled backend binary (see "First-time GCS setup" below)

### First-time GCS setup

```bash
# Create a bucket (one time)
gsutil mb gs://your-bucket-name

# Make it publicly readable (one time)
gsutil iam ch allUsers:objectViewer gs://your-bucket-name
```

Then update the `BINARY_URL` in `cta backend/startup.sh` to point to your bucket.

### Build and upload the backend binary

```bash
cd "cta backend"
cargo build --release
gsutil cp target/release/cta-backend gs://your-bucket-name/cta-backend
```

### Create the GCP VM

1. Go to GCP Console → Compute Engine → VM Instances → Create Instance
2. Choose a machine type (e2-micro works for small loads)
3. Under "Firewall", check **both** "Allow HTTP traffic" and "Allow HTTPS traffic"
4. Expand "Advanced options" → "Management"
5. Paste the contents of `cta backend/startup.sh` into the "Startup script" field
6. Create the instance

The startup script automatically:
- Detects the VM's public IP
- Installs Caddy (reverse proxy with automatic HTTPS)
- Configures a `<PUBLIC_IP>.nip.io` domain (no domain purchase needed)
- Downloads the backend binary from GCS
- Creates a systemd service for the backend
- Starts everything up with HTTPS via Let's Encrypt

Your app will be live at `https://<VM_IP>.nip.io` within a minute or two.

### Updating the backend (quick iteration)

```bash
./build-and-upload.sh <VM_IP>
```

This builds locally and SCPs the binary directly to the VM, then restarts the service.

### SSH into the VM

```bash
gcloud compute ssh <instance-name> --zone <zone>
```

### Check status

```bash
# Backend service
sudo systemctl status cta-backend
sudo journalctl -u cta-backend -f

# Caddy (HTTPS reverse proxy)
sudo systemctl status caddy
sudo journalctl -u caddy --no-pager -n 50

# Test locally on VM
curl http://localhost:8080/health
```

### How it works

```
Internet                    GCP VM
────────────────────────────────────────────
https://<IP>.nip.io ──► Caddy (:443)
                            │ reverse_proxy
                            ▼
                          cta-backend (:8080)
```

- **nip.io** provides free wildcard DNS: `1.2.3.4.nip.io` resolves to `1.2.3.4`
- **Caddy** handles TLS certificates automatically via Let's Encrypt
- **Backend** only listens on localhost:8080, Caddy proxies external traffic to it

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

## Future: Newgrounds API Integration

Adventure nodes could store the Newgrounds user ID of their creator, attributing each contribution to the player who wrote it. This would require integrating with the Newgrounds.io API.

Newgrounds.io is a REST API at `https://newgrounds.io/gateway/v3` — you POST JSON with a component name and parameters, and get JSON back. The JS libraries are just wrappers around this. Since the frontend is Rust/WASM, there are two ways to integrate:

1. **From the frontend via `wasm-bindgen`**: Load a small JS library (e.g. [KilledByAPixel/newgrounds](https://github.com/KilledByAPixel/newgrounds)) in `index.html` and call it from Rust through `#[wasm_bindgen]` extern bindings. The JS library handles the login popup and session encryption. The frontend would authenticate the user, get their session/user info, and include their Newgrounds user ID when submitting an adventure node.

2. **From the backend via direct REST calls**: The backend calls `https://newgrounds.io/gateway/v3` directly using `reqwest` or similar. The frontend passes a Newgrounds session token to the backend, which validates it server-side before accepting a node submission. This is more secure since the backend can verify the user identity rather than trusting the frontend.

Either way, the `AdventureNode` struct in the shared crate would gain a `creator_id: Option<String>` field, and the backend would store who created each node. This also opens the door to Newgrounds medals (e.g. "first contribution") and scoreboards (e.g. most contributions).
