# gytmdl GUI

A cross-platform desktop GUI wrapper for [gytmdl](https://github.com/glomatico/gytmdl) (Glomatico's YouTube Music Downloader).

## Motivation

I fell in love the CLI when I discovered it. But when I recommended the tool to some of my friends, despite the intuitive CLI interaction, some people who were less technical felt simply were too uncomfortable with the commandline to use a tool they love the functionality of. So I wanted to build the GUI wrapper for people to use, and add some QoL features.

## Overview

gytmdl GUI provides an intuitive interface for downloading YouTube Music content with features like:

- Queue Management: Add multiple URLs and manage downloads with pause/resume/retry
- Real-time Progress: Visual progress bars showing download stages and completion
- Cookie Management: Easy import and validation of YouTube Music cookies
- Configuration GUI: User-friendly interface for all gytmdl settings
- Cross-platform: Native apps for Windows, macOS, and Linux
- Sidecar Architecture: Bundles gytmdl CLI for easy updates and stability

## Architecture

Built with:
- Frontend: React + TypeScript + Vite
- Backend: Rust + Tauri
- Sidecar: Bundled gytmdl executable (PyInstaller)

The sidecar architecture ensures the GUI stays compatible with gytmdl updates while providing enhanced user experience.

## Development

### Prerequisites

- Node.js 18+ and npm/yarn
- Rust 1.70+ with Cargo
- Python 3.10+ (for building sidecar binaries)
- System dependencies:
  - macOS: Xcode Command Line Tools
  - Linux: `build-essential`, `libwebkit2gtk-4.0-dev`, `libssl-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`
  - Windows: Microsoft C++ Build Tools

### Setup

```bash
# Clone and enter directory
cd gytmdl-gui

# Install frontend dependencies
npm install

# Install Tauri CLI
npm install -g @tauri-apps/cli

# Run in development mode
npm run tauri dev
```

### Building

```bash
# Build for current platform
npm run tauri build

# Build sidecar binaries (requires Python)
./scripts/build_sidecars.sh
```

## Project Structure

```
gytmdl-gui/
├── src/                          # React frontend
│   ├── components/               # UI components
│   ├── hooks/                    # React hooks
│   ├── types/                    # TypeScript definitions
│   └── App.tsx                   # Main app component
├── src-tauri/                    # Rust backend
│   ├── src/                      # Rust source code
│   ├── binaries/                 # Sidecar binaries
│   └── tauri.conf.json          # Tauri configuration
├── scripts/                      # Build and utility scripts
└── docs/                         # Documentation
```

## Features

### Queue Management
- Add YouTube Music URLs (songs, albums, playlists, artists)
- Concurrent download limiting
- Individual job control (retry, cancel, remove)
- Queue pause/resume functionality

### Progress Visualization
- Real time progress bars with stage indicators
- Download statistics and completion status
- Detailed log viewer for troubleshooting

### Cookie Management
- Import cookies from browser extensions
- Cookie validation and expiration warnings
- PO token support for premium content

### Configuration
- GUI for all gytmdl settings
- Output path and quality selection
- Template customization for file naming
- Configuration profiles and presets

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details. The project also bundles [gytmdl](https://github.com/glomatico/gytmdl) by glomatico, which is also licensed under the MIT License.


## Acknowledgments

- [gytmdl](https://github.com/glomatico/gytmdl) by glomatico for the core downloading functionality.