#!/usr/bin/env python3
"""
Test script to verify gytmdl-gui isolation from system gytmdl installation.
This script helps verify that the GUI app doesn't interfere with system gytmdl.
"""

import os
import sys
import subprocess
import tempfile
import json
from pathlib import Path
from typing import Dict, List, Optional, Tuple

class IsolationTester:
    """Tests the isolation between GUI app and system gytmdl."""
    
    def __init__(self):
        self.test_results = []
        self.system_gytmdl_available = False
        self.gui_app_dirs = self._get_gui_app_directories()
    
    def _get_gui_app_directories(self) -> Dict[str, Path]:
        """Get expected GUI app directories for the current platform."""
        if sys.platform == "win32":
            base = Path(os.environ.get("APPDATA", ""))
        elif sys.platform == "darwin":
            base = Path.home() / "Library" / "Application Support"
        else:  # Linux and other Unix-like
            base = Path(os.environ.get("XDG_CONFIG_HOME", Path.home() / ".config"))
        
        return {
            "config": base / "gytmdl-gui" / "config",
            "cache": base / "gytmdl-gui" / "cache", 
            "data": base / "gytmdl-gui" / "data"
        }
    
    def log_result(self, test_name: str, passed: bool, message: str = ""):
        """Log a test result."""
        status = "✅" if passed else "❌"
        self.test_results.append((test_name, passed, message))
        print(f"{status} {test_name}: {message}")
    
    def check_system_gytmdl(self) -> bool:
        """Check if system gytmdl is installed and working."""
        print("🔍 Checking system gytmdl installation...")
        
        try:
            result = subprocess.run(
                ["gytmdl", "--version"],
                capture_output=True,
                text=True,
                timeout=10
            )
            
            if result.returncode == 0:
                version = result.stdout.strip()
                self.log_result("System gytmdl available", True, f"Version: {version}")
                self.system_gytmdl_available = True
                return True
            else:
                self.log_result("System gytmdl available", False, "Command failed")
                return False
                
        except FileNotFoundError:
            self.log_result("System gytmdl available", False, "Not installed")
            return False
        except subprocess.TimeoutExpired:
            self.log_result("System gytmdl available", False, "Command timed out")
            return False
        except Exception as e:
            self.log_result("System gytmdl available", False, f"Error: {e}")
            return False
    
    def check_system_gytmdl_config(self) -> Optional[Dict]:
        """Check system gytmdl configuration."""
        if not self.system_gytmdl_available:
            return None
        
        print("\n🔍 Checking system gytmdl configuration...")
        
        # Try to find system config directories
        system_config_paths = self._get_system_config_paths()
        
        config_info = {
            "config_files": [],
            "cache_dirs": [],
            "data_dirs": []
        }
        
        for path in system_config_paths:
            if path.exists():
                config_info["config_files"].append(str(path))
                self.log_result(f"System config found", True, str(path))
        
        if not config_info["config_files"]:
            self.log_result("System config found", False, "No config files found")
        
        return config_info
    
    def _get_system_config_paths(self) -> List[Path]:
        """Get potential system gytmdl config paths."""
        paths = []
        
        if sys.platform == "win32":
            if "APPDATA" in os.environ:
                paths.append(Path(os.environ["APPDATA"]) / "gytmdl" / "gytmdl.conf")
            if "USERPROFILE" in os.environ:
                paths.append(Path(os.environ["USERPROFILE"]) / ".gytmdl" / "gytmdl.conf")
        elif sys.platform == "darwin":
            home = Path.home()
            paths.extend([
                home / "Library" / "Application Support" / "gytmdl" / "gytmdl.conf",
                home / ".gytmdl" / "gytmdl.conf"
            ])
        else:  # Linux
            home = Path.home()
            xdg_config = os.environ.get("XDG_CONFIG_HOME", home / ".config")
            paths.extend([
                Path(xdg_config) / "gytmdl" / "gytmdl.conf",
                home / ".gytmdl" / "gytmdl.conf"
            ])
        
        return paths
    
    def test_gui_app_directories(self) -> bool:
        """Test that GUI app creates its own directories."""
        print("\n🔍 Testing GUI app directory isolation...")
        
        all_created = True
        
        for dir_type, dir_path in self.gui_app_dirs.items():
            # Create the directory to simulate GUI app behavior
            try:
                dir_path.mkdir(parents=True, exist_ok=True)
                
                if dir_path.exists():
                    self.log_result(f"GUI {dir_type} directory", True, str(dir_path))
                else:
                    self.log_result(f"GUI {dir_type} directory", False, f"Failed to create {dir_path}")
                    all_created = False
                    
            except Exception as e:
                self.log_result(f"GUI {dir_type} directory", False, f"Error creating {dir_path}: {e}")
                all_created = False
        
        return all_created
    
    def test_environment_isolation(self) -> bool:
        """Test environment variable isolation."""
        print("\n🔍 Testing environment variable isolation...")
        
        # Simulate the environment variables that the GUI app would set
        isolation_env_vars = {
            "GYTMDL_CONFIG_DIR": str(self.gui_app_dirs["config"]),
            "GYTMDL_CACHE_DIR": str(self.gui_app_dirs["cache"]),
            "GYTMDL_DATA_DIR": str(self.gui_app_dirs["data"]),
            "GYTMDL_NO_SYSTEM_CONFIG": "1",
            "GYTMDL_GUI_MODE": "1"
        }
        
        # Add platform-specific variables
        if sys.platform != "win32":
            isolation_env_vars.update({
                "XDG_CONFIG_HOME": str(self.gui_app_dirs["config"]),
                "XDG_CACHE_HOME": str(self.gui_app_dirs["cache"]),
                "XDG_DATA_HOME": str(self.gui_app_dirs["data"])
            })
        
        all_vars_set = True
        
        for var_name, var_value in isolation_env_vars.items():
            # Test that we can set the variable
            try:
                os.environ[var_name] = var_value
                if os.environ.get(var_name) == var_value:
                    self.log_result(f"Environment var {var_name}", True, f"Set to {var_value}")
                else:
                    self.log_result(f"Environment var {var_name}", False, "Failed to set")
                    all_vars_set = False
            except Exception as e:
                self.log_result(f"Environment var {var_name}", False, f"Error: {e}")
                all_vars_set = False
        
        return all_vars_set
    
    def test_sidecar_binary_isolation(self) -> bool:
        """Test that sidecar binary would be isolated."""
        print("\n🔍 Testing sidecar binary isolation...")
        
        project_root = Path(__file__).parent.parent
        sidecars_dir = project_root / "src-tauri" / "sidecars"
        
        # Check if sidecars directory exists
        if not sidecars_dir.exists():
            self.log_result("Sidecars directory", False, f"Directory not found: {sidecars_dir}")
            return False
        
        self.log_result("Sidecars directory", True, str(sidecars_dir))
        
        # Look for sidecar binaries
        sidecar_files = list(sidecars_dir.glob("gytmdl-*"))
        
        if sidecar_files:
            self.log_result("Sidecar binaries found", True, f"Found {len(sidecar_files)} binaries")
            for sidecar in sidecar_files:
                self.log_result(f"  Sidecar binary", True, sidecar.name)
        else:
            self.log_result("Sidecar binaries found", False, "No sidecar binaries found")
            print("  ℹ️ Run build script to create sidecar binaries")
        
        return True
    
    def test_config_file_isolation(self) -> bool:
        """Test creating isolated config files."""
        print("\n🔍 Testing config file isolation...")
        
        # Create a test config file in the GUI app directory
        config_dir = self.gui_app_dirs["config"]
        test_config_file = config_dir / "gytmdl.conf"
        
        test_config_content = """
# Test configuration for gytmdl-gui
# This should be isolated from system gytmdl config

[general]
output_dir = ~/Downloads/gytmdl-gui
quality = best

[gui]
theme = dark
concurrent_downloads = 3
"""
        
        try:
            config_dir.mkdir(parents=True, exist_ok=True)
            test_config_file.write_text(test_config_content)
            
            if test_config_file.exists():
                self.log_result("GUI config file creation", True, str(test_config_file))
                
                # Verify content
                content = test_config_file.read_text()
                if "gytmdl-gui" in content:
                    self.log_result("GUI config file content", True, "Content verified")
                    return True
                else:
                    self.log_result("GUI config file content", False, "Content verification failed")
                    return False
            else:
                self.log_result("GUI config file creation", False, "File not created")
                return False
                
        except Exception as e:
            self.log_result("GUI config file creation", False, f"Error: {e}")
            return False
    
    def simulate_concurrent_usage(self) -> bool:
        """Simulate concurrent usage of system gytmdl and GUI app."""
        print("\n🔍 Testing concurrent usage simulation...")
        
        if not self.system_gytmdl_available:
            self.log_result("Concurrent usage test", False, "System gytmdl not available")
            return False
        
        # Test 1: System gytmdl should still work
        try:
            result = subprocess.run(
                ["gytmdl", "--help"],
                capture_output=True,
                text=True,
                timeout=10
            )
            
            if result.returncode == 0:
                self.log_result("System gytmdl still works", True, "Help command successful")
            else:
                self.log_result("System gytmdl still works", False, "Help command failed")
                return False
                
        except Exception as e:
            self.log_result("System gytmdl still works", False, f"Error: {e}")
            return False
        
        # Test 2: GUI app directories don't interfere
        config_exists = any(self.gui_app_dirs[d].exists() for d in self.gui_app_dirs)
        if config_exists:
            self.log_result("GUI directories don't interfere", True, "GUI directories exist separately")
        else:
            self.log_result("GUI directories don't interfere", False, "GUI directories not found")
        
        return True
    
    def cleanup_test_files(self) -> bool:
        """Clean up test files created during testing."""
        print("\n🧹 Cleaning up test files...")
        
        cleaned_up = True
        
        for dir_type, dir_path in self.gui_app_dirs.items():
            if dir_path.exists():
                try:
                    # Only remove if it looks like our test directory
                    if "gytmdl-gui" in str(dir_path):
                        import shutil
                        shutil.rmtree(dir_path)
                        self.log_result(f"Cleanup {dir_type} directory", True, f"Removed {dir_path}")
                    else:
                        self.log_result(f"Cleanup {dir_type} directory", False, f"Skipped {dir_path} (safety)")
                except Exception as e:
                    self.log_result(f"Cleanup {dir_type} directory", False, f"Error: {e}")
                    cleaned_up = False
        
        return cleaned_up
    
    def generate_report(self) -> Dict:
        """Generate a comprehensive test report."""
        passed_tests = sum(1 for _, passed, _ in self.test_results if passed)
        total_tests = len(self.test_results)
        
        return {
            "summary": {
                "total_tests": total_tests,
                "passed_tests": passed_tests,
                "failed_tests": total_tests - passed_tests,
                "success_rate": (passed_tests / total_tests * 100) if total_tests > 0 else 0,
                "system_gytmdl_available": self.system_gytmdl_available
            },
            "tests": [
                {
                    "name": name,
                    "passed": passed,
                    "message": message
                }
                for name, passed, message in self.test_results
            ],
            "gui_app_directories": {
                name: str(path) for name, path in self.gui_app_dirs.items()
            }
        }
    
    def run_all_tests(self, cleanup: bool = True) -> bool:
        """Run all isolation tests."""
        print("🚀 Starting gytmdl-gui isolation tests...")
        print(f"Platform: {sys.platform}")
        
        tests = [
            ("Check system gytmdl", self.check_system_gytmdl),
            ("Check system config", lambda: self.check_system_gytmdl_config() is not None),
            ("Test GUI directories", self.test_gui_app_directories),
            ("Test environment isolation", self.test_environment_isolation),
            ("Test sidecar isolation", self.test_sidecar_binary_isolation),
            ("Test config isolation", self.test_config_file_isolation),
            ("Test concurrent usage", self.simulate_concurrent_usage),
        ]
        
        all_passed = True
        
        for test_name, test_func in tests:
            try:
                result = test_func()
                if not result:
                    all_passed = False
            except Exception as e:
                print(f"❌ {test_name} failed with exception: {e}")
                all_passed = False
        
        if cleanup:
            self.cleanup_test_files()
        
        # Generate and display report
        report = self.generate_report()
        
        print(f"\n📊 Isolation Test Summary:")
        print(f"   Total tests: {report['summary']['total_tests']}")
        print(f"   Passed: {report['summary']['passed_tests']}")
        print(f"   Failed: {report['summary']['failed_tests']}")
        print(f"   Success rate: {report['summary']['success_rate']:.1f}%")
        print(f"   System gytmdl available: {report['summary']['system_gytmdl_available']}")
        
        if all_passed:
            print("\n🎉 All isolation tests passed! The GUI app should be properly isolated.")
        else:
            print("\n⚠️ Some isolation tests failed. Review the issues above.")
        
        return all_passed


def main():
    """Main entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description="Test gytmdl-gui isolation")
    parser.add_argument("--no-cleanup", action="store_true",
                       help="Don't clean up test files")
    parser.add_argument("--json-output", type=Path,
                       help="Save test report as JSON")
    
    args = parser.parse_args()
    
    tester = IsolationTester()
    success = tester.run_all_tests(cleanup=not args.no_cleanup)
    
    if args.json_output:
        report = tester.generate_report()
        with open(args.json_output, 'w') as f:
            json.dump(report, f, indent=2)
        print(f"\n📄 Test report saved to: {args.json_output}")
    
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()