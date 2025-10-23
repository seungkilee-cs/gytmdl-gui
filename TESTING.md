# Testing gytmdl-gui

This document provides simple instructions for testing the gytmdl-gui application locally.

## Quick Start

### Option 1: Complete Automated Test (Recommended)

Run the comprehensive test script that handles everything:

```bash
python3 scripts/test-and-build.py
```

This will:
- ✅ Check all dependencies
- ✅ Build sidecar binary for your platform
- ✅ Build the frontend
- ✅ Build the Tauri application
- ✅ Create platform-specific installers
- ✅ Test the installers

### Option 2: Step-by-Step Testing

If you prefer more control:

```bash
# 1. Test isolation system
python3 scripts/test-isolation.py

# 2. Build sidecar binary only
python3 scripts/build-simple-sidecar.py

# 3. Build everything
./scripts/quick-test.sh
```

### Option 3: Manual Build

For complete manual control:

```bash
# Install PyInstaller
pip install pyinstaller

# Install gytmdl in development mode
pip install -e ../gytmdl

# Build sidecar binary manually
python3 -m PyInstaller --onefile --console --name gytmdl-aarch64-apple-darwin --distpath src-tauri/sidecars /tmp/entry.py

# Build frontend
npm install
npm run build

# Build Tauri app
cargo tauri build
```

## What Gets Built

After successful building, you'll find installers in:

### macOS
- `target/release/bundle/dmg/*.dmg` - DMG installer

### Linux  
- `target/release/bundle/deb/*.deb` - DEB package
- `target/release/bundle/appimage/*.AppImage` - AppImage

### Windows
- `target/release/bundle/msi/*.msi` - MSI installer

## Testing as End User

### Install and Test

1. **Install the appropriate package for your platform**
2. **Launch the application**
3. **Test basic functionality**:
   - Add a YouTube Music URL
   - Configure download settings
   - Start a download
   - Verify files are saved correctly

### Test URLs

Try these test URLs to verify functionality:

```
# Single track
https://music.youtube.com/watch?v=dQw4w9WgXcQ

# Album  
https://music.youtube.com/playlist?list=OLAK5uy_l1x-JAx0w53suECoCI0YJtW6VB8DBQWRQ

# Playlist
https://music.youtube.com/playlist?list=PLrAl6rYgs4IvGFBDEaVGFXt6K2cqfzgKN
```

## Configuration Isolation

The GUI app is completely isolated from system gytmdl installations:

### Separate Directories
- **System gytmdl**: `~/.config/gytmdl/`
- **GUI app**: `~/.config/gytmdl-gui/`

### Test Isolation
```bash
# Test that both can coexist
python3 scripts/test-isolation.py
```

This verifies:
- ✅ GUI app uses its own directories
- ✅ System gytmdl (if installed) is unaffected
- ✅ Environment variables are properly isolated
- ✅ No configuration conflicts

## Troubleshooting

### Common Issues

**Sidecar build fails**:
```bash
# Check if gytmdl source exists
ls -la ../gytmdl

# Install PyInstaller
pip install pyinstaller

# Install gytmdl dependencies
pip install -e ../gytmdl
```

**Frontend build fails**:
```bash
# Check Node.js version (should be 18+)
node --version

# Clean and reinstall
rm -rf node_modules package-lock.json
npm install
```

**Tauri build fails**:
```bash
# Check Rust version
cargo --version

# Update Rust
rustup update

# Clean build
cargo clean
```

### Getting Help

1. **Check the logs** in the terminal output
2. **Run individual test scripts** to isolate issues
3. **Verify all dependencies** are installed correctly
4. **Check the detailed guides** in `docs/` directory

## Advanced Testing

For more comprehensive testing, see:
- `docs/testing-guide.md` - Detailed testing procedures
- `docs/testing-and-isolation-guide.md` - Isolation system details
- `scripts/test-local-build.py` - Advanced local testing
- `scripts/validate-release-pipeline.py` - Pipeline validation

## Success Criteria

A successful test should result in:
- ✅ Sidecar binary builds without errors
- ✅ Frontend compiles successfully  
- ✅ Tauri app builds and creates installers
- ✅ Installers can be opened/installed
- ✅ App launches and basic functionality works
- ✅ Downloads complete successfully
- ✅ No interference with system gytmdl (if installed)

Once these criteria are met, the application is ready for distribution!