#!/bin/bash

# Data Models Testing Script for gytmdl-gui
# This script specifically runs tests for the data models (state management)

set -e

echo "[TEST-BACKEND-DATA] Running Data Models Tests"
echo "============================"

# Change to the Tauri backend directory
cd "$(dirname "$0")/../src-tauri"

echo "[RUST] Running state management tests..."
cargo test modules::state::tests

echo "[RUST] Running config manager tests..."
cargo test modules::config_manager::tests

echo "[RUST] Running data model tests with detailed output..."
cargo test modules::state::tests modules::config_manager::tests -- --nocapture

echo "All tests completed successfully"