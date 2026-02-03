#!/bin/bash
set -e

cd "$(dirname "$0")"

echo "Building project..."
trunk build --release

echo "Serving on http://localhost:8000"
cd dist
python3 -m http.server 8000
