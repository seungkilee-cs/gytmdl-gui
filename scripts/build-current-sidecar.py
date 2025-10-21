#!/usr/bin/env python3
"""
Build sidecar binary for the current platform only.
"""

import os
import sys
import subprocess
import platform
import shutil
import hashlib
import json
from pathlib import Path

def build_current_sidecar():
    """Build sidecar binary for current platform."""
    print("🔨 Building sidecar binary for current platform...")
    
    # Get paths
    project_root = Path(__file__).parent.parent
    gytmdl_src = project_root.parent / "gytmdl"
    sidecars_dir = project_root / "src-tauri" / "sidecars"
    spec_file = project_root / "build-scripts" / "pyinstaller-config.spec"
    
    # Ensure directories exist
    sidecars_dir.mkdir(parents=True, exist_ok=True)
    
    # Get platform info
    system = platform.system().lower()
    machine = platform.machine().lower()
    
    if system == "darwin":
        if machine == "arm64":
            target = "aarch64-apple-darwin"
        else:
            target = "x86_64-apple-darwin"
        extension = ""
    elif system == "windows":
        target = "x86_64-pc-windows-msvc" if "64" in str(sys.maxsize) else "i686-pc-windows-msvc"
        extension = ".exe"
    elif system == "linux":
        target = f"{machine}-unknown-linux-gnu"
        extension = ""
    else:
        raise ValueError(f"Unsupported platform: {system}")
    
    binary_name = f"gytmdl-{target}{extension}"
    print(f"Building: {binary_name}")
    
    # Install gytmdl dependencies
    print("📦 Installing gytmdl dependencies...")
    try:
        subprocess.run([
            sys.executable, "-m", "pip", "install", "-e", str(gytmdl_src)
        ], check=True, cwd=gytmdl_src)
        print("✅ Dependencies installed")
    except subprocess.CalledProcessError as e:
        print(f"❌ Failed to install dependencies: {e}")
        return False
    
    # Run PyInstaller
    print("🏗️ Running PyInstaller...")
    try:
        cmd = [
            sys.executable, "-m", "PyInstaller",
            "--clean",
            "--onefile",
            "--name", binary_name,
            "--distpath", str(sidecars_dir),
            str(gytmdl_src / "gytmdl" / "__main__.py")
        ]
        
        print(f"Command: {' '.join(cmd)}")
        result = subprocess.run(cmd, cwd=gytmdl_src, capture_output=True, text=True)
        
        if result.returncode != 0:
            print(f"❌ PyInstaller failed:")
            print(f"STDOUT: {result.stdout}")
            print(f"STDERR: {result.stderr}")
            return False
        
        print("✅ PyInstaller completed")
        
    except Exception as e:
        print(f"❌ PyInstaller error: {e}")
        return False
    
    # Check if binary was created
    binary_path = sidecars_dir / binary_name
    if not binary_path.exists():
        print(f"❌ Binary not found at {binary_path}")
        # List what's in the sidecars directory
        print("Contents of sidecars directory:")
        for item in sidecars_dir.iterdir():
            print(f"  - {item.name}")
        return False
    
    print(f"✅ Binary created: {binary_path}")
    print(f"Size: {binary_path.stat().st_size / 1024 / 1024:.1f} MB")
    
    # Test the binary
    print("🧪 Testing binary...")
    try:
        result = subprocess.run([str(binary_path), "--version"], 
                              capture_output=True, text=True, timeout=30)
        
        if result.returncode == 0:
            print(f"✅ Binary test passed: {result.stdout.strip()}")
        else:
            print(f"⚠️ Binary test failed (return code: {result.returncode})")
            print(f"STDERR: {result.stderr}")
    
    except subprocess.TimeoutExpired:
        print("⚠️ Binary test timed out")
    except Exception as e:
        print(f"⚠️ Binary test error: {e}")
    
    # Create manifest
    print("📋 Creating manifest...")
    checksum = hashlib.sha256()
    with open(binary_path, "rb") as f:
        for chunk in iter(lambda: f.read(4096), b""):
            checksum.update(chunk)
    
    manifest = {
        "binary_name": binary_name,
        "platform": {
            "os": system,
            "arch": machine,
            "target": target,
            "extension": extension
        },
        "size_bytes": binary_path.stat().st_size,
        "sha256": checksum.hexdigest(),
        "build_timestamp": subprocess.run(
            ["date", "-u", "+%Y-%m-%dT%H:%M:%SZ"], 
            capture_output=True, text=True
        ).stdout.strip() if sys.platform != "win32" else "unknown"
    }
    
    manifest_path = sidecars_dir / f"{binary_name}.json"
    with open(manifest_path, "w") as f:
        json.dump(manifest, f, indent=2)
    
    print(f"✅ Manifest created: {manifest_path}")
    
    print("🎉 Sidecar build completed successfully!")
    return True

if __name__ == "__main__":
    success = build_current_sidecar()
    sys.exit(0 if success else 1)