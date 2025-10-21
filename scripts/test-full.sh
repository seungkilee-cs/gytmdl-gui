#!/bin/bash

# Full Test Suite Script for gytmdl-gui
# This script runs both frontend and backend tests

set -e

echo "[TEST-FULL] Running Full Test Suite"
echo "=========================="

# Run backend tests
echo "[RUST] Running Rust backend tests..."
cd "$(dirname "$0")/../src-tauri"
cargo test

# Run frontend tests (no tests yet)
echo "[REACT]  Checking for frontend tests..."
cd "$(dirname "$0")/.."

if [ -f "package.json" ] && grep -q '"test"' package.json; then
    echo "Running frontend tests..."
    npm test
else
    echo "No frontend tests configured yet."
fi

echo "All tests completed successfully"