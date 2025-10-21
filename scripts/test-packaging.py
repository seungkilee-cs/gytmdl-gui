#!/usr/bin/env python3
"""
Comprehensive test runner for gytmdl-gui packaging functionality.
This script runs various tests to verify that the packaging system works correctly.
"""

import os
import sys
import subprocess
import platform
import json
import tempfile
import shutil
from pathlib import Path
from typing import Dict, List, Optional, Tuple

class PackagingTestRunner:
    """Test runner for packaging functionality."""
    
    def __init__(self, project_root: Path):
        self.project_root = Path(project_root).resolve()
        self.test_results = []
        self.failed_tests = []
        
    def run_test(self, test_name: str, test_func) -> bool:
        """Run a single test and record the result."""
        print(f"🧪 Running test: {test_name}")
        
        try:
            result = test_func()
            if result:
                print(f"  ✅ {test_name} passed")
                self.test_results.append((test_name, True, None))
                return True
            else:
                print(f"  ❌ {test_name} failed")
                self.test_results.append((test_name, False, "Test returned False"))
                self.failed_tests.append(test_name)
                return False
        except Exception as e:
            print(f"  ❌ {test_name} failed with exception: {e}")
            self.test_results.append((test_name, False, str(e)))
            self.failed_tests.append(test_name)
            return False
    
    def test_project_structure(self) -> bool:
        """Test that the project has the correct structure for packaging."""
        required_files = [
            "package.json",
            "src-tauri/Cargo.toml",
            "src-tauri/tauri.conf.json",
            "build-config.json",
            "build-scripts/build-sidecars.py",
            "build-scripts/pyinstaller-config.spec",
            "scripts/build-and-package.py",
        ]
        
        for file_path in required_files:
            full_path = self.project_root / file_path
            if not full_path.exists():
                print(f"    Missing required file: {file_path}")
                return False
        
        required_dirs = [
            "src",
            "src-tauri/src",
            "build-scripts",
            "scripts",
        ]
        
        for dir_path in required_dirs:
            full_path = self.project_root / dir_path
            if not full_path.exists() or not full_path.is_dir():
                print(f"    Missing required directory: {dir_path}")
                return False
        
        return True
    
    def test_tauri_config_validity(self) -> bool:
        """Test that the Tauri configuration is valid."""
        config_path = self.project_root / "src-tauri" / "tauri.conf.json"
        
        try:
            with open(config_path) as f:
                config = json.load(f)
            
            # Check essential fields
            required_fields = ["productName", "version", "identifier", "bundle"]
            for field in required_fields:
                if field not in config:
                    print(f"    Missing required field: {field}")
                    return False
            
            # Check bundle configuration
            bundle = config["bundle"]
            bundle_fields = ["active", "targets", "icon", "resources", "externalBin"]
            for field in bundle_fields:
                if field not in bundle:
                    print(f"    Missing bundle field: {field}")
                    return False
            
            # Check sidecar binaries are configured
            external_bin = bundle["externalBin"]
            if not isinstance(external_bin, list) or len(external_bin) == 0:
                print("    externalBin should be a non-empty list")
                return False
            
            # Check for expected platforms
            external_bin_str = " ".join(external_bin)
            expected_platforms = ["windows-msvc", "apple-darwin", "linux-gnu"]
            for platform_name in expected_platforms:
                if platform_name not in external_bin_str:
                    print(f"    Missing platform in externalBin: {platform_name}")
                    return False
            
            return True
            
        except json.JSONDecodeError as e:
            print(f"    Invalid JSON: {e}")
            return False
        except Exception as e:
            print(f"    Error reading config: {e}")
            return False
    
    def test_build_config_validity(self) -> bool:
        """Test that the build configuration is valid."""
        config_path = self.project_root / "build-config.json"
        
        try:
            with open(config_path) as f:
                config = json.load(f)
            
            # Check essential sections
            required_sections = ["release", "code_signing", "bundle"]
            for section in required_sections:
                if section not in config:
                    print(f"    Missing required section: {section}")
                    return False
            
            # Check platform-specific bundle configs
            bundle = config["bundle"]
            platform_configs = ["windows", "macos", "linux"]
            for platform_name in platform_configs:
                if platform_name not in bundle:
                    print(f"    Missing platform bundle config: {platform_name}")
                    return False
            
            return True
            
        except json.JSONDecodeError as e:
            print(f"    Invalid JSON: {e}")
            return False
        except Exception as e:
            print(f"    Error reading config: {e}")
            return False
    
    def test_package_json_validity(self) -> bool:
        """Test that package.json is properly configured."""
        package_path = self.project_root / "package.json"
        
        try:
            with open(package_path) as f:
                package = json.load(f)
            
            # Check essential fields
            required_fields = ["name", "version", "scripts", "devDependencies"]
            for field in required_fields:
                if field not in package:
                    print(f"    Missing required field: {field}")
                    return False
            
            # Check for Tauri-related scripts
            scripts = package["scripts"]
            required_scripts = ["tauri", "build", "dev"]
            for script in required_scripts:
                if script not in scripts:
                    print(f"    Missing required script: {script}")
                    return False
            
            # Check for Tauri CLI in devDependencies
            dev_deps = package["devDependencies"]
            if "@tauri-apps/cli" not in dev_deps:
                print("    Missing @tauri-apps/cli in devDependencies")
                return False
            
            return True
            
        except json.JSONDecodeError as e:
            print(f"    Invalid JSON: {e}")
            return False
        except Exception as e:
            print(f"    Error reading package.json: {e}")
            return False
    
    def test_cargo_toml_validity(self) -> bool:
        """Test that Cargo.toml is properly configured."""
        cargo_path = self.project_root / "src-tauri" / "Cargo.toml"
        
        try:
            with open(cargo_path) as f:
                cargo_content = f.read()
            
            # Check for essential sections and dependencies
            required_content = [
                "[package]",
                "[dependencies]",
                "tauri",
                "serde",
                "tokio",
                "which",  # For binary detection
                "regex",
                "chrono",
                "uuid",
            ]
            
            for content in required_content:
                if content not in cargo_content:
                    print(f"    Missing required content: {content}")
                    return False
            
            return True
            
        except Exception as e:
            print(f"    Error reading Cargo.toml: {e}")
            return False
    
    def test_build_scripts_exist(self) -> bool:
        """Test that all build scripts exist and are properly configured."""
        scripts_to_check = [
            ("build-scripts/build-sidecars.py", True),
            ("build-scripts/build-all-platforms.sh", True),
            ("build-scripts/build-all-platforms.bat", False),  # Not executable on Unix
            ("scripts/build-and-package.py", True),
            ("scripts/dev-build.sh", True),
            ("scripts/test-packaging.py", True),  # This script itself
        ]
        
        for script_path, should_be_executable in scripts_to_check:
            full_path = self.project_root / script_path
            
            if not full_path.exists():
                print(f"    Missing build script: {script_path}")
                return False
            
            # Check if shell scripts are executable (Unix only)
            if should_be_executable and os.name == 'posix':
                if script_path.endswith('.sh') or script_path.endswith('.py'):
                    stat_info = full_path.stat()
                    if not (stat_info.st_mode & 0o111):
                        print(f"    Script not executable: {script_path}")
                        return False
        
        return True
    
    def test_installer_configs_exist(self) -> bool:
        """Test that installer configuration files exist."""
        configs_to_check = [
            "src-tauri/entitlements.plist",
            "src-tauri/wix-template.wxs",
            "src-tauri/scripts/preinst.sh",
            "src-tauri/scripts/postinst.sh",
            "src-tauri/scripts/prerm.sh",
            "src-tauri/scripts/postrm.sh",
        ]
        
        for config_path in configs_to_check:
            full_path = self.project_root / config_path
            if not full_path.exists():
                print(f"    Missing installer config: {config_path}")
                return False
        
        return True
    
    def test_github_actions_workflow(self) -> bool:
        """Test that GitHub Actions workflow is properly configured."""
        workflow_path = self.project_root / ".github" / "workflows" / "build-and-release.yml"
        
        if not workflow_path.exists():
            print("    Missing GitHub Actions workflow")
            return False
        
        try:
            with open(workflow_path) as f:
                workflow_content = f.read()
            
            # Check for essential workflow components
            required_content = [
                "name: Build and Release",
                "build-sidecars",
                "build-tauri",
                "create-release",
                "test-installers",
                "macos-latest",
                "ubuntu-20.04",
                "windows-latest",
                "x86_64-apple-darwin",
                "aarch64-apple-darwin",
                "x86_64-unknown-linux-gnu",
                "x86_64-pc-windows-msvc",
            ]
            
            for content in required_content:
                if content not in workflow_content:
                    print(f"    Missing workflow content: {content}")
                    return False
            
            return True
            
        except Exception as e:
            print(f"    Error reading workflow: {e}")
            return False
    
    def test_rust_compilation(self) -> bool:
        """Test that the Rust code compiles successfully."""
        try:
            # Run cargo check to verify compilation
            result = subprocess.run(
                ["cargo", "check"],
                cwd=self.project_root / "src-tauri",
                capture_output=True,
                text=True,
                timeout=120  # 2 minute timeout
            )
            
            if result.returncode != 0:
                print(f"    Cargo check failed:")
                print(f"    STDOUT: {result.stdout}")
                print(f"    STDERR: {result.stderr}")
                return False
            
            return True
            
        except subprocess.TimeoutExpired:
            print("    Cargo check timed out")
            return False
        except FileNotFoundError:
            print("    Cargo not found (skipping Rust compilation test)")
            return True  # Don't fail if cargo is not available
        except Exception as e:
            print(f"    Error running cargo check: {e}")
            return False
    
    def test_frontend_build(self) -> bool:
        """Test that the frontend builds successfully."""
        try:
            # Check if node_modules exists, if not try to install
            node_modules = self.project_root / "node_modules"
            if not node_modules.exists():
                print("    Installing npm dependencies...")
                result = subprocess.run(
                    ["npm", "install"],
                    cwd=self.project_root,
                    capture_output=True,
                    text=True,
                    timeout=300  # 5 minute timeout
                )
                
                if result.returncode != 0:
                    print(f"    npm install failed:")
                    print(f"    STDERR: {result.stderr}")
                    return False
            
            # Run build
            result = subprocess.run(
                ["npm", "run", "build"],
                cwd=self.project_root,
                capture_output=True,
                text=True,
                timeout=120  # 2 minute timeout
            )
            
            if result.returncode != 0:
                print(f"    Frontend build failed:")
                print(f"    STDERR: {result.stderr}")
                return False
            
            # Check if dist directory was created
            dist_dir = self.project_root / "dist"
            if not dist_dir.exists():
                print("    Frontend build did not create dist directory")
                return False
            
            return True
            
        except subprocess.TimeoutExpired:
            print("    Frontend build timed out")
            return False
        except FileNotFoundError:
            print("    npm not found (skipping frontend build test)")
            return True  # Don't fail if npm is not available
        except Exception as e:
            print(f"    Error running frontend build: {e}")
            return False
    
    def test_sidecar_build_script(self) -> bool:
        """Test that the sidecar build script can be executed."""
        build_script = self.project_root / "build-scripts" / "build-sidecars.py"
        
        try:
            # Run the script with --help to verify it works
            result = subprocess.run(
                [sys.executable, str(build_script), "--help"],
                capture_output=True,
                text=True,
                timeout=30
            )
            
            if result.returncode != 0:
                print(f"    Sidecar build script failed:")
                print(f"    STDERR: {result.stderr}")
                return False
            
            # Check that help output contains expected content
            if "Build gytmdl sidecar binaries" not in result.stdout:
                print("    Sidecar build script help output unexpected")
                return False
            
            return True
            
        except subprocess.TimeoutExpired:
            print("    Sidecar build script timed out")
            return False
        except Exception as e:
            print(f"    Error running sidecar build script: {e}")
            return False
    
    def test_packaging_script(self) -> bool:
        """Test that the packaging script can be executed."""
        packaging_script = self.project_root / "scripts" / "build-and-package.py"
        
        try:
            # Run the script with --help to verify it works
            result = subprocess.run(
                [sys.executable, str(packaging_script), "--help"],
                capture_output=True,
                text=True,
                timeout=30
            )
            
            if result.returncode != 0:
                print(f"    Packaging script failed:")
                print(f"    STDERR: {result.stderr}")
                return False
            
            # Check that help output contains expected content
            if "Build and package gytmdl-gui" not in result.stdout:
                print("    Packaging script help output unexpected")
                return False
            
            return True
            
        except subprocess.TimeoutExpired:
            print("    Packaging script timed out")
            return False
        except Exception as e:
            print(f"    Error running packaging script: {e}")
            return False
    
    def run_all_tests(self) -> bool:
        """Run all packaging tests."""
        print("🚀 Running gytmdl-gui packaging tests...")
        print(f"Platform: {platform.system()} {platform.machine()}")
        print(f"Project root: {self.project_root}")
        
        tests = [
            ("Project Structure", self.test_project_structure),
            ("Tauri Config Validity", self.test_tauri_config_validity),
            ("Build Config Validity", self.test_build_config_validity),
            ("Package.json Validity", self.test_package_json_validity),
            ("Cargo.toml Validity", self.test_cargo_toml_validity),
            ("Build Scripts Exist", self.test_build_scripts_exist),
            ("Installer Configs Exist", self.test_installer_configs_exist),
            ("GitHub Actions Workflow", self.test_github_actions_workflow),
            ("Rust Compilation", self.test_rust_compilation),
            ("Frontend Build", self.test_frontend_build),
            ("Sidecar Build Script", self.test_sidecar_build_script),
            ("Packaging Script", self.test_packaging_script),
        ]
        
        passed = 0
        total = len(tests)
        
        for test_name, test_func in tests:
            if self.run_test(test_name, test_func):
                passed += 1
        
        print(f"\n📊 Test Results: {passed}/{total} tests passed")
        
        if self.failed_tests:
            print(f"❌ Failed tests: {', '.join(self.failed_tests)}")
            return False
        else:
            print("✅ All tests passed!")
            return True


def main():
    """Main entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description="Test gytmdl-gui packaging functionality")
    parser.add_argument("--project-root", type=Path,
                       default=Path(__file__).parent.parent,
                       help="Project root directory")
    
    args = parser.parse_args()
    
    # Change to project root
    os.chdir(args.project_root)
    
    # Create and run test runner
    runner = PackagingTestRunner(args.project_root)
    success = runner.run_all_tests()
    
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()