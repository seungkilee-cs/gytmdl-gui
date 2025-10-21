#!/bin/bash
# Post-removal script for gytmdl-gui

set -e

echo "Cleaning up after gytmdl-gui removal..."

if [ "$EUID" -eq 0 ]; then
    echo "Performing system-wide cleanup..."
    
    # Remove desktop entry
    rm -f /usr/share/applications/gytmdl-gui.desktop
    
    # Remove application directory if empty
    rmdir /opt/gytmdl-gui 2>/dev/null || true
    
    # Update desktop database
    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database /usr/share/applications 2>/dev/null || true
    fi
    
    # Update icon cache
    if command -v gtk-update-icon-cache >/dev/null 2>&1; then
        gtk-update-icon-cache -f -t /usr/share/icons/hicolor 2>/dev/null || true
    fi
    
    # Remove system config template (but preserve user configs)
    rm -rf /etc/gytmdl-gui 2>/dev/null || true
    
    echo "System-wide cleanup completed."
    
else
    echo "Performing user cleanup..."
    
    # Remove user desktop entry
    rm -f "$HOME/.local/share/applications/gytmdl-gui.desktop"
    
    # Update user desktop database
    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
    fi
    
    echo "User cleanup completed."
fi

echo "gytmdl-gui has been successfully removed."
echo ""
echo "Note: User configuration and download files have been preserved."
echo "To completely remove all data, manually delete:"
if [ "$EUID" -eq 0 ]; then
    echo "- User config directories: ~/.config/gytmdl-gui"
    echo "- User data directories: ~/.local/share/gytmdl-gui"
else
    echo "- Config directory: $HOME/.config/gytmdl-gui"
    echo "- Data directory: $HOME/.local/share/gytmdl-gui"
fi