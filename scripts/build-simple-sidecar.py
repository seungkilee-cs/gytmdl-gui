#!/usr/bin/env python3
"""
Simple sidecar builder that creates a gytmdl binary for the current platform.
This is a fallback approach that's more reliable than the complex PyInstaller setup.
"""

import os
import sys
import subprocess
import shutil
import platform
import tempfile
from pathlib import Path

def get_platform_info():
    """Get current platform information."""
    system = platform.system().lower()
    machine = platform.machine().lower()
    
    if system == "darwin":
        if machine == "arm64":
            return "aarch64-apple-darwin", ""
        else:
            return "x86_64-apple-darwin", ""
    elif system == "linux":
        if machine == "x86_64":
            return "x86_64-unknown-linux-gnu", ""
        elif machine == "aarch64":
            return "aarch64-unknown-linux-gnu", ""
        else:
            return "unknown-linux-gnu", ""
    elif system == "windows":
        if "64" in str(sys.maxsize):
            return "x86_64-pc-windows-msvc", ".exe"
        else:
            return "i686-pc-windows-msvc", ".exe"
    else:
        return "unknown", ""

def main():
    """Main entry point."""
    print("=== Simple gytmdl Sidecar Builder ===")
    
    # Get paths
    script_dir = Path(__file__).parent
    project_root = script_dir.parent
    gytmdl_src = project_root.parent / "gytmdl"
    sidecars_dir = project_root / "src-tauri" / "sidecars"
    
    # Create sidecars directory
    sidecars_dir.mkdir(parents=True, exist_ok=True)
    
    # Get platform info
    target, extension = get_platform_info()
    binary_name = f"gytmdl-{target}{extension}"
    output_path = sidecars_dir / binary_name
    
    print(f"Target platform: {target}")
    print(f"Output binary: {output_path}")
    
    # Check if gytmdl source exists
    if not gytmdl_src.exists():
        print(f"❌ gytmdl source not found at {gytmdl_src}")
        print("Please ensure the gytmdl repository is cloned in the parent directory")
        return 1
    
    print(f"✅ gytmdl source found at {gytmdl_src}")
    
    # Check PyInstaller
    try:
        subprocess.run([sys.executable, "-m", "PyInstaller", "--version"], 
                      capture_output=True, check=True)
        print("✅ PyInstaller is available")
    except (subprocess.CalledProcessError, FileNotFoundError):
        print("❌ PyInstaller not found. Installing...")
        try:
            subprocess.run([sys.executable, "-m", "pip", "install", "pyinstaller"], 
                          check=True)
            print("✅ PyInstaller installed")
        except subprocess.CalledProcessError:
            print("❌ Failed to install PyInstaller")
            return 1
    
    # Install gytmdl in development mode
    print("📦 Installing gytmdl...")
    try:
        subprocess.run([sys.executable, "-m", "pip", "install", "-e", str(gytmdl_src)], 
                      check=True, capture_output=True)
        print("✅ gytmdl installed")
    except subprocess.CalledProcessError as e:
        print(f"❌ Failed to install gytmdl: {e}")
        return 1
    
    # Create a simple entry script
    with tempfile.NamedTemporaryFile(mode='w', suffix='.py', delete=False) as f:
        f.write("""
import sys
import gytmdl.cli

if __name__ == "__main__":
    gytmdl.cli.main()
""")
        entry_script = f.name
    
    try:
        # Build with PyInstaller using simple approach
        print("🔨 Building binary with PyInstaller...")
        
        cmd = [
            sys.executable, "-m", "PyInstaller",
            "--onefile",
            "--console",
            "--name", binary_name.replace(extension, ""),  # Remove extension, PyInstaller adds it
            "--distpath", str(sidecars_dir),
            "--workpath", str(sidecars_dir / "build"),
            "--specpath", str(sidecars_dir / "spec"),
            entry_script
        ]
        
        print(f"Running: {' '.join(cmd)}")
        result = subprocess.run(cmd, capture_output=True, text=True)
        
        if result.returncode != 0:
            print(f"❌ PyInstaller failed:")
            print(f"STDOUT: {result.stdout}")
            print(f"STDERR: {result.stderr}")
            return 1
        
        # Check if binary was created
        expected_binary = sidecars_dir / (binary_name.replace(extension, "") + extension)
        if not expected_binary.exists():
            # Try with the exact name
            expected_binary = sidecars_dir / binary_name
        
        if expected_binary.exists():
            # Rename to our expected name if needed
            if expected_binary.name != binary_name:
                final_binary = sidecars_dir / binary_name
                shutil.move(expected_binary, final_binary)
                expected_binary = final_binary
            
            print(f"✅ Binary created: {expected_binary}")
            
            # Test the binary
            print("🧪 Testing binary...")
            try:
                test_result = subprocess.run([str(expected_binary), "--version"], 
                                           capture_output=True, text=True, timeout=30)
                if test_result.returncode == 0:
                    print(f"✅ Binary test passed: {test_result.stdout.strip()}")
                else:
                    print(f"⚠️ Binary test failed (return code {test_result.returncode})")
                    print(f"STDERR: {test_result.stderr}")
            except subprocess.TimeoutExpired:
                print("⚠️ Binary test timed out")
            except Exception as e:
                print(f"⚠️ Binary test error: {e}")
            
            return 0
        else:
            print(f"❌ Binary not found at expected location: {expected_binary}")
            print("Contents of sidecars directory:")
            for item in sidecars_dir.iterdir():
                print(f"  - {item.name}")
            return 1
            
    finally:
        # Clean up temporary entry script
        try:
            os.unlink(entry_script)
        except:
            pass
        
        # Clean up build artifacts
        build_dir = sidecars_dir / "build"
        spec_dir = sidecars_dir / "spec"
        for cleanup_dir in [build_dir, spec_dir]:
            if cleanup_dir.exists():
                shutil.rmtree(cleanup_dir, ignore_errors=True)

if __name__ == "__main__":
    sys.exit(main())