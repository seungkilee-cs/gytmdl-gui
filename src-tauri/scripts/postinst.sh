#!/bin/bash
# Post-installation script for gytmdl-gui

set -e

APP_NAME="gytmdl-gui"
DESKTOP_FILE_CONTENT='[Desktop Entry]
Version=1.0
Type=Application
Name=gytmdl GUI
Comment=YouTube Music downloader with GUI
Exec=/opt/gytmdl-gui/gytmdl-gui
Icon=gytmdl-gui
Terminal=false
Categories=AudioVideo;Audio;
MimeType=x-scheme-handler/https;x-scheme-handler/http;
Keywords=youtube;music;download;audio;
StartupNotify=true'

if [ "$EUID" -eq 0 ]; then
    echo "Completing system-wide installation..."
    
    # Create desktop entry
    echo "$DESKTOP_FILE_CONTENT" > /usr/share/applications/gytmdl-gui.desktop
    chmod 644 /usr/share/applications/gytmdl-gui.desktop
    
    # Set proper permissions for the application
    chmod +x /opt/gytmdl-gui/gytmdl-gui
    chmod +x /opt/gytmdl-gui/sidecars/* 2>/dev/null || true
    
    # Update desktop database
    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database /usr/share/applications
    fi
    
    # Update icon cache
    if command -v gtk-update-icon-cache >/dev/null 2>&1; then
        gtk-update-icon-cache -f -t /usr/share/icons/hicolor 2>/dev/null || true
    fi
    
    echo "System-wide installation completed."
    echo "You can now launch gytmdl-gui from your application menu or by running: gytmdl-gui"
    
else
    echo "Completing user installation..."
    
    # Create desktop entry for user
    DESKTOP_FILE_CONTENT_USER='[Desktop Entry]
Version=1.0
Type=Application
Name=gytmdl GUI
Comment=YouTube Music downloader with GUI
Exec='$HOME'/.local/bin/gytmdl-gui/gytmdl-gui
Icon=gytmdl-gui
Terminal=false
Categories=AudioVideo;Audio;
MimeType=x-scheme-handler/https;x-scheme-handler/http;
Keywords=youtube;music;download;audio;
StartupNotify=true'
    
    echo "$DESKTOP_FILE_CONTENT_USER" > "$HOME/.local/share/applications/gytmdl-gui.desktop"
    chmod 644 "$HOME/.local/share/applications/gytmdl-gui.desktop"
    
    # Update user desktop database
    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
    fi
    
    echo "User installation completed."
    echo "You can now launch gytmdl-gui from your application menu."
fi

# Create default configuration directory
if [ "$EUID" -eq 0 ]; then
    # System-wide config template
    mkdir -p /etc/gytmdl-gui
    echo "# gytmdl-gui system configuration" > /etc/gytmdl-gui/config.json.template
else
    # User config directory
    mkdir -p "$HOME/.config/gytmdl-gui"
fi

echo "Installation completed successfully!"
echo ""
echo "To get started:"
echo "1. Launch gytmdl-gui from your application menu"
echo "2. Configure your download preferences in Settings"
echo "3. Import YouTube Music cookies if needed for premium content"
echo ""
echo "For support and documentation, visit: https://github.com/your-repo/gytmdl-gui"