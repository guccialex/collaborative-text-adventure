#!/bin/bash
set -e

if [ -z "$1" ]; then
    echo "Usage: ./build_for_newgrounds.sh <VM_IP>"
    echo "Example: ./build_for_newgrounds.sh 34.123.45.67"
    exit 1
fi

cd "$(dirname "$0")/cta frontend"
API_BASE="https://$1.nip.io" trunk build --release --public-url ./
cd dist && zip -r ../../cta-game.zip .
echo "Done! Upload cta-game.zip to Newgrounds."
