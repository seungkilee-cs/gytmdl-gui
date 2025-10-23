#!/bin/bash
# Quick test script for gytmdl-gui
# This script helps you quickly test the application locally

set -e

echo "🚀 gytmdl-gui Quick Test Script"
echo "================================"

# Get the project root directory
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
echo "Project root: $PROJECT_ROOT"

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check dependencies
echo ""
echo "🔍 Checking dependencies..."

MISSING_DEPS=()

if ! command_exists node; then
    MISSING_DEPS+=("node")
fi

if ! command_exists npm; then
    MISSING_DEPS+=("npm")
fi

if ! command_exists cargo; then
    MISSING_DEPS+=("cargo")
fi

if ! command_exists python3; then
    MISSING_DEPS+=("python3")
fi

if [ ${#MISSING_DEPS[@]} -ne 0 ]; then
    echo "❌ Missing dependencies: ${MISSING_DEPS[*]}"
    echo "Please install the missing dependencies and try again."
    exit 1
fi

echo "✅ All dependencies found"

# Check PyInstaller
echo ""
echo "🔍 Checking PyInstaller..."
if python3 -c "import PyInstaller" 2>/dev/null; then
    echo "✅ PyInstaller is available"
else
    echo "⚠️ PyInstaller not found. Installing..."
    pip3 install pyinstaller
fi

# Test isolation system
echo ""
echo "🔒 Testing isolation system..."
python3 "$PROJECT_ROOT/scripts/test-isolation.py"

# Build sidecar binary for current platform
echo ""
echo "🔨 Building sidecar binary..."
if [ -f "$PROJECT_ROOT/scripts/build-simple-sidecar.py" ]; then
    python3 "$PROJECT_ROOT/scripts/build-simple-sidecar.py"
else
    echo "⚠️ Simple build script not found, trying production build..."
    if [ -f "$PROJECT_ROOT/scripts/build-production-bundle.py" ]; then
        python3 "$PROJECT_ROOT/scripts/build-production-bundle.py" --project-root "$PROJECT_ROOT"
    else
        echo "❌ No sidecar build script found"
        exit 1
    fi
fi

# Install frontend dependencies
echo ""
echo "📦 Installing frontend dependencies..."
cd "$PROJECT_ROOT"
npm install

# Build frontend
echo ""
echo "🎨 Building frontend..."
npm run build

# Build Tauri app
echo ""
echo "🦀 Building Tauri app..."
cargo tauri build

# Check build results
echo ""
echo "📋 Checking build results..."

BUILD_DIR="$PROJECT_ROOT/target/release"
BUNDLE_DIR="$BUILD_DIR/bundle"

if [ -d "$BUNDLE_DIR" ]; then
    echo "✅ Build completed successfully!"
    echo ""
    echo "📦 Available installers:"
    
    # Check for different installer types
    if [ -d "$BUNDLE_DIR/dmg" ]; then
        find "$BUNDLE_DIR/dmg" -name "*.dmg" -exec echo "  🍎 macOS: {}" \;
    fi
    
    if [ -d "$BUNDLE_DIR/deb" ]; then
        find "$BUNDLE_DIR/deb" -name "*.deb" -exec echo "  🐧 Linux DEB: {}" \;
    fi
    
    if [ -d "$BUNDLE_DIR/rpm" ]; then
        find "$BUNDLE_DIR/rpm" -name "*.rpm" -exec echo "  🐧 Linux RPM: {}" \;
    fi
    
    if [ -d "$BUNDLE_DIR/appimage" ]; then
        find "$BUNDLE_DIR/appimage" -name "*.AppImage" -exec echo "  🐧 Linux AppImage: {}" \;
    fi
    
    if [ -d "$BUNDLE_DIR/msi" ]; then
        find "$BUNDLE_DIR/msi" -name "*.msi" -exec echo "  🪟 Windows MSI: {}" \;
    fi
    
    if [ -d "$BUNDLE_DIR/nsis" ]; then
        find "$BUNDLE_DIR/nsis" -name "*-setup.exe" -exec echo "  🪟 Windows NSIS: {}" \;
    fi
    
else
    echo "❌ Build failed - no bundle directory found"
    exit 1
fi

# Test the built application (platform-specific)
echo ""
echo "🧪 Testing built application..."

case "$(uname -s)" in
    Darwin)
        # macOS
        if [ -d "$BUNDLE_DIR/dmg" ]; then
            DMG_FILE=$(find "$BUNDLE_DIR/dmg" -name "*.dmg" | head -1)
            if [ -n "$DMG_FILE" ]; then
                echo "🍎 Testing macOS DMG: $(basename "$DMG_FILE")"
                
                # Mount the DMG
                MOUNT_POINT=$(hdiutil attach "$DMG_FILE" -readonly -nobrowse | grep -o '/Volumes/.*')
                
                if [ -n "$MOUNT_POINT" ]; then
                    echo "✅ DMG mounted successfully at: $MOUNT_POINT"
                    
                    # Find the app bundle
                    APP_BUNDLE=$(find "$MOUNT_POINT" -name "*.app" | head -1)
                    if [ -n "$APP_BUNDLE" ]; then
                        echo "✅ App bundle found: $(basename "$APP_BUNDLE")"
                        
                        # Check for sidecar binaries
                        RESOURCES_DIR="$APP_BUNDLE/Contents/Resources"
                        if [ -d "$RESOURCES_DIR" ]; then
                            SIDECAR_COUNT=$(find "$RESOURCES_DIR" -name "gytmdl-*" | wc -l)
                            echo "✅ Found $SIDECAR_COUNT sidecar binaries"
                        fi
                    fi
                    
                    # Unmount the DMG
                    hdiutil detach "$MOUNT_POINT" >/dev/null 2>&1
                fi
            fi
        fi
        ;;
    Linux)
        # Linux
        if [ -d "$BUNDLE_DIR/deb" ]; then
            DEB_FILE=$(find "$BUNDLE_DIR/deb" -name "*.deb" | head -1)
            if [ -n "$DEB_FILE" ]; then
                echo "🐧 Testing Linux DEB: $(basename "$DEB_FILE")"
                
                # Check package info
                if command_exists dpkg-deb; then
                    echo "📋 Package info:"
                    dpkg-deb --info "$DEB_FILE" | head -10
                    
                    echo "📦 Package contents (gytmdl files):"
                    dpkg-deb --contents "$DEB_FILE" | grep gytmdl || echo "  No gytmdl files found in package listing"
                fi
            fi
        fi
        
        if [ -d "$BUNDLE_DIR/appimage" ]; then
            APPIMAGE_FILE=$(find "$BUNDLE_DIR/appimage" -name "*.AppImage" | head -1)
            if [ -n "$APPIMAGE_FILE" ]; then
                echo "🐧 Testing Linux AppImage: $(basename "$APPIMAGE_FILE")"
                chmod +x "$APPIMAGE_FILE"
                echo "✅ AppImage is executable"
            fi
        fi
        ;;
    MINGW*|MSYS*|CYGWIN*)
        # Windows (Git Bash, MSYS2, etc.)
        if [ -d "$BUNDLE_DIR/msi" ]; then
            MSI_FILE=$(find "$BUNDLE_DIR/msi" -name "*.msi" | head -1)
            if [ -n "$MSI_FILE" ]; then
                echo "🪟 Testing Windows MSI: $(basename "$MSI_FILE")"
                echo "✅ MSI installer created"
            fi
        fi
        ;;
esac

echo ""
echo "🎉 Quick test completed successfully!"
echo ""
echo "📋 Next steps:"
echo "  1. Install the appropriate installer for your platform"
echo "  2. Launch the application"
echo "  3. Test basic functionality (add a YouTube Music URL)"
echo "  4. Verify downloads work correctly"
echo "  5. Check that the app doesn't interfere with system gytmdl (if installed)"
echo ""
echo "📚 For more detailed testing, see: docs/testing-guide.md"