#!/bin/bash
# Pre-installation script for gytmdl-gui

set -e

# Check if running as root for system-wide installation
if [ "$EUID" -eq 0 ]; then
    echo "Installing gytmdl-gui system-wide..."
    
    # Create application directory if it doesn't exist
    mkdir -p /opt/gytmdl-gui
    
    # Create symlink directory for desktop integration
    mkdir -p /usr/share/applications
    mkdir -p /usr/share/icons/hicolor/128x128/apps
    mkdir -p /usr/share/icons/hicolor/256x256/apps
else
    echo "Installing gytmdl-gui for current user..."
    
    # Create user application directory
    mkdir -p "$HOME/.local/share/applications"
    mkdir -p "$HOME/.local/share/icons/hicolor/128x128/apps"
    mkdir -p "$HOME/.local/share/icons/hicolor/256x256/apps"
fi

# Check for required dependencies
echo "Checking system dependencies..."

# Check for basic libraries that might be needed
if ! ldconfig -p | grep -q libssl; then
    echo "Warning: libssl not found. Some features may not work."
fi

if ! ldconfig -p | grep -q libcrypto; then
    echo "Warning: libcrypto not found. Some features may not work."
fi

echo "Pre-installation checks completed."