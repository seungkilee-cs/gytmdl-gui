#!/usr/bin/env python3
"""
Build script for creating gytmdl sidecar binaries for all supported platforms.
This script uses PyInstaller to create standalone executables that can be bundled with the Tauri app.
"""

import os
import sys
import subprocess
import shutil
import platform
import hashlib
from pathlib import Path
from typing import Dict, List, Optional

class SidecarBuilder:
    """Handles building and packaging gytmdl sidecar binaries."""
    
    def __init__(self, gytmdl_src_path: Path, output_dir: Path):
        self.gytmdl_src = Path(gytmdl_src_path).resolve()
        self.output_dir = Path(output_dir).resolve()
        self.build_dir = self.output_dir / "build"
        self.dist_dir = self.output_dir / "dist"
        self.spec_file = Path(__file__).parent / "pyinstaller-config.spec"
        
        # Ensure directories exist
        self.output_dir.mkdir(parents=True, exist_ok=True)
        self.build_dir.mkdir(parents=True, exist_ok=True)
        self.dist_dir.mkdir(parents=True, exist_ok=True)
        
    def get_platform_info(self) -> Dict[str, str]:
        """Get current platform information for binary naming."""
        system = platform.system().lower()
        machine = platform.machine().lower()
        
        if system == "windows":
            if "64" in str(sys.maxsize):
                return {
                    "os": "windows",
                    "arch": "x86_64",
                    "target": "x86_64-pc-windows-msvc",
                    "extension": ".exe"
                }
            else:
                return {
                    "os": "windows", 
                    "arch": "i686",
                    "target": "i686-pc-windows-msvc",
                    "extension": ".exe"
                }
        elif system == "darwin":
            if machine == "arm64":
                return {
                    "os": "macos",
                    "arch": "aarch64", 
                    "target": "aarch64-apple-darwin",
                    "extension": ""
                }
            else:
                return {
                    "os": "macos",
                    "arch": "x86_64",
                    "target": "x86_64-apple-darwin", 
                    "extension": ""
                }
        elif system == "linux":
            if machine == "x86_64":
                return {
                    "os": "linux",
                    "arch": "x86_64",
                    "target": "x86_64-unknown-linux-gnu",
                    "extension": ""
                }
            elif machine == "aarch64":
                return {
                    "os": "linux",
                    "arch": "aarch64", 
                    "target": "aarch64-unknown-linux-gnu",
                    "extension": ""
                }
            else:
                return {
                    "os": "linux",
                    "arch": "unknown",
                    "target": "unknown-linux-gnu", 
                    "extension": ""
                }
        else:
            raise ValueError(f"Unsupported platform: {system}")
    
    def check_dependencies(self) -> bool:
        """Check if all required dependencies are available."""
        try:
            # Check if PyInstaller is available
            subprocess.run(["pyinstaller", "--version"], 
                         capture_output=True, check=True)
            print("✓ PyInstaller is available")
            
            # Check if gytmdl source exists
            if not self.gytmdl_src.exists():
                print(f"✗ gytmdl source not found at {self.gytmdl_src}")
                return False
            print(f"✓ gytmdl source found at {self.gytmdl_src}")
            
            # Check if gytmdl can be imported
            sys.path.insert(0, str(self.gytmdl_src))
            try:
                import gytmdl
                print("✓ gytmdl module can be imported")
            except ImportError as e:
                print(f"✗ Cannot import gytmdl: {e}")
                return False
            
            return True
            
        except subprocess.CalledProcessError:
            print("✗ PyInstaller not found. Install with: pip install pyinstaller")
            return False
        except Exception as e:
            print(f"✗ Dependency check failed: {e}")
            return False
    
    def install_gytmdl_dependencies(self) -> bool:
        """Install gytmdl dependencies in the current environment."""
        try:
            requirements_file = self.gytmdl_src / "requirements.txt"
            if requirements_file.exists():
                print("Installing gytmdl dependencies...")
                subprocess.run([
                    sys.executable, "-m", "pip", "install", "-r", str(requirements_file)
                ], check=True)
                print("✓ gytmdl dependencies installed")
            else:
                print("No requirements.txt found, attempting to install gytmdl directly...")
                subprocess.run([
                    sys.executable, "-m", "pip", "install", "-e", str(self.gytmdl_src)
                ], check=True)
                print("✓ gytmdl installed in development mode")
            
            return True
        except subprocess.CalledProcessError as e:
            print(f"✗ Failed to install dependencies: {e}")
            return False
    
    def build_binary(self) -> Optional[Path]:
        """Build the gytmdl binary for the current platform."""
        platform_info = self.get_platform_info()
        binary_name = f"gytmdl-{platform_info['target']}{platform_info['extension']}"
        
        print(f"Building binary for {platform_info['os']} {platform_info['arch']}...")
        
        try:
            # Run PyInstaller with just the spec file
            cmd = [
                "pyinstaller",
                "--clean",
                str(self.spec_file)
            ]
            
            print(f"Running: {' '.join(cmd)}")
            result = subprocess.run(cmd, cwd=self.gytmdl_src, capture_output=True, text=True)
            
            if result.returncode != 0:
                print(f"✗ PyInstaller failed:")
                print(f"STDOUT: {result.stdout}")
                print(f"STDERR: {result.stderr}")
                return None
            
            # Find the generated binary
            expected_binary = self.dist_dir / binary_name
            if not expected_binary.exists():
                # PyInstaller might have created it without the full target name
                simple_name = f"gytmdl{platform_info['extension']}"
                simple_binary = self.dist_dir / simple_name
                if simple_binary.exists():
                    # Rename to the expected name
                    simple_binary.rename(expected_binary)
                else:
                    print(f"✗ Binary not found at {expected_binary} or {simple_binary}")
                    return None
            
            print(f"✓ Binary built successfully: {expected_binary}")
            return expected_binary
            
        except Exception as e:
            print(f"✗ Build failed: {e}")
            return None
    
    def validate_binary(self, binary_path: Path) -> bool:
        """Validate that the built binary works correctly."""
        try:
            print(f"Validating binary: {binary_path}")
            
            # Test version command
            result = subprocess.run([str(binary_path), "--version"], 
                                  capture_output=True, text=True, timeout=30)
            
            if result.returncode == 0:
                version_output = result.stdout.strip()
                print(f"✓ Binary validation successful. Version: {version_output}")
                return True
            else:
                print(f"✗ Binary validation failed. Return code: {result.returncode}")
                print(f"STDERR: {result.stderr}")
                return False
                
        except subprocess.TimeoutExpired:
            print("✗ Binary validation timed out")
            return False
        except Exception as e:
            print(f"✗ Binary validation error: {e}")
            return False
    
    def calculate_checksum(self, binary_path: Path) -> str:
        """Calculate SHA256 checksum of the binary."""
        sha256_hash = hashlib.sha256()
        with open(binary_path, "rb") as f:
            for chunk in iter(lambda: f.read(4096), b""):
                sha256_hash.update(chunk)
        return sha256_hash.hexdigest()
    
    def create_manifest(self, binary_path: Path) -> Path:
        """Create a manifest file with binary information."""
        platform_info = self.get_platform_info()
        checksum = self.calculate_checksum(binary_path)
        
        manifest = {
            "binary_name": binary_path.name,
            "platform": platform_info,
            "size_bytes": binary_path.stat().st_size,
            "sha256": checksum,
            "build_timestamp": subprocess.run(
                ["date", "-u", "+%Y-%m-%dT%H:%M:%SZ"], 
                capture_output=True, text=True
            ).stdout.strip() if sys.platform != "win32" else "unknown"
        }
        
        manifest_path = binary_path.with_suffix(".json")
        import json
        with open(manifest_path, "w") as f:
            json.dump(manifest, f, indent=2)
        
        print(f"✓ Manifest created: {manifest_path}")
        return manifest_path
    
    def build(self) -> bool:
        """Main build process."""
        print("=== gytmdl Sidecar Binary Builder ===")
        
        # Check dependencies
        if not self.check_dependencies():
            return False
        
        # Install gytmdl dependencies
        if not self.install_gytmdl_dependencies():
            return False
        
        # Build binary
        binary_path = self.build_binary()
        if not binary_path:
            return False
        
        # Validate binary
        if not self.validate_binary(binary_path):
            return False
        
        # Create manifest
        self.create_manifest(binary_path)
        
        print(f"\n✓ Build completed successfully!")
        print(f"Binary: {binary_path}")
        print(f"Size: {binary_path.stat().st_size / 1024 / 1024:.1f} MB")
        
        return True


def main():
    """Main entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description="Build gytmdl sidecar binaries")
    parser.add_argument("--gytmdl-src", type=Path, 
                       default=Path(__file__).parent.parent.parent / "gytmdl",
                       help="Path to gytmdl source directory")
    parser.add_argument("--output-dir", type=Path,
                       default=Path(__file__).parent.parent / "src-tauri" / "sidecars",
                       help="Output directory for built binaries")
    
    args = parser.parse_args()
    
    builder = SidecarBuilder(args.gytmdl_src, args.output_dir)
    success = builder.build()
    
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()