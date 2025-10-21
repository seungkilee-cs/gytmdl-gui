# gytmdl Sidecar Binary Build Scripts

This directory contains scripts and configuration for building gytmdl sidecar binaries that can be bundled with the Tauri application.

## Overview

The sidecar architecture allows the GUI to use gytmdl as a separate process, providing several benefits:
- Easy updates without recompiling the entire app
- Stability (gytmdl crashes don't affect the GUI)
- Clear licensing boundaries
- Maintainability (no need to fork gytmdl)

## Files

- `pyinstaller-config.spec` - PyInstaller specification for building gytmdl binaries
- `build-sidecars.py` - Python script for building platform-specific binaries
- `build-all-platforms.sh` - Shell script for Unix-like systems (macOS, Linux)
- `build-all-platforms.bat` - Batch script for Windows
- `README.md` - This documentation file

## Prerequisites

1. **Python 3.7+** with pip
2. **PyInstaller** (`pip install pyinstaller`)
3. **gytmdl source code** (should be in `../../../gytmdl` relative to this directory)
4. **Platform-specific requirements**:
   - Windows: Visual Studio Build Tools or Visual Studio
   - macOS: Xcode Command Line Tools
   - Linux: GCC and development libraries

## Building Binaries

### Automatic Build (Recommended)

Run the appropriate script for your platform:

**Unix-like systems (macOS, Linux):**
```bash
./build-all-platforms.sh
```

**Windows:**
```cmd
build-all-platforms.bat
```

### Manual Build

If you need more control over the build process:

```bash
python build-sidecars.py --gytmdl-src /path/to/gytmdl --output-dir /path/to/output
```

### Cross-Platform Building

To build binaries for all supported platforms, you need to run the build script on each target platform:

1. **Windows (x86_64)**: Run on Windows 10/11 x64
2. **macOS (Intel)**: Run on macOS with Intel processor
3. **macOS (Apple Silicon)**: Run on macOS with M1/M2 processor
4. **Linux (x86_64)**: Run on Linux x64 distribution

## Output

Built binaries will be placed in `../src-tauri/sidecars/` with the following naming convention:

- `gytmdl-x86_64-pc-windows-msvc.exe` (Windows x64)
- `gytmdl-x86_64-apple-darwin` (macOS Intel)
- `gytmdl-aarch64-apple-darwin` (macOS Apple Silicon)
- `gytmdl-x86_64-unknown-linux-gnu` (Linux x64)

Each binary will have an accompanying `.json` manifest file with metadata:
- Binary size and checksum
- Platform information
- Build timestamp

## Integration with Tauri

The built binaries are automatically detected by the Tauri application through:

1. **Bundle Configuration**: `tauri.conf.json` includes the sidecar binaries in the app bundle
2. **Runtime Detection**: The Rust code automatically selects the appropriate binary for the current platform
3. **Validation**: Binary integrity is checked using the manifest files

## Binary Validation

The build system includes several validation steps:

1. **Functionality Test**: Each binary is tested with `--version` command
2. **Integrity Check**: SHA256 checksums are calculated and stored
3. **Platform Detection**: Automatic selection of the correct binary at runtime
4. **Manifest Validation**: Metadata consistency checks

## Troubleshooting

### Common Issues

**PyInstaller not found:**
```bash
pip install pyinstaller
```

**gytmdl source not found:**
- Ensure the gytmdl repository is cloned to `../../../gytmdl`
- Or specify the correct path with `--gytmdl-src`

**Permission denied (Unix):**
```bash
chmod +x build-all-platforms.sh
```

**Binary too large:**
- The build script uses UPX compression to reduce binary size
- Install UPX for better compression: https://upx.github.io/

**Import errors during build:**
- Ensure all gytmdl dependencies are installed: `pip install -r ../../../gytmdl/requirements.txt`
- Check that gytmdl can be imported: `python -c "import gytmdl"`

### Debug Mode

For debugging build issues, you can run PyInstaller directly:

```bash
pyinstaller --clean --onefile --console pyinstaller-config.spec
```

### Verbose Output

Add `--verbose` flag to the build script for detailed output:

```bash
python build-sidecars.py --verbose
```

## Security Considerations

- Binaries are validated using checksums to ensure integrity
- The build process excludes unnecessary modules to reduce attack surface
- Code signing should be implemented for production releases (especially on macOS and Windows)

## Updating gytmdl

When updating to a new version of gytmdl:

1. Update the gytmdl source code
2. Rebuild all platform binaries
3. Test the new binaries with the GUI
4. Update version information in the app

## CI/CD Integration

For automated builds, consider:

1. **GitHub Actions**: Build on multiple platforms simultaneously
2. **Docker**: Use containers for consistent Linux builds
3. **Code Signing**: Automate certificate-based signing for distribution
4. **Artifact Storage**: Store binaries for easy distribution

Example GitHub Actions workflow structure:
```yaml
jobs:
  build-windows:
    runs-on: windows-latest
  build-macos-intel:
    runs-on: macos-latest
  build-macos-arm:
    runs-on: macos-latest-xlarge
  build-linux:
    runs-on: ubuntu-latest
```