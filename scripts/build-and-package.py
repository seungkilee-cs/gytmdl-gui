#!/usr/bin/env python3
"""
Comprehensive build and packaging script for gytmdl-gui.
This script handles the complete build process including:
- Building sidecar binaries
- Building the Tauri application
- Creating platform-specific installers
- Code signing (when configured)
"""

import os
import sys
import subprocess
import platform
import shutil
import json
import argparse
from pathlib import Path
from typing import Dict, List, Optional, Tuple

class BuildError(Exception):
    """Custom exception for build errors."""
    pass

class PackagingPipeline:
    """Main class for handling the complete build and packaging pipeline."""
    
    def __init__(self, project_root: Path, config: Dict):
        self.project_root = Path(project_root).resolve()
        self.config = config
        self.platform_info = self._get_platform_info()
        self.build_dir = self.project_root / "target" / "release"
        self.dist_dir = self.project_root / "dist"
        
        # Ensure directories exist
        self.dist_dir.mkdir(parents=True, exist_ok=True)
        
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
            raise BuildError(f"Unsupported platform: {system}")
    
    def check_dependencies(self) -> bool:
        """Check if all required build dependencies are available."""
        print("🔍 Checking build dependencies...")
        
        required_tools = {
            "node": "Node.js is required for frontend build",
            "npm": "npm is required for dependency management", 
            "cargo": "Rust/Cargo is required for Tauri build",
            "python3": "Python 3 is required for sidecar builds"
        }
        
        missing_tools = []
        
        for tool, description in required_tools.items():
            try:
                subprocess.run([tool, "--version"], 
                             capture_output=True, check=True)
                print(f"  ✓ {tool} found")
            except (subprocess.CalledProcessError, FileNotFoundError):
                print(f"  ✗ {tool} not found - {description}")
                missing_tools.append(tool)
        
        # Platform-specific checks
        if self.platform_info["os"] == "windows":
            # Check for Windows-specific tools
            if self.config.get("code_signing", {}).get("enabled", False):
                try:
                    subprocess.run(["signtool"], capture_output=True, check=True)
                    print("  ✓ signtool found (code signing available)")
                except (subprocess.CalledProcessError, FileNotFoundError):
                    print("  ⚠ signtool not found (code signing disabled)")
        
        elif self.platform_info["os"] == "macos":
            # Check for macOS-specific tools
            if self.config.get("code_signing", {}).get("enabled", False):
                try:
                    subprocess.run(["codesign", "--version"], 
                                 capture_output=True, check=True)
                    print("  ✓ codesign found")
                except (subprocess.CalledProcessError, FileNotFoundError):
                    print("  ⚠ codesign not found (code signing disabled)")
        
        if missing_tools:
            print(f"\n❌ Missing required tools: {', '.join(missing_tools)}")
            return False
        
        print("✅ All dependencies satisfied")
        return True
    
    def build_sidecar_binaries(self) -> bool:
        """Build gytmdl sidecar binaries."""
        print("🔨 Building sidecar binaries...")
        
        build_script = self.project_root / "build-scripts" / "build-sidecars.py"
        if not build_script.exists():
            print("❌ Sidecar build script not found")
            return False
        
        try:
            cmd = [
                sys.executable, str(build_script),
                "--gytmdl-src", str(self.project_root.parent / "gytmdl"),
                "--output-dir", str(self.project_root / "src-tauri" / "sidecars")
            ]
            
            result = subprocess.run(cmd, cwd=self.project_root, 
                                  capture_output=True, text=True)
            
            if result.returncode != 0:
                print(f"❌ Sidecar build failed:")
                print(f"STDOUT: {result.stdout}")
                print(f"STDERR: {result.stderr}")
                return False
            
            print("✅ Sidecar binaries built successfully")
            return True
            
        except Exception as e:
            print(f"❌ Sidecar build error: {e}")
            return False
    
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
        """Build the Tauri application."""
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
    
    def sign_binaries(self) -> bool:
        """Sign binaries if code signing is enabled."""
        if not self.config.get("code_signing", {}).get("enabled", False):
            print("⏭ Code signing disabled, skipping...")
            return True
        
        print("🔐 Signing binaries...")
        
        signing_config = self.config["code_signing"]
        
        if self.platform_info["os"] == "windows":
            return self._sign_windows_binaries(signing_config)
        elif self.platform_info["os"] == "macos":
            return self._sign_macos_binaries(signing_config)
        else:
            print("⏭ Code signing not supported on this platform")
            return True
    
    def _sign_windows_binaries(self, config: Dict) -> bool:
        """Sign Windows binaries using signtool."""
        try:
            # Find the built executable
            exe_path = self.build_dir / "gytmdl-gui.exe"
            if not exe_path.exists():
                print(f"❌ Executable not found: {exe_path}")
                return False
            
            cmd = [
                "signtool", "sign",
                "/f", config["certificate_path"],
                "/p", config["certificate_password"],
                "/t", config.get("timestamp_url", "http://timestamp.digicert.com"),
                "/d", "gytmdl GUI",
                str(exe_path)
            ]
            
            subprocess.run(cmd, check=True)
            print("✅ Windows binary signed successfully")
            return True
            
        except subprocess.CalledProcessError as e:
            print(f"❌ Windows signing failed: {e}")
            return False
    
    def _sign_macos_binaries(self, config: Dict) -> bool:
        """Sign macOS binaries using codesign."""
        try:
            # Find the built app bundle
            app_path = self.build_dir / "bundle" / "macos" / "gytmdl-gui.app"
            if not app_path.exists():
                print(f"❌ App bundle not found: {app_path}")
                return False
            
            # Sign the app bundle
            cmd = [
                "codesign",
                "--sign", config["signing_identity"],
                "--force",
                "--options", "runtime",
                "--entitlements", str(self.project_root / "src-tauri" / "entitlements.plist"),
                "--deep",
                str(app_path)
            ]
            
            subprocess.run(cmd, check=True)
            
            # Notarize if configured
            if config.get("notarize", False):
                return self._notarize_macos_app(app_path, config)
            
            print("✅ macOS binary signed successfully")
            return True
            
        except subprocess.CalledProcessError as e:
            print(f"❌ macOS signing failed: {e}")
            return False
    
    def _notarize_macos_app(self, app_path: Path, config: Dict) -> bool:
        """Notarize macOS app with Apple."""
        print("📋 Notarizing macOS app...")
        
        try:
            # Create zip for notarization
            zip_path = app_path.parent / f"{app_path.stem}.zip"
            subprocess.run([
                "ditto", "-c", "-k", "--keepParent",
                str(app_path), str(zip_path)
            ], check=True)
            
            # Submit for notarization
            cmd = [
                "xcrun", "notarytool", "submit",
                str(zip_path),
                "--apple-id", config["apple_id"],
                "--password", config["app_password"],
                "--team-id", config["team_id"],
                "--wait"
            ]
            
            subprocess.run(cmd, check=True)
            
            # Staple the notarization
            subprocess.run([
                "xcrun", "stapler", "staple", str(app_path)
            ], check=True)
            
            print("✅ macOS app notarized successfully")
            return True
            
        except subprocess.CalledProcessError as e:
            print(f"❌ macOS notarization failed: {e}")
            return False
    
    def create_installers(self) -> List[Path]:
        """Create platform-specific installers."""
        print("📦 Creating installers...")
        
        installers = []
        
        for format_name in self.platform_info["installer_formats"]:
            installer_path = self._create_installer(format_name)
            if installer_path:
                installers.append(installer_path)
        
        return installers
    
    def _create_installer(self, format_name: str) -> Optional[Path]:
        """Create a specific installer format."""
        print(f"  📦 Creating {format_name} installer...")
        
        try:
            if format_name == "msi":
                return self._create_msi_installer()
            elif format_name == "nsis":
                return self._create_nsis_installer()
            elif format_name == "dmg":
                return self._create_dmg_installer()
            elif format_name == "deb":
                return self._create_deb_installer()
            elif format_name == "rpm":
                return self._create_rpm_installer()
            elif format_name == "appimage":
                return self._create_appimage_installer()
            else:
                print(f"    ⚠ Unsupported installer format: {format_name}")
                return None
                
        except Exception as e:
            print(f"    ❌ Failed to create {format_name} installer: {e}")
            return None
    
    def _create_msi_installer(self) -> Optional[Path]:
        """Create Windows MSI installer."""
        # MSI creation is handled by Tauri build process
        msi_path = self.build_dir / "bundle" / "msi" / "gytmdl-gui_0.1.0_x64_en-US.msi"
        if msi_path.exists():
            dest_path = self.dist_dir / f"gytmdl-gui-{self.platform_info['target']}.msi"
            shutil.copy2(msi_path, dest_path)
            print(f"    ✅ MSI installer created: {dest_path}")
            return dest_path
        return None
    
    def _create_dmg_installer(self) -> Optional[Path]:
        """Create macOS DMG installer."""
        # DMG creation is handled by Tauri build process
        dmg_path = self.build_dir / "bundle" / "dmg" / "gytmdl-gui_0.1.0_x64.dmg"
        if dmg_path.exists():
            dest_path = self.dist_dir / f"gytmdl-gui-{self.platform_info['target']}.dmg"
            shutil.copy2(dmg_path, dest_path)
            print(f"    ✅ DMG installer created: {dest_path}")
            return dest_path
        return None
    
    def _create_deb_installer(self) -> Optional[Path]:
        """Create Debian package."""
        # DEB creation is handled by Tauri build process
        deb_path = self.build_dir / "bundle" / "deb" / "gytmdl-gui_0.1.0_amd64.deb"
        if deb_path.exists():
            dest_path = self.dist_dir / f"gytmdl-gui-{self.platform_info['target']}.deb"
            shutil.copy2(deb_path, dest_path)
            print(f"    ✅ DEB package created: {dest_path}")
            return dest_path
        return None
    
    def _create_rpm_installer(self) -> Optional[Path]:
        """Create RPM package."""
        # RPM creation is handled by Tauri build process
        rpm_path = self.build_dir / "bundle" / "rpm" / "gytmdl-gui-0.1.0-1.x86_64.rpm"
        if rpm_path.exists():
            dest_path = self.dist_dir / f"gytmdl-gui-{self.platform_info['target']}.rpm"
            shutil.copy2(rpm_path, dest_path)
            print(f"    ✅ RPM package created: {dest_path}")
            return dest_path
        return None
    
    def _create_appimage_installer(self) -> Optional[Path]:
        """Create AppImage."""
        # AppImage creation is handled by Tauri build process
        appimage_path = self.build_dir / "bundle" / "appimage" / "gytmdl-gui_0.1.0_amd64.AppImage"
        if appimage_path.exists():
            dest_path = self.dist_dir / f"gytmdl-gui-{self.platform_info['target']}.AppImage"
            shutil.copy2(appimage_path, dest_path)
            print(f"    ✅ AppImage created: {dest_path}")
            return dest_path
        return None
    
    def _create_nsis_installer(self) -> Optional[Path]:
        """Create NSIS installer."""
        # NSIS creation is handled by Tauri build process
        nsis_path = self.build_dir / "bundle" / "nsis" / "gytmdl-gui_0.1.0_x64-setup.exe"
        if nsis_path.exists():
            dest_path = self.dist_dir / f"gytmdl-gui-{self.platform_info['target']}-setup.exe"
            shutil.copy2(nsis_path, dest_path)
            print(f"    ✅ NSIS installer created: {dest_path}")
            return dest_path
        return None
    
    def generate_checksums(self, files: List[Path]) -> Path:
        """Generate checksum file for all installers."""
        print("🔐 Generating checksums...")
        
        checksums_file = self.dist_dir / "checksums.txt"
        
        with open(checksums_file, "w") as f:
            for file_path in files:
                if file_path.exists():
                    # Calculate SHA256
                    import hashlib
                    sha256_hash = hashlib.sha256()
                    with open(file_path, "rb") as binary_file:
                        for chunk in iter(lambda: binary_file.read(4096), b""):
                            sha256_hash.update(chunk)
                    
                    checksum = sha256_hash.hexdigest()
                    f.write(f"{checksum}  {file_path.name}\n")
        
        print(f"✅ Checksums generated: {checksums_file}")
        return checksums_file
    
    def run_full_pipeline(self) -> bool:
        """Run the complete build and packaging pipeline."""
        print("🚀 Starting full build and packaging pipeline...")
        print(f"Platform: {self.platform_info['os']} {self.platform_info['arch']}")
        print(f"Target: {self.platform_info['target']}")
        
        steps = [
            ("Check dependencies", self.check_dependencies),
            ("Build sidecar binaries", self.build_sidecar_binaries),
            ("Build frontend", self.build_frontend),
            ("Build Tauri app", self.build_tauri_app),
            ("Sign binaries", self.sign_binaries),
        ]
        
        for step_name, step_func in steps:
            print(f"\n📋 Step: {step_name}")
            if not step_func():
                print(f"❌ Pipeline failed at step: {step_name}")
                return False
        
        # Create installers
        print(f"\n📋 Step: Create installers")
        installers = self.create_installers()
        
        if not installers:
            print("⚠ No installers were created")
        else:
            # Generate checksums
            self.generate_checksums(installers)
        
        print("\n🎉 Build and packaging pipeline completed successfully!")
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
    parser = argparse.ArgumentParser(description="Build and package gytmdl-gui")
    parser.add_argument("--config", type=Path, 
                       default=Path("build-config.json"),
                       help="Build configuration file")
    parser.add_argument("--project-root", type=Path,
                       default=Path(__file__).parent.parent,
                       help="Project root directory")
    
    args = parser.parse_args()
    
    # Load configuration
    config = load_config(args.config)
    
    # Create and run pipeline
    pipeline = PackagingPipeline(args.project_root, config)
    success = pipeline.run_full_pipeline()
    
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()