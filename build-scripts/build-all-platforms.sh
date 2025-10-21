#!/bin/bash
# Cross-platform build script for gytmdl sidecar binaries
# This script should be run on each target platform to generate the appropriate binary

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
GYTMDL_SRC="$(dirname "$PROJECT_ROOT")/gytmdl"
OUTPUT_DIR="$PROJECT_ROOT/src-tauri/sidecars"

echo -e "${GREEN}=== gytmdl Sidecar Binary Builder ===${NC}"
echo "Script directory: $SCRIPT_DIR"
echo "Project root: $PROJECT_ROOT"
echo "gytmdl source: $GYTMDL_SRC"
echo "Output directory: $OUTPUT_DIR"

# Check if Python is available
if ! command -v python3 &> /dev/null; then
    if ! command -v python &> /dev/null; then
        echo -e "${RED}Error: Python is not installed or not in PATH${NC}"
        exit 1
    else
        PYTHON_CMD="python"
    fi
else
    PYTHON_CMD="python3"
fi

echo -e "${GREEN}Using Python: $PYTHON_CMD${NC}"

# Check if pip is available
if ! $PYTHON_CMD -m pip --version &> /dev/null; then
    echo -e "${RED}Error: pip is not available${NC}"
    exit 1
fi

# Install PyInstaller if not available
if ! $PYTHON_CMD -c "import PyInstaller" 2>/dev/null; then
    echo -e "${YELLOW}Installing PyInstaller...${NC}"
    $PYTHON_CMD -m pip install pyinstaller
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Run the Python build script
echo -e "${GREEN}Building sidecar binary for current platform...${NC}"
$PYTHON_CMD "$SCRIPT_DIR/build-sidecars.py" \
    --gytmdl-src "$GYTMDL_SRC" \
    --output-dir "$OUTPUT_DIR"

if [ $? -eq 0 ]; then
    echo -e "${GREEN}Build completed successfully!${NC}"
    echo -e "${GREEN}Binaries are available in: $OUTPUT_DIR${NC}"
    ls -la "$OUTPUT_DIR"
else
    echo -e "${RED}Build failed!${NC}"
    exit 1
fi