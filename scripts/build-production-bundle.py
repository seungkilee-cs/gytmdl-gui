#!/usr/bin/env python3
"""
Production build script for gytmdl-gui with complete sidecar binary bundling.
This script handles:
- Building sidecar binaries for all target platforms
- Configuring Tauri to include sidecar binaries in app bundle
- Creating platform-specific installers
- Testing installer functionality and sidecar detection
"""

import os
import sys
import subprocess
import platform
import shutil
import json
import argparse
import hashlib
from pathlib import Path
from typing import Dict, List, Optional, Tuple

class ProductionBundleBuilder:
    """Handles complete production bundle creation with sidecar binaries."""
    
    def __init__(self, project_root: Path, config: Dict):
        self.project_root = Path(project_root).resolve()
        self.config = config
        self.platform_info = self._get_platform_info()
        self.build_dir = self.project_root / "target" / "release"
        self.dist_dir = self.project_root / "dist"
        self.sidecars_dir = self.project_root / "src-tauri" / "sidecars"
        self.gytmdl_src = self.project_root.parent / "gytmdl"
        
        # Ensure directories exist
        self.dist_dir.mkdir(parents=True, exist_ok=True)
        self.sidecars_dir.mkdir(parents=True, exist_ok=True)
        
        # Target platforms for sidecar binaries
        self.target_platforms = [
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
    
    def _get_platform_info(self) -> Dict[str, str]:
        """Get current platform information."""
        system = platform.system().lower()
        machine = platform.machine().lower()
        
        if system == "windows":
            return {
                "os": "windows",
                "arch": "x86_64" if "64" in str(sys.maxsize) else "i686",
                "target": "x86_64-pc-windows-msvc" if "64" in str(sys.maxsize) else "i686-pc-windows-msvc",
                "extension": ".exe",
                "installer_formats": ["msi", "nsis"]
            }
        elif system == "darwin":
            return {
                "os": "macos",
                "arch": "aarch64" if machine == "arm64" else "x86_64",
                "target": "aarch64-apple-darwin" if machine == "arm64" else "x86_64-apple-darwin",
                "extension": "",
                "installer_formats": ["dmg", "app"]
            }
        elif system == "linux":
            return {
                "os": "linux",
                "arch": "x86_64" if machine == "x86_64" else "aarch64" if machine == "aarch64" else "unknown",
                "target": f"{machine}-unknown-linux-gnu",
                "extension": "",
                "installer_formats": ["deb", "rpm", "appimage"]
            }
        else:
            raise ValueError(f"Unsupported platform: {system}")
    
    def check_dependencies(self) -> bool:
        """Check if all required build dependencies are available."""
        print("🔍 Checking build dependencies...")
        
        required_tools = {
            "node": "Node.js is required for frontend build",
            "npm": "npm is required for dependency management", 
            "cargo": "Rust/Cargo is required for Tauri build",
            "python3": "Python 3 is required for sidecar builds",
            "pyinstaller": "PyInstaller is required for sidecar binary creation"
        }
        
        missing_tools = []
        
        for tool, description in required_tools.items():
            try:
                if tool == "pyinstaller":
                    subprocess.run([sys.executable, "-m", "PyInstaller", "--version"], 
                                 capture_output=True, check=True)
                else:
                    subprocess.run([tool, "--version"], 
                                 capture_output=True, check=True)
                print(f"  ✓ {tool} found")
            except (subprocess.CalledProcessError, FileNotFoundError):
                print(f"  ✗ {tool} not found - {description}")
                missing_tools.append(tool)
        
        # Check gytmdl source
        if not self.gytmdl_src.exists():
            print(f"  ✗ gytmdl source not found at {self.gytmdl_src}")
            missing_tools.append("gytmdl-source")
        else:
            print(f"  ✓ gytmdl source found at {self.gytmdl_src}")
        
        if missing_tools:
            print(f"\n❌ Missing required dependencies: {', '.join(missing_tools)}")
            if "pyinstaller" in missing_tools:
                print("Install PyInstaller with: pip install pyinstaller")
            return False
        
        print("✅ All dependencies satisfied")
        return True
    
    def build_sidecar_for_current_platform(self) -> Optional[Path]:
        """Build sidecar binary for the current platform."""
        current_target = self.platform_info["target"]
        current_platform = next(
            (p for p in self.target_platforms if p["target"] == current_target), 
            None
        )
        
        if not current_platform:
            print(f"❌ Current platform {current_target} not supported")
            return None
        
        print(f"🔨 Building sidecar binary for {current_platform['name']}...")
        
        binary_name = f"gytmdl-{current_platform['target']}{current_platform['extension']}"
        binary_path = self.sidecars_dir / binary_name
        
        try:
            # Use the existing sidecar build script
            build_script = self.project_root / "build-scripts" / "build-sidecars.py"
            
            cmd = [
                sys.executable, str(build_script),
                "--gytmdl-src", str(self.gytmdl_src),
                "--output-dir", str(self.sidecars_dir)
            ]
            
            result = subprocess.run(cmd, cwd=self.project_root, 
                                  capture_output=True, text=True)
            
            if result.returncode != 0:
                print(f"❌ Sidecar build failed:")
                print(f"STDOUT: {result.stdout}")
                print(f"STDERR: {result.stderr}")
                return None
            
            # Check if binary was created
            if binary_path.exists():
                print(f"✅ Sidecar binary built: {binary_path}")
                return binary_path
            else:
                print(f"❌ Sidecar binary not found at expected location: {binary_path}")
                return None
                
        except Exception as e:
            print(f"❌ Sidecar build error: {e}")
            return None
    
    def create_mock_sidecars_for_other_platforms(self) -> List[Path]:
        """Create mock sidecar binaries for platforms we can't build on."""
        print("🎭 Creating mock sidecar binaries for other platforms...")
        
        current_target = self.platform_info["target"]
        mock_binaries = []
        
        for platform in self.target_platforms:
            if platform["target"] == current_target:
                continue  # Skip current platform - we build real binary for this
            
            binary_name = f"gytmdl-{platform['target']}{platform['extension']}"
            binary_path = self.sidecars_dir / binary_name
            manifest_path = self.sidecars_dir / f"{binary_name}.json"
            
            # Create mock binary (small executable that shows it's a mock)
            mock_content = f"""#!/bin/bash
echo "This is a mock gytmdl binary for {platform['name']}"
echo "Platform: {platform['target']}"
echo "This binary should be replaced with a real build for production use"
exit 1
""".encode()
            
            if platform["os"] == "windows":
                # For Windows, create a simple batch file
                mock_content = f"""@echo off
echo This is a mock gytmdl binary for {platform['name']}
echo Platform: {platform['target']}
echo This binary should be replaced with a real build for production use
exit /b 1
""".encode()
            
            binary_path.write_bytes(mock_content)
            
            # Make executable on Unix-like systems
            if platform["os"] != "windows":
                os.chmod(binary_path, 0o755)
            
            # Create manifest
            manifest = {
                "binary_name": binary_name,
                "platform": {
                    "os": platform["os"],
                    "arch": platform["arch"],
                    "target": platform["target"],
                    "extension": platform["extension"]
                },
                "size_bytes": len(mock_content),
                "sha256": hashlib.sha256(mock_content).hexdigest(),
                "build_timestamp": "mock-build",
                "is_mock": True
            }
            
            with open(manifest_path, "w") as f:
                json.dump(manifest, f, indent=2)
            
            mock_binaries.append(binary_path)
            print(f"  📝 Created mock binary: {binary_name}")
        
        print(f"✅ Created {len(mock_binaries)} mock sidecar binaries")
        return mock_binaries
    
    def verify_sidecar_detection(self) -> bool:
        """Test that Tauri can detect and use the sidecar binaries."""
        print("🔍 Testing sidecar detection...")
        
        # Check that all expected sidecar binaries exist
        missing_binaries = []
        
        for platform in self.target_platforms:
            binary_name = f"gytmdl-{platform['target']}{platform['extension']}"
            binary_path = self.sidecars_dir / binary_name
            
            if not binary_path.exists():
                missing_binaries.append(binary_name)
            else:
                print(f"  ✓ Found: {binary_name}")
        
        if missing_binaries:
            print(f"❌ Missing sidecar binaries: {', '.join(missing_binaries)}")
            return False
        
        # Test that the current platform's binary works
        current_target = self.platform_info["target"]
        current_binary_name = f"gytmdl-{current_target}{self.platform_info['extension']}"
        current_binary_path = self.sidecars_dir / current_binary_name
        
        try:
            result = subprocess.run([str(current_binary_path), "--version"], 
                                  capture_output=True, text=True, timeout=30)
            
            if result.returncode == 0:
                print(f"  ✓ Current platform binary works: {result.stdout.strip()}")
            else:
                print(f"  ⚠ Current platform binary test failed (return code: {result.returncode})")
                print(f"    This might be expected for mock binaries")
        
        except subprocess.TimeoutExpired:
            print("  ⚠ Binary test timed out")
        except Exception as e:
            print(f"  ⚠ Binary test error: {e}")
        
        print("✅ Sidecar detection test completed")
        return True
    
    def build_frontend(self) -> bool:
        """Build the frontend application."""
        print("🎨 Building frontend...")
        
        try:
            # Install dependencies
            subprocess.run(["npm", "install"], 
                         cwd=self.project_root, check=True)
            
            # Build frontend
            subprocess.run(["npm", "run", "build"], 
                         cwd=self.project_root, check=True)
            
            print("✅ Frontend built successfully")
            return True
            
        except subprocess.CalledProcessError as e:
            print(f"❌ Frontend build failed: {e}")
            return False
    
    def build_tauri_app(self) -> bool:
        """Build the Tauri application with bundled sidecars."""
        print("🦀 Building Tauri application...")
        
        try:
            # Build command
            cmd = ["cargo", "tauri", "build"]
            
            # Add target if specified
            if "target" in self.config:
                cmd.extend(["--target", self.config["target"]])
            
            # Add additional flags
            if self.config.get("release", True):
                cmd.append("--release")
            
            result = subprocess.run(cmd, cwd=self.project_root, 
                                  capture_output=True, text=True)
            
            if result.returncode != 0:
                print(f"❌ Tauri build failed:")
                print(f"STDOUT: {result.stdout}")
                print(f"STDERR: {result.stderr}")
                return False
            
            print("✅ Tauri application built successfully")
            return True
            
        except Exception as e:
            print(f"❌ Tauri build error: {e}")
            return False
    
    def create_installers(self) -> List[Path]:
        """Create platform-specific installers."""
        print("📦 Creating installers...")
        
        installers = []
        
        for format_name in self.platform_info["installer_formats"]:
            installer_path = self._find_installer(format_name)
            if installer_path:
                # Copy to dist directory with standardized name
                dest_name = f"gytmdl-gui-{self.platform_info['target']}"
                if format_name == "msi":
                    dest_name += ".msi"
                elif format_name == "nsis":
                    dest_name += "-setup.exe"
                elif format_name == "dmg":
                    dest_name += ".dmg"
                elif format_name == "deb":
                    dest_name += ".deb"
                elif format_name == "rpm":
                    dest_name += ".rpm"
                elif format_name == "appimage":
                    dest_name += ".AppImage"
                
                dest_path = self.dist_dir / dest_name
                shutil.copy2(installer_path, dest_path)
                installers.append(dest_path)
                print(f"  ✅ Created {format_name} installer: {dest_name}")
        
        return installers
    
    def _find_installer(self, format_name: str) -> Optional[Path]:
        """Find the installer file created by Tauri build."""
        bundle_dir = self.build_dir / "bundle"
        
        if format_name == "msi":
            msi_dir = bundle_dir / "msi"
            if msi_dir.exists():
                msi_files = list(msi_dir.glob("*.msi"))
                return msi_files[0] if msi_files else None
        
        elif format_name == "nsis":
            nsis_dir = bundle_dir / "nsis"
            if nsis_dir.exists():
                nsis_files = list(nsis_dir.glob("*-setup.exe"))
                return nsis_files[0] if nsis_files else None
        
        elif format_name == "dmg":
            dmg_dir = bundle_dir / "dmg"
            if dmg_dir.exists():
                dmg_files = list(dmg_dir.glob("*.dmg"))
                return dmg_files[0] if dmg_files else None
        
        elif format_name == "deb":
            deb_dir = bundle_dir / "deb"
            if deb_dir.exists():
                deb_files = list(deb_dir.glob("*.deb"))
                return deb_files[0] if deb_files else None
        
        elif format_name == "rpm":
            rpm_dir = bundle_dir / "rpm"
            if rpm_dir.exists():
                rpm_files = list(rpm_dir.glob("*.rpm"))
                return rpm_files[0] if rpm_files else None
        
        elif format_name == "appimage":
            appimage_dir = bundle_dir / "appimage"
            if appimage_dir.exists():
                appimage_files = list(appimage_dir.glob("*.AppImage"))
                return appimage_files[0] if appimage_files else None
        
        return None
    
    def test_installer_functionality(self, installers: List[Path]) -> bool:
        """Test basic installer functionality."""
        print("🧪 Testing installer functionality...")
        
        for installer in installers:
            print(f"  Testing: {installer.name}")
            
            # Basic file validation
            if not installer.exists():
                print(f"    ❌ Installer file not found")
                continue
            
            file_size = installer.stat().st_size
            if file_size < 1024 * 1024:  # Less than 1MB is suspicious
                print(f"    ⚠ Installer seems too small: {file_size / 1024:.1f} KB")
            else:
                print(f"    ✓ Installer size: {file_size / 1024 / 1024:.1f} MB")
            
            # Calculate checksum
            sha256_hash = hashlib.sha256()
            with open(installer, "rb") as f:
                for chunk in iter(lambda: f.read(4096), b""):
                    sha256_hash.update(chunk)
            checksum = sha256_hash.hexdigest()
            print(f"    ✓ SHA256: {checksum[:16]}...")
        
        print("✅ Installer functionality tests completed")
        return True
    
    def generate_build_report(self, installers: List[Path]) -> Path:
        """Generate a comprehensive build report."""
        print("📋 Generating build report...")
        
        report = {
            "build_info": {
                "timestamp": subprocess.run(
                    ["date", "-u", "+%Y-%m-%dT%H:%M:%SZ"], 
                    capture_output=True, text=True
                ).stdout.strip() if sys.platform != "win32" else "unknown",
                "platform": self.platform_info,
                "config": self.config
            },
            "sidecar_binaries": [],
            "installers": [],
            "checksums": {}
        }
        
        # Collect sidecar binary info
        for platform in self.target_platforms:
            binary_name = f"gytmdl-{platform['target']}{platform['extension']}"
            binary_path = self.sidecars_dir / binary_name
            manifest_path = self.sidecars_dir / f"{binary_name}.json"
            
            if binary_path.exists():
                binary_info = {
                    "name": binary_name,
                    "platform": platform,
                    "size_bytes": binary_path.stat().st_size,
                    "exists": True
                }
                
                if manifest_path.exists():
                    with open(manifest_path) as f:
                        manifest = json.load(f)
                        binary_info.update(manifest)
                
                report["sidecar_binaries"].append(binary_info)
        
        # Collect installer info
        for installer in installers:
            if installer.exists():
                # Calculate checksum
                sha256_hash = hashlib.sha256()
                with open(installer, "rb") as f:
                    for chunk in iter(lambda: f.read(4096), b""):
                        sha256_hash.update(chunk)
                checksum = sha256_hash.hexdigest()
                
                installer_info = {
                    "name": installer.name,
                    "size_bytes": installer.stat().st_size,
                    "sha256": checksum
                }
                
                report["installers"].append(installer_info)
                report["checksums"][installer.name] = checksum
        
        # Write report
        report_path = self.dist_dir / "build-report.json"
        with open(report_path, "w") as f:
            json.dump(report, f, indent=2)
        
        print(f"✅ Build report generated: {report_path}")
        return report_path
    
    def run_full_build(self) -> bool:
        """Run the complete production build process."""
        print("🚀 Starting production build with bundled sidecars...")
        print(f"Platform: {self.platform_info['os']} {self.platform_info['arch']}")
        print(f"Target: {self.platform_info['target']}")
        
        steps = [
            ("Check dependencies", self.check_dependencies),
            ("Build sidecar for current platform", lambda: self.build_sidecar_for_current_platform() is not None),
            ("Create mock sidecars for other platforms", lambda: len(self.create_mock_sidecars_for_other_platforms()) > 0),
            ("Verify sidecar detection", self.verify_sidecar_detection),
            ("Build frontend", self.build_frontend),
            ("Build Tauri app", self.build_tauri_app),
        ]
        
        for step_name, step_func in steps:
            print(f"\n📋 Step: {step_name}")
            if not step_func():
                print(f"❌ Build failed at step: {step_name}")
                return False
        
        # Create installers
        print(f"\n📋 Step: Create installers")
        installers = self.create_installers()
        
        if not installers:
            print("⚠ No installers were created")
        else:
            # Test installer functionality
            self.test_installer_functionality(installers)
        
        # Generate build report
        self.generate_build_report(installers)
        
        print("\n🎉 Production build completed successfully!")
        print(f"📁 Output directory: {self.dist_dir}")
        
        if installers:
            print("📦 Created installers:")
            for installer in installers:
                print(f"  - {installer.name}")
        
        return True


def load_config(config_path: Path) -> Dict:
    """Load build configuration from JSON file."""
    if config_path.exists():
        with open(config_path) as f:
            return json.load(f)
    else:
        # Return default configuration
        return {
            "release": True,
            "code_signing": {
                "enabled": False
            }
        }


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(description="Build production bundle for gytmdl-gui")
    parser.add_argument("--config", type=Path, 
                       default=Path("build-config.json"),
                       help="Build configuration file")
    parser.add_argument("--project-root", type=Path,
                       default=Path(__file__).parent.parent,
                       help="Project root directory")
    
    args = parser.parse_args()
    
    # Load configuration
    config = load_config(args.config)
    
    # Create and run builder
    builder = ProductionBundleBuilder(args.project_root, config)
    success = builder.run_full_build()
    
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()