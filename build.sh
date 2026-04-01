#!/usr/bin/env bash
set -euo pipefail

# Build frontend
cd frontend
npm i
npm run build
cd ..

# Build Rust
if [ $# -eq 0 ]; then
  cargo build --release
else
  cargo zigbuild --release --target "$1"
fi

cp -f -v target/release/notebooklm-cli ./
