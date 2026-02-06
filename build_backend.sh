#!/bin/bash
set -e

cd "$(dirname "$0")/cta backend"
cargo build --release
cp target/release/cta-backend ../
echo "Done! cta-backend binary is in the project root."
