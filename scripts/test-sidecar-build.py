#!/usr/bin/env python3
"""
Simple test script to verify sidecar build functionality.
"""

import os
import sys
import subprocess
import platform
from pathlib import Path

def test_sidecar_build():
    """Test the sidecar build process."""
    print("🧪 Testing sidecar build process...")
    
    # Get paths
    project_root = Path(__file__).parent.parent
    gytmdl_src = project_root.parent / "gytmdl"
    sidecars_dir = project_root / "src-tauri" / "sidecars"
    
    print(f"Project root: {project_root}")
    print(f"gytmdl source: {gytmdl_src}")
    print(f"Sidecars dir: {sidecars_dir}")
    
    # Check if gytmdl source exists
    if not gytmdl_src.exists():
        print(f"❌ gytmdl source not found at {gytmdl_src}")
        return False
    
    print(f"✅ gytmdl source found")
    
    # Check if gytmdl can be imported
    sys.path.insert(0, str(gytmdl_src))
    try:
        import gytmdl
        print(f"✅ gytmdl module imported successfully")
        print(f"gytmdl location: {gytmdl.__file__}")
    except ImportError as e:
        print(f"❌ Cannot import gytmdl: {e}")
        return False
    
    # Test PyInstaller
    try:
        result = subprocess.run([sys.executable, "-m", "PyInstaller", "--version"], 
                              capture_output=True, text=True)
        if result.returncode == 0:
            print(f"✅ PyInstaller available: {result.stdout.strip()}")
        else:
            print(f"❌ PyInstaller test failed")
            return False
    except Exception as e:
        print(f"❌ PyInstaller error: {e}")
        return False
    
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
        print(f"❌ Unsupported platform: {system}")
        return False
    
    print(f"✅ Platform: {system} {machine}")
    print(f"✅ Target: {target}")
    
    # Create expected binary name
    binary_name = f"gytmdl-{target}{extension}"
    print(f"✅ Expected binary name: {binary_name}")
    
    # Ensure sidecars directory exists
    sidecars_dir.mkdir(parents=True, exist_ok=True)
    
    print("🎉 All checks passed! Ready for sidecar build.")
    return True

if __name__ == "__main__":
    success = test_sidecar_build()
    sys.exit(0 if success else 1)