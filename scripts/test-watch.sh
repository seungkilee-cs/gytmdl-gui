#!/bin/bash

# Watch Mode Testing Script for gytmdl-gui
# This script runs tests in watch mode for development

set -e

echo "[WATCH] Running Tests in Watch Mode"
echo "=============================="
echo "Press Ctrl+C to stop watching"

# Change to the Tauri backend directory
cd "$(dirname "$0")/../src-tauri"

# Install cargo-watch if not already installed
if ! command -v cargo-watch &> /dev/null; then
    echo "📦 Installing cargo-watch..."
    cargo install cargo-watch
fi

echo "[WATCH] Watching for changes and running tests..."
cargo watch -x test