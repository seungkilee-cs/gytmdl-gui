#!/bin/bash
# Pre-removal script for gytmdl-gui

set -e

echo "Preparing to remove gytmdl-gui..."

# Stop any running instances
if command -v pkill >/dev/null 2>&1; then
    pkill -f gytmdl-gui || true
fi

# Give processes time to terminate gracefully
sleep 2

echo "Pre-removal preparation completed."