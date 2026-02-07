# Collaborative Text Adventure

A branching text adventure where players read, choose paths, and write new story branches. Rust frontend (Leptos/WASM) + Rust backend (Actix-web).

## Project Structure

```
cta frontend/           Leptos WASM frontend (compiles to static HTML/JS/WASM)
  src/config.rs         API_BASE configuration
cta backend/            Actix-web backend (persists to adventure.db via SQLite)
  startup.sh            GCP VM startup script
shared/                 Types shared between frontend and backend
run.sh                  Run both locally
build_backend.sh        Build backend binary to project root for GCS upload
build_for_newgrounds.sh Build frontend zip for Newgrounds upload
```

## Prerequisites

```bash
rustup toolchain install nightly --allow-downgrade
rustup target add wasm32-unknown-unknown
cargo install trunk
```

## Run Locally

```bash
./run.sh
# Frontend: http://localhost:8000
# Backend:  http://localhost:8080
```

Or with hot-reload: `cd "cta frontend" && trunk serve --port 3000 --open`

## Deploy Backend to GCP VM

### First time

1. Create a public GCS bucket. Update `BINARY_URL` in `cta backend/startup.sh` to point to it.

2. Build and upload the backend binary:
   ```bash
   ./build_backend.sh
   ```
   Upload the `cta-backend` binary from the project root to the bucket.

3. Create a Compute Engine VM:
   - Machine type: e2-micro
   - Firewall: check **Allow HTTP** and **Allow HTTPS**
   - Advanced options → Management → paste contents of `startup.sh`

   Live at `https://<VM_IP>.nip.io` within a couple minutes (Caddy handles HTTPS automatically).

### Update the backend

```bash
./build_backend.sh
```

Upload `cta-backend` to the GCS bucket, then restart the VM (or `sudo systemctl restart cta-backend` on it).

### Debug on the VM

```bash
gcloud compute ssh <instance-name> --zone <zone>
sudo systemctl status cta-backend
sudo journalctl -u cta-backend -f
curl http://localhost:8080/health
```

## Deploy Frontend to Newgrounds

```bash
./build_for_newgrounds.sh <VM_IP>
```

Upload the resulting `cta-game.zip` to Newgrounds as an HTML5 game.

If the VM IP changes, rebuild and re-upload. A real domain instead of nip.io avoids this.

## Deploy Frontend Anywhere Else

The frontend is just static files. Build and host wherever:

```bash
cd "cta frontend"
API_BASE=https://<VM_IP>.nip.io trunk build --release
# dist/ contains everything - upload to any static host
```

Works with GitHub Pages, Netlify, Vercel, S3, or just `python3 -m http.server` in `dist/`.

## Environment Variables

| Variable | Where | Default | Description |
|----------|-------|---------|-------------|
| `API_BASE` | Frontend (compile-time) | `http://localhost:8080` | Backend URL for production builds |
| `HOST` | Backend (runtime) | `0.0.0.0` | Bind address |
| `PORT` | Backend (runtime) | `8080` | Listen port |
| `RUST_LOG` | Backend (runtime) | `info` | Log level |

## API

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Health check |
| `/api` | POST | Bincode-encoded `ServerMessage` (adventure nodes, descendant counts) |
| `/api/counter` | GET | Get counter value |
| `/api/counter/increment` | POST | Increment counter |

## Newgrounds Integration

Each submitted node carries the user's `ngio_session_id` from the URL query string. The backend verifies it server-side against the Newgrounds gateway API and stores the verified username in `created_by`.
