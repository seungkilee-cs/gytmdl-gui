#!/bin/bash

# Backend Testing Script for gytmdl-gui
# This script runs all Rust backend tests

set -e

echo "[TEST-BACKEND] Running gytmdl-gui Backend Tests"
echo "=================================="

# Change to the Tauri backend directory
cd "$(dirname "$0")/../src-tauri"

echo "[RUST] Building backend..."
cargo build

echo "[RUST] Running all tests..."
cargo test

echo "[RUST] Running tests with coverage info..."
cargo test -- --nocapture

echo "All tests completed successfully"