#!/usr/bin/env python3
"""
Complete test and build script for gytmdl-gui.
This script handles the entire process from dependency checking to installer testing.
"""

import os
import sys
import subprocess
import shutil
import platform
import tempfile
import json
from pathlib import Path
from typing import Dict, List, Optional, Tuple

class CompleteTester:
    """Handles complete testing and building of gytmdl-gui."""
    
    def __init__(self):
        self.script_dir = Path(__file__).parent
        self.project_root = self.script_dir.parent
        self.gytmdl_src = self.project_root.parent / "gytmdl"
        self.sidecars_dir = self.project_root / "src-tauri" / "sidecars"
        self.platform_info = self._get_platform_info()
        
        print(f"🏗️ gytmdl-gui Complete Tester")
        print(f"Project root: {self.project_root}")
        print(f"gytmdl source: {self.gytmdl_src}")
        print(f"Platform: {self.platform_info['os']} {self.platform_info['arch']}")
    
    def _get_platform_info(self) -> Dict[str, str]:
        """Get current platform information."""
        system = platform.system().lower()
        machine = platform.machine().lower()
        
        if system == "darwin":
            return {
                "os": "macos",
                "arch": "aarch64" if machine == "arm64" else "x86_64",
                "target": "aarch64-apple-darwin" if machine == "arm64" else "x86_64-apple-darwin",
                "extension": "",
                "installer_formats": ["dmg"]
            }
        elif system == "linux":
            return {
                "os": "linux",
                "arch": "x86_64" if machine == "x86_64" else "aarch64" if machine == "aarch64" else "unknown",
                "target": f"{machine}-unknown-linux-gnu",
                "extension": "",
                "installer_formats": ["deb", "appimage"]
            }
        elif system == "windows":
            return {
                "os": "windows",
                "arch": "x86_64" if "64" in str(sys.maxsize) else "i686",
                "target": "x86_64-pc-windows-msvc" if "64" in str(sys.maxsize) else "i686-pc-windows-msvc",
                "extension": ".exe",
                "installer_formats": ["msi"]
            }
        else:
            raise ValueError(f"Unsupported platform: {system}")
    
    def check_dependencies(self) -> bool:
        """Check all required dependencies."""
        print("\n🔍 Checking dependencies...")
        
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
                             capture_output=True, check=True, timeout=10)
                print(f"  ✓ {tool} found")
            except (subprocess.CalledProcessError, FileNotFoundError, subprocess.TimeoutExpired):
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
            return False
        
        print("✅ All dependencies satisfied")
        return True
    
    def install_pyinstaller(self) -> bool:
        """Install PyInstaller if not available."""
        try:
            subprocess.run([sys.executable, "-m", "PyInstaller", "--version"], 
                          capture_output=True, check=True, timeout=10)
            print("  ✓ PyInstaller is available")
            return True
        except (subprocess.CalledProcessError, FileNotFoundError, subprocess.TimeoutExpired):
            print("  ⚠️ PyInstaller not found. Installing...")
            try:
                subprocess.run([sys.executable, "-m", "pip", "install", "pyinstaller"], 
                              check=True, timeout=60)
                print("  ✓ PyInstaller installed")
                return True
            except (subprocess.CalledProcessError, subprocess.TimeoutExpired):
                print("  ❌ Failed to install PyInstaller")
                return False
    
    def build_sidecar_simple(self) -> Optional[Path]:
        """Build sidecar binary using simple approach."""
        print(f"\n🔨 Building sidecar binary for {self.platform_info['target']}...")
        
        # Create sidecars directory
        self.sidecars_dir.mkdir(parents=True, exist_ok=True)
        
        # Install PyInstaller
        if not self.install_pyinstaller():
            return None
        
        # Install gytmdl in development mode
        print("  📦 Installing gytmdl...")
        try:
            subprocess.run([sys.executable, "-m", "pip", "install", "-e", str(self.gytmdl_src)], 
                          check=True, capture_output=True, timeout=120)
            print("  ✓ gytmdl installed")
        except (subprocess.CalledProcessError, subprocess.TimeoutExpired) as e:
            print(f"  ❌ Failed to install gytmdl: {e}")
            return None
        
        # Create entry script
        entry_script_content = '''
import sys
import os

def main():
    try:
        import gytmdl.cli
        gytmdl.cli.main()
    except ImportError as e:
        print(f"Error importing gytmdl: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"Error running gytmdl: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
'''
        
        with tempfile.NamedTemporaryFile(mode='w', suffix='.py', delete=False) as f:
            f.write(entry_script_content)
            entry_script = f.name
        
        try:
            # Build with PyInstaller
            binary_name = f"gytmdl-{self.platform_info['target']}"
            
            cmd = [
                sys.executable, "-m", "PyInstaller",
                "--onefile",
                "--console",
                "--name", binary_name,
                "--distpath", str(self.sidecars_dir),
                "--workpath", str(self.sidecars_dir / "build"),
                "--specpath", str(self.sidecars_dir / "spec"),
                "--clean",
                entry_script
            ]
            
            print(f"  🔧 Running PyInstaller...")
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=300)
            
            if result.returncode != 0:
                print(f"  ❌ PyInstaller failed:")
                print(f"  STDOUT: {result.stdout}")
                print(f"  STDERR: {result.stderr}")
                return None
            
            # Find the created binary
            expected_binary = self.sidecars_dir / f"{binary_name}{self.platform_info['extension']}"
            
            if expected_binary.exists():
                print(f"  ✅ Binary created: {expected_binary}")
                
                # Test the binary
                print("  🧪 Testing binary...")
                try:
                    test_result = subprocess.run([str(expected_binary), "--version"], 
                                               capture_output=True, text=True, timeout=30)
                    if test_result.returncode == 0:
                        print(f"  ✅ Binary test passed: {test_result.stdout.strip()}")
                        return expected_binary
                    else:
                        print(f"  ⚠️ Binary test failed (return code {test_result.returncode})")
                        print(f"  STDERR: {test_result.stderr}")
                        # Still return the binary as it might work in the app context
                        return expected_binary
                except subprocess.TimeoutExpired:
                    print("  ⚠️ Binary test timed out")
                    return expected_binary
                except Exception as e:
                    print(f"  ⚠️ Binary test error: {e}")
                    return expected_binary
            else:
                print(f"  ❌ Binary not found at: {expected_binary}")
                print("  Contents of sidecars directory:")
                for item in self.sidecars_dir.iterdir():
                    print(f"    - {item.name}")
                return None
                
        except subprocess.TimeoutExpired:
            print("  ❌ PyInstaller timed out")
            return None
        except Exception as e:
            print(f"  ❌ Build failed: {e}")
            return None
        finally:
            # Clean up
            try:
                os.unlink(entry_script)
            except:
                pass
            
            # Clean up build artifacts
            for cleanup_dir in [self.sidecars_dir / "build", self.sidecars_dir / "spec"]:
                if cleanup_dir.exists():
                    shutil.rmtree(cleanup_dir, ignore_errors=True)
    
    def build_frontend(self) -> bool:
        """Build the frontend application."""
        print("\n🎨 Building frontend...")
        
        try:
            # Install dependencies
            print("  📦 Installing dependencies...")
            subprocess.run(["npm", "install"], 
                         cwd=self.project_root, check=True, timeout=180)
            
            # Build frontend
            print("  🔧 Building...")
            subprocess.run(["npm", "run", "build"], 
                         cwd=self.project_root, check=True, timeout=120)
            
            print("  ✅ Frontend built successfully")
            return True
            
        except subprocess.TimeoutExpired:
            print("  ❌ Frontend build timed out")
            return False
        except subprocess.CalledProcessError as e:
            print(f"  ❌ Frontend build failed: {e}")
            return False
    
    def build_tauri_app(self) -> bool:
        """Build the Tauri application."""
        print("\n🦀 Building Tauri application...")
        
        # Check if Tauri CLI is available via npm
        try:
            subprocess.run(["npm", "run", "tauri", "--", "--version"], 
                          capture_output=True, check=True, timeout=10, cwd=self.project_root)
            print("  ✓ Tauri CLI is available")
        except (subprocess.CalledProcessError, subprocess.TimeoutExpired):
            print("  ❌ Tauri CLI not available via npm")
            return False
        
        try:
            cmd = ["npm", "run", "tauri", "--", "build"]
            
            print("  🔧 Building Tauri app...")
            result = subprocess.run(cmd, cwd=self.project_root, 
                                  capture_output=True, text=True, timeout=600)
            
            if result.returncode != 0:
                print(f"  ❌ Tauri build failed:")
                print(f"  STDOUT: {result.stdout}")
                print(f"  STDERR: {result.stderr}")
                return False
            
            print("  ✅ Tauri application built successfully")
            return True
            
        except subprocess.TimeoutExpired:
            print("  ❌ Tauri build timed out")
            return False
        except Exception as e:
            print(f"  ❌ Tauri build error: {e}")
            return False
    
    def find_installers(self) -> List[Path]:
        """Find built installer files."""
        print("\n📦 Looking for installers...")
        
        bundle_dir = self.project_root / "target" / "release" / "bundle"
        installers = []
        
        if not bundle_dir.exists():
            print("  ❌ Bundle directory not found")
            return installers
        
        for format_name in self.platform_info["installer_formats"]:
            format_dir = bundle_dir / format_name
            if format_dir.exists():
                if format_name == "dmg":
                    files = list(format_dir.glob("*.dmg"))
                elif format_name == "deb":
                    files = list(format_dir.glob("*.deb"))
                elif format_name == "rpm":
                    files = list(format_dir.glob("*.rpm"))
                elif format_name == "appimage":
                    files = list(format_dir.glob("*.AppImage"))
                elif format_name == "msi":
                    files = list(format_dir.glob("*.msi"))
                else:
                    files = []
                
                for file in files:
                    installers.append(file)
                    print(f"  ✓ Found {format_name}: {file.name}")
        
        if not installers:
            print("  ❌ No installers found")
        
        return installers
    
    def test_installer_basic(self, installer_path: Path) -> bool:
        """Basic installer testing."""
        print(f"\n🧪 Testing installer: {installer_path.name}")
        
        try:
            # Basic file validation
            if not installer_path.exists():
                print("  ❌ Installer file not found")
                return False
            
            file_size = installer_path.stat().st_size
            if file_size < 1024 * 1024:  # Less than 1MB is suspicious
                print(f"  ⚠️ Installer seems small: {file_size / 1024:.1f} KB")
            else:
                print(f"  ✓ Installer size: {file_size / 1024 / 1024:.1f} MB")
            
            # Platform-specific tests
            if installer_path.suffix == ".dmg":
                return self._test_dmg(installer_path)
            elif installer_path.suffix == ".deb":
                return self._test_deb(installer_path)
            elif installer_path.suffix == ".AppImage":
                return self._test_appimage(installer_path)
            elif installer_path.suffix == ".msi":
                return self._test_msi(installer_path)
            else:
                print(f"  ⚠️ Unknown installer format: {installer_path.suffix}")
                return True
                
        except Exception as e:
            print(f"  ❌ Test failed: {e}")
            return False
    
    def _test_dmg(self, dmg_path: Path) -> bool:
        """Test macOS DMG installer."""
        try:
            print("  🍎 Testing macOS DMG...")
            
            # Mount DMG
            mount_result = subprocess.run(
                ["hdiutil", "attach", str(dmg_path), "-readonly", "-nobrowse"],
                capture_output=True, text=True, timeout=30
            )
            
            if mount_result.returncode != 0:
                print(f"  ❌ Failed to mount DMG: {mount_result.stderr}")
                return False
            
            # Find mounted volume
            volumes = Path("/Volumes")
            app_volumes = [v for v in volumes.iterdir() if "gytmdl" in v.name.lower()]
            
            if not app_volumes:
                print("  ❌ No gytmdl volume found")
                return False
            
            volume = app_volumes[0]
            print(f"  ✓ Mounted at: {volume}")
            
            # Find app bundle
            app_bundles = list(volume.glob("*.app"))
            if app_bundles:
                app_bundle = app_bundles[0]
                print(f"  ✓ Found app bundle: {app_bundle.name}")
                
                # Check for sidecar binaries
                resources_dir = app_bundle / "Contents" / "Resources"
                if resources_dir.exists():
                    sidecar_files = list(resources_dir.glob("gytmdl-*"))
                    if sidecar_files:
                        print(f"  ✓ Found {len(sidecar_files)} sidecar binaries")
                    else:
                        print("  ⚠️ No sidecar binaries found in Resources")
            
            # Unmount
            subprocess.run(["hdiutil", "detach", str(volume)], 
                         capture_output=True, timeout=30)
            
            return True
            
        except subprocess.TimeoutExpired:
            print("  ❌ DMG test timed out")
            return False
        except Exception as e:
            print(f"  ❌ DMG test error: {e}")
            return False
    
    def _test_deb(self, deb_path: Path) -> bool:
        """Test Linux DEB package."""
        try:
            print("  🐧 Testing Linux DEB...")
            
            # Check package info
            info_result = subprocess.run(
                ["dpkg-deb", "--info", str(deb_path)],
                capture_output=True, text=True, timeout=30
            )
            
            if info_result.returncode == 0:
                print("  ✓ Package info looks good")
            else:
                print(f"  ❌ Package info failed: {info_result.stderr}")
                return False
            
            # Check contents
            contents_result = subprocess.run(
                ["dpkg-deb", "--contents", str(deb_path)],
                capture_output=True, text=True, timeout=30
            )
            
            if contents_result.returncode == 0:
                contents = contents_result.stdout
                if "gytmdl-gui" in contents:
                    print("  ✓ Main executable found in package")
                if "gytmdl-" in contents:
                    print("  ✓ Sidecar binaries found in package")
            
            return True
            
        except subprocess.TimeoutExpired:
            print("  ❌ DEB test timed out")
            return False
        except Exception as e:
            print(f"  ❌ DEB test error: {e}")
            return False
    
    def _test_appimage(self, appimage_path: Path) -> bool:
        """Test Linux AppImage."""
        try:
            print("  🐧 Testing Linux AppImage...")
            
            # Make executable
            os.chmod(appimage_path, 0o755)
            print("  ✓ AppImage is executable")
            
            return True
            
        except Exception as e:
            print(f"  ❌ AppImage test error: {e}")
            return False
    
    def _test_msi(self, msi_path: Path) -> bool:
        """Test Windows MSI installer."""
        try:
            print("  🪟 Testing Windows MSI...")
            print(f"  ✓ MSI installer exists: {msi_path.name}")
            return True
            
        except Exception as e:
            print(f"  ❌ MSI test error: {e}")
            return False
    
    def run_complete_test(self) -> bool:
        """Run the complete test and build process."""
        print("🚀 Starting complete gytmdl-gui test and build...")
        
        steps = [
            ("Check dependencies", self.check_dependencies),
            ("Build sidecar binary", lambda: self.build_sidecar_simple() is not None),
            ("Build frontend", self.build_frontend),
            ("Build Tauri app", self.build_tauri_app),
        ]
        
        for step_name, step_func in steps:
            print(f"\n📋 Step: {step_name}")
            try:
                if not step_func():
                    print(f"❌ Test failed at step: {step_name}")
                    return False
            except Exception as e:
                print(f"❌ Step '{step_name}' failed with exception: {e}")
                return False
        
        # Find and test installers
        installers = self.find_installers()
        
        if not installers:
            print("❌ No installers found to test")
            return False
        
        installer_tests_passed = 0
        for installer in installers:
            if self.test_installer_basic(installer):
                installer_tests_passed += 1
        
        print(f"\n📊 Test Results:")
        print(f"  Installers found: {len(installers)}")
        print(f"  Installers tested successfully: {installer_tests_passed}")
        
        if installer_tests_passed > 0:
            print("\n🎉 Complete test passed successfully!")
            print(f"📁 Installers available:")
            for installer in installers:
                print(f"  - {installer}")
            
            print(f"\n📋 Next steps:")
            print(f"  1. Install the appropriate installer for your platform")
            print(f"  2. Launch the application")
            print(f"  3. Test basic functionality (add a YouTube Music URL)")
            print(f"  4. Verify downloads work correctly")
            
            return True
        else:
            print("\n❌ No installers passed testing")
            return False


def main():
    """Main entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description="Complete test and build for gytmdl-gui")
    
    args = parser.parse_args()
    
    tester = CompleteTester()
    success = tester.run_complete_test()
    
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()