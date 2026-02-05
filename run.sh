#!/bin/bash
set -e

cd "$(dirname "$0")"

# Kill all child processes when script exits
trap 'kill 0' EXIT

# Build frontend
echo "Building frontend..."
cd "cta frontend"
trunk build --release
cd ..

# Build backend
echo "Building backend..."
cd "cta backend"
cargo build --release
cd ..

echo ""
echo "==================================="
echo "Frontend: http://localhost:8000"
echo "Backend:  http://localhost:8080"
echo "==================================="
echo "Press Ctrl+C to stop both servers"
echo ""

# Run both servers in background
(cd "cta frontend/dist" && python3 -m http.server 8000) &
(cd "cta backend" && cargo run --release) &

# Wait for any process to exit (or Ctrl+C)
wait
