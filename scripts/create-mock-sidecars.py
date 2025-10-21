#!/usr/bin/env python3
"""
Create mock sidecar binaries for testing the bundle configuration.
"""

import os
import sys
import platform
import hashlib
import json
from pathlib import Path

def create_mock_sidecars():
    """Create mock sidecar binaries for all platforms."""
    print("🎭 Creating mock sidecar binaries...")
    
    # Get paths
    project_root = Path(__file__).parent.parent
    sidecars_dir = project_root / "src-tauri" / "sidecars"
    
    # Ensure directory exists
    sidecars_dir.mkdir(parents=True, exist_ok=True)
    
    # Target platforms
    platforms = [
        {
            "name": "Windows x64",
            "target": "x86_64-pc-windows-msvc",
            "extension": ".exe",
            "os": "windows",
            "arch": "x86_64"
        },
        {
            "name": "Windows x86",
            "target": "i686-pc-windows-msvc", 
            "extension": ".exe",
            "os": "windows",
            "arch": "i686"
        },
        {
            "name": "macOS Intel",
            "target": "x86_64-apple-darwin",
            "extension": "",
            "os": "macos",
            "arch": "x86_64"
        },
        {
            "name": "macOS Apple Silicon",
            "target": "aarch64-apple-darwin",
            "extension": "",
            "os": "macos", 
            "arch": "aarch64"
        },
        {
            "name": "Linux x64",
            "target": "x86_64-unknown-linux-gnu",
            "extension": "",
            "os": "linux",
            "arch": "x86_64"
        },
        {
            "name": "Linux ARM64",
            "target": "aarch64-unknown-linux-gnu",
            "extension": "",
            "os": "linux",
            "arch": "aarch64"
        }
    ]
    
    # Get current platform info
    current_system = platform.system().lower()
    current_machine = platform.machine().lower()
    
    if current_system == "darwin":
        current_target = "aarch64-apple-darwin" if current_machine == "arm64" else "x86_64-apple-darwin"
    elif current_system == "windows":
        current_target = "x86_64-pc-windows-msvc" if "64" in str(sys.maxsize) else "i686-pc-windows-msvc"
    elif current_system == "linux":
        current_target = f"{current_machine}-unknown-linux-gnu"
    else:
        current_target = None
    
    created_count = 0
    
    for platform_info in platforms:
        binary_name = f"gytmdl-{platform_info['target']}{platform_info['extension']}"
        binary_path = sidecars_dir / binary_name
        manifest_path = sidecars_dir / f"{binary_name}.json"
        
        # Create mock binary content
        if platform_info["target"] == current_target:
            # For current platform, create a more realistic mock
            mock_content = f"""#!/bin/bash
echo "gytmdl mock binary for {platform_info['name']}"
echo "Version: 1.0.0-mock"
echo "Platform: {platform_info['target']}"
echo "This is a functional mock for the current platform"
if [ "$1" = "--version" ]; then
    echo "gytmdl 1.0.0-mock"
    exit 0
fi
echo "Mock binary - replace with real gytmdl for production"
exit 0
""".encode()
        else:
            # For other platforms, create simple mock
            mock_content = f"""#!/bin/bash
echo "gytmdl mock binary for {platform_info['name']}"
echo "Platform: {platform_info['target']}"
echo "This binary should be replaced with a real build for production use"
exit 1
""".encode()
        
        if platform_info["os"] == "windows":
            # For Windows, create a batch file content
            if platform_info["target"] == current_target:
                mock_content = f"""@echo off
echo gytmdl mock binary for {platform_info['name']}
echo Version: 1.0.0-mock
echo Platform: {platform_info['target']}
echo This is a functional mock for the current platform
if "%1"=="--version" (
    echo gytmdl 1.0.0-mock
    exit /b 0
)
echo Mock binary - replace with real gytmdl for production
exit /b 0
""".encode()
            else:
                mock_content = f"""@echo off
echo gytmdl mock binary for {platform_info['name']}
echo Platform: {platform_info['target']}
echo This binary should be replaced with a real build for production use
exit /b 1
""".encode()
        
        # Write binary
        binary_path.write_bytes(mock_content)
        
        # Make executable on Unix-like systems
        if platform_info["os"] != "windows":
            os.chmod(binary_path, 0o755)
        
        # Create manifest
        manifest = {
            "binary_name": binary_name,
            "platform": {
                "os": platform_info["os"],
                "arch": platform_info["arch"],
                "target": platform_info["target"],
                "extension": platform_info["extension"]
            },
            "size_bytes": len(mock_content),
            "sha256": hashlib.sha256(mock_content).hexdigest(),
            "build_timestamp": "mock-build-2024-01-01T00:00:00Z",
            "is_mock": True,
            "is_current_platform": platform_info["target"] == current_target
        }
        
        with open(manifest_path, "w") as f:
            json.dump(manifest, f, indent=2)
        
        created_count += 1
        status = "✅ (current platform)" if platform_info["target"] == current_target else "📝 (mock)"
        print(f"  {status} Created: {binary_name}")
    
    print(f"🎉 Created {created_count} mock sidecar binaries")
    print(f"📁 Location: {sidecars_dir}")
    
    # List all created files
    print("\nCreated files:")
    for item in sorted(sidecars_dir.iterdir()):
        if item.is_file():
            size = item.stat().st_size
            print(f"  - {item.name} ({size} bytes)")
    
    return True

if __name__ == "__main__":
    success = create_mock_sidecars()
    sys.exit(0 if success else 1)