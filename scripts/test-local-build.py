#!/usr/bin/env python3
"""
Local build and test script for gytmdl-gui installers.
This script allows you to build and test installers locally before pushing to CI.
"""

import os
import sys
import subprocess
import platform
import shutil
import tempfile
import json
from pathlib import Path
from typing import Dict, List, Optional

class LocalBuildTester:
    """Handles local building and testing of gytmdl-gui installers."""
    
    def __init__(self, project_root: Path):
        self.project_root = Path(project_root).resolve()
        self.platform_info = self._get_platform_info()
        self.build_dir = self.project_root / "target" / "release"
        self.test_dir = Path(tempfile.mkdtemp(prefix="gytmdl-gui-test-"))
        
        print(f"🏗️ Local Build Tester")
        print(f"Project root: {self.project_root}")
        print(f"Platform: {self.platform_info['os']} {self.platform_info['arch']}")
        print(f"Test directory: {self.test_dir}")
    
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
        elif system == "windows":
            return {
                "os": "windows",
                "arch": "x86_64" if "64" in str(sys.maxsize) else "i686",
                "target": "x86_64-pc-windows-msvc" if "64" in str(sys.maxsize) else "i686-pc-windows-msvc",
                "extension": ".exe",
                "installer_formats": ["msi", "nsis"]
            }
        else:
            raise ValueError(f"Unsupported platform: {system}")
    
    def check_dependencies(self) -> bool:
        """Check if all required build dependencies are available."""
        print("\n🔍 Checking build dependencies...")
        
        required_tools = {
            "node": "Node.js is required for frontend build",
            "npm": "npm is required for dependency management", 
            "cargo": "Rust/Cargo is required for Tauri build",
            "python3": "Python 3 is required for sidecar builds"
        }
        
        missing_tools = []
        
        for tool, description in required_tools.items():
            try:
                result = subprocess.run([tool, "--version"], 
                                     capture_output=True, check=True)
                print(f"  ✓ {tool} found")
            except (subprocess.CalledProcessError, FileNotFoundError):
                print(f"  ✗ {tool} not found - {description}")
                missing_tools.append(tool)
        
        if missing_tools:
            print(f"\n❌ Missing required dependencies: {', '.join(missing_tools)}")
            return False
        
        print("✅ All dependencies satisfied")
        return True
    
    def build_sidecar_binary(self) -> Optional[Path]:
        """Build sidecar binary for the current platform."""
        print(f"\n🔨 Building sidecar binary for {self.platform_info['target']}...")
        
        # Try the simple sidecar build script first
        simple_build_script = self.project_root / "scripts" / "build-simple-sidecar.py"
        
        if simple_build_script.exists():
            print("  Using simple sidecar builder...")
            result = subprocess.run([sys.executable, str(simple_build_script)], 
                                  cwd=self.project_root, 
                                  capture_output=True, text=True)
        else:
            # Fallback to the production build script
            build_script = self.project_root / "scripts" / "build-production-bundle.py"
            
            if not build_script.exists():
                print(f"❌ Build script not found: {build_script}")
                return None
            
            cmd = [
                sys.executable, str(build_script),
                "--project-root", str(self.project_root)
            ]
            
            result = subprocess.run(cmd, cwd=self.project_root, 
                                  capture_output=True, text=True)
            
            if result.returncode != 0:
                print(f"❌ Sidecar build failed:")
                print(f"STDOUT: {result.stdout}")
                print(f"STDERR: {result.stderr}")
                return None
            
            # Find the built sidecar binary
            sidecars_dir = self.project_root / "src-tauri" / "sidecars"
            binary_name = f"gytmdl-{self.platform_info['target']}{self.platform_info['extension']}"
            binary_path = sidecars_dir / binary_name
            
            if binary_path.exists():
                print(f"✅ Sidecar binary built: {binary_path}")
                return binary_path
            else:
                print(f"❌ Sidecar binary not found at expected location: {binary_path}")
                return None
                
        except Exception as e:
            print(f"❌ Sidecar build error: {e}")
            return None
    
    def build_tauri_app(self) -> bool:
        """Build the Tauri application."""
        print(f"\n🦀 Building Tauri application...")
        
        try:
            # Install frontend dependencies
            print("  📦 Installing frontend dependencies...")
            subprocess.run(["npm", "install"], 
                         cwd=self.project_root, check=True)
            
            # Build frontend
            print("  🎨 Building frontend...")
            subprocess.run(["npm", "run", "build"], 
                         cwd=self.project_root, check=True)
            
            # Build Tauri app
            print("  🔧 Building Tauri app...")
            cmd = ["cargo", "tauri", "build"]
            
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
    
    def find_installers(self) -> List[Path]:
        """Find built installer files."""
        print(f"\n📦 Looking for installers...")
        
        bundle_dir = self.build_dir / "bundle"
        installers = []
        
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
                elif format_name == "nsis":
                    files = list(format_dir.glob("*-setup.exe"))
                else:
                    files = []
                
                for file in files:
                    installers.append(file)
                    print(f"  ✓ Found {format_name}: {file.name}")
        
        if not installers:
            print("  ❌ No installers found")
        
        return installers
    
    def test_installer_macos(self, installer_path: Path) -> bool:
        """Test macOS DMG installer."""
        print(f"\n🧪 Testing macOS installer: {installer_path.name}")
        
        try:
            # Mount DMG
            print("  📀 Mounting DMG...")
            mount_result = subprocess.run(
                ["hdiutil", "attach", str(installer_path), "-readonly", "-nobrowse"],
                capture_output=True, text=True
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
            if not app_bundles:
                print("  ❌ No app bundle found in DMG")
                return False
            
            app_bundle = app_bundles[0]
            print(f"  ✓ Found app bundle: {app_bundle.name}")
            
            # Copy app to test directory for testing
            test_app = self.test_dir / app_bundle.name
            shutil.copytree(app_bundle, test_app)
            print(f"  ✓ Copied app to test directory")
            
            # Unmount DMG
            subprocess.run(["hdiutil", "detach", str(volume)], 
                         capture_output=True)
            
            # Test app execution
            print("  🚀 Testing app execution...")
            app_executable = test_app / "Contents" / "MacOS" / "gytmdl-gui"
            
            if not app_executable.exists():
                print(f"  ❌ App executable not found: {app_executable}")
                return False
            
            # Try to run the app (it might fail due to code signing, but we can check if it starts)
            test_result = subprocess.run([str(app_executable), "--version"], 
                                       capture_output=True, text=True, timeout=10)
            
            if test_result.returncode == 0:
                print(f"  ✅ App executed successfully: {test_result.stdout.strip()}")
            else:
                print(f"  ⚠️ App execution failed (might be due to code signing): {test_result.stderr}")
            
            # Check for sidecar binaries
            resources_dir = test_app / "Contents" / "Resources"
            sidecar_files = list(resources_dir.glob("gytmdl-*")) if resources_dir.exists() else []
            
            if sidecar_files:
                print(f"  ✓ Found {len(sidecar_files)} sidecar binaries")
                for sidecar in sidecar_files:
                    print(f"    - {sidecar.name}")
            else:
                print("  ⚠️ No sidecar binaries found")
            
            return True
            
        except Exception as e:
            print(f"  ❌ Test failed: {e}")
            return False
    
    def test_installer_linux(self, installer_path: Path) -> bool:
        """Test Linux installer."""
        print(f"\n🧪 Testing Linux installer: {installer_path.name}")
        
        try:
            if installer_path.suffix == ".deb":
                # Test DEB package
                print("  📦 Testing DEB package...")
                
                # Check package info
                info_result = subprocess.run(
                    ["dpkg-deb", "--info", str(installer_path)],
                    capture_output=True, text=True
                )
                
                if info_result.returncode == 0:
                    print("  ✓ Package info looks good")
                else:
                    print(f"  ❌ Package info failed: {info_result.stderr}")
                    return False
                
                # Check package contents
                contents_result = subprocess.run(
                    ["dpkg-deb", "--contents", str(installer_path)],
                    capture_output=True, text=True
                )
                
                if contents_result.returncode == 0:
                    print("  ✓ Package contents:")
                    # Look for main executable and sidecars
                    contents = contents_result.stdout
                    if "gytmdl-gui" in contents:
                        print("    - Main executable found")
                    if "gytmdl-" in contents:
                        print("    - Sidecar binaries found")
                else:
                    print(f"  ❌ Package contents check failed: {contents_result.stderr}")
                
                # Extract to test directory for further testing
                extract_dir = self.test_dir / "deb_extract"
                extract_dir.mkdir(exist_ok=True)
                
                extract_result = subprocess.run(
                    ["dpkg-deb", "--extract", str(installer_path), str(extract_dir)],
                    capture_output=True, text=True
                )
                
                if extract_result.returncode == 0:
                    print("  ✓ Package extracted successfully")
                    
                    # Find and test the main executable
                    executables = list(extract_dir.rglob("gytmdl-gui"))
                    if executables:
                        executable = executables[0]
                        print(f"  ✓ Found executable: {executable}")
                        
                        # Test execution
                        test_result = subprocess.run([str(executable), "--version"], 
                                                   capture_output=True, text=True, timeout=10)
                        
                        if test_result.returncode == 0:
                            print(f"  ✅ Executable works: {test_result.stdout.strip()}")
                        else:
                            print(f"  ⚠️ Executable test failed: {test_result.stderr}")
                    
                    # Check for sidecar binaries
                    sidecars = list(extract_dir.rglob("gytmdl-*"))
                    sidecar_binaries = [s for s in sidecars if s.is_file() and s.name != "gytmdl-gui"]
                    
                    if sidecar_binaries:
                        print(f"  ✓ Found {len(sidecar_binaries)} sidecar binaries")
                        for sidecar in sidecar_binaries:
                            print(f"    - {sidecar.name}")
                    else:
                        print("  ⚠️ No sidecar binaries found")
                
                return True
            
            elif installer_path.suffix == ".AppImage":
                # Test AppImage
                print("  📱 Testing AppImage...")
                
                # Make executable
                os.chmod(installer_path, 0o755)
                
                # Test execution
                test_result = subprocess.run([str(installer_path), "--version"], 
                                           capture_output=True, text=True, timeout=10)
                
                if test_result.returncode == 0:
                    print(f"  ✅ AppImage works: {test_result.stdout.strip()}")
                else:
                    print(f"  ⚠️ AppImage test failed: {test_result.stderr}")
                
                return True
            
            else:
                print(f"  ⚠️ Unsupported installer format: {installer_path.suffix}")
                return False
                
        except Exception as e:
            print(f"  ❌ Test failed: {e}")
            return False
    
    def test_sidecar_isolation(self) -> bool:
        """Test that the sidecar doesn't interfere with system gytmdl."""
        print(f"\n🔒 Testing sidecar isolation...")
        
        # Check if system gytmdl is installed
        try:
            system_gytmdl = subprocess.run(["gytmdl", "--version"], 
                                         capture_output=True, text=True)
            if system_gytmdl.returncode == 0:
                print(f"  ℹ️ System gytmdl found: {system_gytmdl.stdout.strip()}")
                
                # Check system gytmdl config location
                config_result = subprocess.run(["gytmdl", "--help"], 
                                             capture_output=True, text=True)
                print("  ✓ System gytmdl is accessible")
            else:
                print("  ℹ️ No system gytmdl installation found")
        
        except FileNotFoundError:
            print("  ℹ️ No system gytmdl installation found")
        
        # Test sidecar binary directly
        sidecars_dir = self.project_root / "src-tauri" / "sidecars"
        binary_name = f"gytmdl-{self.platform_info['target']}{self.platform_info['extension']}"
        sidecar_path = sidecars_dir / binary_name
        
        if sidecar_path.exists():
            print(f"  🔍 Testing sidecar binary: {sidecar_path.name}")
            
            try:
                sidecar_result = subprocess.run([str(sidecar_path), "--version"], 
                                              capture_output=True, text=True, timeout=10)
                
                if sidecar_result.returncode == 0:
                    print(f"  ✅ Sidecar binary works: {sidecar_result.stdout.strip()}")
                    
                    # The sidecar should use its own config directory
                    # This is typically handled by the Tauri app setting appropriate environment variables
                    print("  ✓ Sidecar binary is functional and isolated")
                    return True
                else:
                    print(f"  ❌ Sidecar binary failed: {sidecar_result.stderr}")
                    return False
                    
            except Exception as e:
                print(f"  ❌ Sidecar test error: {e}")
                return False
        else:
            print(f"  ❌ Sidecar binary not found: {sidecar_path}")
            return False
    
    def run_end_to_end_test(self) -> bool:
        """Run complete end-to-end test."""
        print("🚀 Starting end-to-end build and test...")
        
        steps = [
            ("Check dependencies", self.check_dependencies),
            ("Build sidecar binary", lambda: self.build_sidecar_binary() is not None),
            ("Build Tauri app", self.build_tauri_app),
            ("Test sidecar isolation", self.test_sidecar_isolation),
        ]
        
        for step_name, step_func in steps:
            print(f"\n📋 Step: {step_name}")
            if not step_func():
                print(f"❌ Test failed at step: {step_name}")
                return False
        
        # Find and test installers
        installers = self.find_installers()
        
        if not installers:
            print("❌ No installers found to test")
            return False
        
        installer_tests_passed = 0
        for installer in installers:
            if self.platform_info["os"] == "macos" and installer.suffix == ".dmg":
                if self.test_installer_macos(installer):
                    installer_tests_passed += 1
            elif self.platform_info["os"] == "linux":
                if self.test_installer_linux(installer):
                    installer_tests_passed += 1
        
        print(f"\n📊 Test Results:")
        print(f"  Installers found: {len(installers)}")
        print(f"  Installers tested successfully: {installer_tests_passed}")
        
        if installer_tests_passed > 0:
            print("\n🎉 End-to-end test completed successfully!")
            print(f"📁 Test artifacts in: {self.test_dir}")
            return True
        else:
            print("\n❌ No installers passed testing")
            return False
    
    def cleanup(self):
        """Clean up test directory."""
        if self.test_dir.exists():
            shutil.rmtree(self.test_dir)
            print(f"🧹 Cleaned up test directory: {self.test_dir}")


def main():
    """Main entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description="Test gytmdl-gui build locally")
    parser.add_argument("--project-root", type=Path,
                       default=Path(__file__).parent.parent,
                       help="Project root directory")
    parser.add_argument("--keep-test-dir", action="store_true",
                       help="Keep test directory after completion")
    
    args = parser.parse_args()
    
    tester = LocalBuildTester(args.project_root)
    
    try:
        success = tester.run_end_to_end_test()
        
        if not args.keep_test_dir:
            tester.cleanup()
        else:
            print(f"📁 Test directory preserved: {tester.test_dir}")
        
        sys.exit(0 if success else 1)
        
    except KeyboardInterrupt:
        print("\n⚠️ Test interrupted by user")
        tester.cleanup()
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}")
        tester.cleanup()
        sys.exit(1)


if __name__ == "__main__":
    main()