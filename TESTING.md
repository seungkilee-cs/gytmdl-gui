# Testing gytmdl-gui

This guide explains how to test the gytmdl-gui application to ensure everything works correctly.

## Prerequisites

Before testing, make sure you have:

1. **Node.js** (v18 or later) and **npm**
2. **Rust** and **Cargo** (latest stable)
3. **Python 3.7+** with **pip**
4. **gytmdl** source code (should be in `../gytmdl` relative to this project)

## Quick Start Testing

### 1. Install Dependencies

```bash
# Install frontend dependencies
npm install

# Install Tauri CLI (if not already installed)
npm install -g @tauri-apps/cli

# Install Python dependencies for sidecar building
pip install pyinstaller
```

### 2. Build Sidecar Binaries (Optional)

If you want to test with actual gytmdl binaries:

```bash
# Make sure gytmdl source is available
ls ../gytmdl  # Should show gytmdl source files

# Build sidecar binaries for current platform
./build-scripts/build-all-platforms.sh  # On Unix
# OR
build-scripts\build-all-platforms.bat   # On Windows
```

### 3. Run the Application in Development Mode

```bash
npm run tauri dev
```

This will:
- Build the frontend
- Start the Rust backend
- Open the application window

### 4. Test Basic Functionality

Once the app opens:

1. **Test URL Addition:**
   - Paste a YouTube Music URL (e.g., `https://music.youtube.com/watch?v=dQw4w9WgXcQ`)
   - Click "Add" button
   - The URL should appear in the queue

2. **Test Queue Management:**
   - Try pausing/resuming the queue
   - Test removing jobs
   - Test clearing completed jobs

3. **Test Configuration:**
   - Click on "Config" tab
   - Modify settings like output path, audio quality
   - Save configuration

4. **Test Cookie Management:**
   - Click on "Cookies" tab
   - Test cookie import functionality

## Testing Without gytmdl Binary

If you don't have gytmdl binaries built, the app will still work but downloads won't actually process. You can test:

- UI functionality
- Queue management
- Configuration saving/loading
- Cookie management interface

The app will show appropriate error messages when trying to download without a valid gytmdl binary.

## Running Tests

### Frontend Tests

```bash
# Run frontend unit tests (if any)
npm test
```

### Backend Tests

```bash
# Run Rust tests
cd src-tauri
cargo test
```

### Integration Tests

```bash
# Run packaging tests
python scripts/test-packaging.py
```

## Building for Production

### Development Build

```bash
# Quick development build
./scripts/dev-build.sh
```

### Full Production Build

```bash
# Build everything including sidecars and installers
python scripts/build-and-package.py
```

## Common Issues and Solutions

### 1. "Binary not found" Error

**Problem:** The app can't find the gytmdl binary.

**Solution:**
- Build sidecar binaries using the build scripts
- Or place a gytmdl binary in `src-tauri/sidecars/`
- Or install gytmdl in your system PATH

### 2. "Failed to add URL" Error

**Problem:** URL validation or backend communication issue.

**Solution:**
- Make sure the URL is a valid YouTube Music URL
- Check the browser console for detailed error messages
- Verify the Rust backend is running

### 3. Frontend Build Errors

**Problem:** npm build fails.

**Solution:**
```bash
# Clear cache and reinstall
rm -rf node_modules package-lock.json
npm install
```

### 4. Rust Compilation Errors

**Problem:** Cargo build fails.

**Solution:**
```bash
# Update Rust and dependencies
rustup update
cd src-tauri
cargo update
cargo clean
cargo build
```

## Testing Checklist

Use this checklist to verify all functionality:

- [ ] App starts without errors
- [ ] Can add YouTube Music URLs to queue
- [ ] Queue displays jobs correctly
- [ ] Can pause/resume queue
- [ ] Can remove individual jobs
- [ ] Can clear completed jobs
- [ ] Configuration tab loads
- [ ] Can modify and save configuration
- [ ] Cookie management tab loads
- [ ] Can import cookie files
- [ ] No console errors in browser dev tools
- [ ] App responds to window resize
- [ ] All navigation tabs work

## Performance Testing

For performance testing:

1. Add multiple URLs (10-20) to the queue
2. Monitor memory usage
3. Check for memory leaks during long runs
4. Test with large playlists

## Platform-Specific Testing

### Windows
- Test with Windows Defender enabled
- Verify installer works correctly
- Test with different user permissions

### macOS
- Test on both Intel and Apple Silicon Macs
- Verify app bundle is properly signed
- Test Gatekeeper compatibility

### Linux
- Test on different distributions (Ubuntu, Fedora, etc.)
- Verify .deb and .rpm packages install correctly
- Test with different desktop environments

## Reporting Issues

When reporting issues, include:

1. Operating system and version
2. Node.js and Rust versions
3. Steps to reproduce the issue
4. Console output and error messages
5. Screenshots if applicable

## Automated Testing

The project includes GitHub Actions workflows for automated testing:

- **Build Tests:** Verify the app builds on all platforms
- **Integration Tests:** Test packaging and distribution
- **Release Tests:** Verify installers work correctly

Check the `.github/workflows/` directory for workflow definitions.