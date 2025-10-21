#!/bin/bash
# Development build script for gytmdl-gui
# This script provides a quick way to build the app for local testing

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo -e "${BLUE}🚀 gytmdl-gui Development Build${NC}"
echo "Project root: $PROJECT_ROOT"

# Parse command line arguments
BUILD_SIDECARS=true
BUILD_FRONTEND=true
BUILD_TAURI=true
RELEASE_MODE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --no-sidecars)
            BUILD_SIDECARS=false
            shift
            ;;
        --no-frontend)
            BUILD_FRONTEND=false
            shift
            ;;
        --no-tauri)
            BUILD_TAURI=false
            shift
            ;;
        --release)
            RELEASE_MODE=true
            shift
            ;;
        --help)
            echo "Usage: $0 [options]"
            echo "Options:"
            echo "  --no-sidecars   Skip building sidecar binaries"
            echo "  --no-frontend   Skip building frontend"
            echo "  --no-tauri      Skip building Tauri app"
            echo "  --release       Build in release mode"
            echo "  --help          Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

cd "$PROJECT_ROOT"

# Check if required tools are available
echo -e "${YELLOW}🔍 Checking dependencies...${NC}"

check_command() {
    if ! command -v "$1" &> /dev/null; then
        echo -e "${RED}❌ $1 is not installed or not in PATH${NC}"
        exit 1
    else
        echo -e "${GREEN}✓ $1 found${NC}"
    fi
}

check_command node
check_command npm
check_command cargo
check_command python3

# Build sidecar binaries
if [ "$BUILD_SIDECARS" = true ]; then
    echo -e "\n${YELLOW}🔨 Building sidecar binaries...${NC}"
    
    if [ -f "build-scripts/build-all-platforms.sh" ]; then
        chmod +x build-scripts/build-all-platforms.sh
        ./build-scripts/build-all-platforms.sh
    else
        echo -e "${RED}❌ Sidecar build script not found${NC}"
        exit 1
    fi
fi

# Build frontend
if [ "$BUILD_FRONTEND" = true ]; then
    echo -e "\n${YELLOW}🎨 Building frontend...${NC}"
    
    # Install dependencies if node_modules doesn't exist
    if [ ! -d "node_modules" ]; then
        echo "Installing npm dependencies..."
        npm install
    fi
    
    # Build frontend
    npm run build
fi

# Build Tauri app
if [ "$BUILD_TAURI" = true ]; then
    echo -e "\n${YELLOW}🦀 Building Tauri application...${NC}"
    
    # Install Tauri CLI if not available
    if ! command -v cargo-tauri &> /dev/null; then
        echo "Installing Tauri CLI..."
        cargo install tauri-cli
    fi
    
    # Build command
    if [ "$RELEASE_MODE" = true ]; then
        echo "Building in release mode..."
        cargo tauri build
    else
        echo "Building in development mode..."
        cargo tauri build --debug
    fi
fi

echo -e "\n${GREEN}🎉 Build completed successfully!${NC}"

# Show output information
if [ "$BUILD_TAURI" = true ]; then
    echo -e "\n${BLUE}📁 Build outputs:${NC}"
    
    if [ "$RELEASE_MODE" = true ]; then
        BUILD_DIR="src-tauri/target/release"
        echo "Release build directory: $BUILD_DIR"
    else
        BUILD_DIR="src-tauri/target/debug"
        echo "Debug build directory: $BUILD_DIR"
    fi
    
    # List built files
    if [ -d "$BUILD_DIR" ]; then
        echo "Built executable:"
        ls -la "$BUILD_DIR"/gytmdl-gui* 2>/dev/null || echo "  (executable not found)"
        
        if [ -d "$BUILD_DIR/bundle" ]; then
            echo "Installers:"
            find "$BUILD_DIR/bundle" -name "*.dmg" -o -name "*.deb" -o -name "*.rpm" -o -name "*.msi" -o -name "*.AppImage" -o -name "*-setup.exe" 2>/dev/null | head -5
        fi
    fi
fi

echo -e "\n${GREEN}✅ Development build ready!${NC}"

# Provide next steps
echo -e "\n${BLUE}Next steps:${NC}"
if [ "$RELEASE_MODE" = false ]; then
    echo "• Run the app: cargo tauri dev"
    echo "• Or run the built executable from: $BUILD_DIR"
else
    echo "• Install and test the generated installer packages"
    echo "• Or run the built executable from: $BUILD_DIR"
fi
echo "• Check logs in the terminal for any issues"