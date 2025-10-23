#!/usr/bin/env python3
"""
Validation script for the automated release pipeline setup.
This script checks that all required components are properly configured.
"""

import os
import sys
import json
import subprocess
from pathlib import Path
from typing import Dict, List, Optional, Tuple

class ReleasePipelineValidator:
    """Validates the automated release pipeline configuration."""
    
    def __init__(self, project_root: Path):
        self.project_root = Path(project_root).resolve()
        self.workflows_dir = self.project_root / ".github" / "workflows"
        self.config_file = self.project_root / ".github" / "release-config.yml"
        
        self.required_workflows = [
            "build-and-release.yml",
            "dependency-update.yml", 
            "security-scan.yml",
            "release-management.yml"
        ]
        
        self.validation_results = []
    
    def log_result(self, check: str, passed: bool, message: str = ""):
        """Log a validation result."""
        status = "✅" if passed else "❌"
        self.validation_results.append((check, passed, message))
        print(f"{status} {check}: {message}")
    
    def check_workflow_files(self) -> bool:
        """Check that all required workflow files exist."""
        print("🔍 Checking workflow files...")
        
        if not self.workflows_dir.exists():
            self.log_result("Workflows directory", False, "Directory does not exist")
            return False
        
        all_exist = True
        for workflow in self.required_workflows:
            workflow_path = self.workflows_dir / workflow
            exists = workflow_path.exists()
            self.log_result(f"Workflow {workflow}", exists, 
                          f"Found at {workflow_path}" if exists else "File missing")
            if not exists:
                all_exist = False
        
        return all_exist
    
    def check_workflow_syntax(self) -> bool:
        """Basic syntax check for workflow files."""
        print("\n🔍 Checking workflow syntax...")
        
        all_valid = True
        for workflow in self.required_workflows:
            workflow_path = self.workflows_dir / workflow
            if not workflow_path.exists():
                continue
            
            try:
                with open(workflow_path, 'r') as f:
                    content = f.read()
                
                # Basic YAML structure checks
                has_name = "name:" in content
                has_on = "on:" in content
                has_jobs = "jobs:" in content
                
                valid = has_name and has_on and has_jobs
                self.log_result(f"Syntax {workflow}", valid,
                              "Basic YAML structure looks good" if valid else "Missing required sections")
                
                if not valid:
                    all_valid = False
                    
            except Exception as e:
                self.log_result(f"Syntax {workflow}", False, f"Error reading file: {e}")
                all_valid = False
        
        return all_valid
    
    def check_project_structure(self) -> bool:
        """Check that the project has the required structure."""
        print("\n🔍 Checking project structure...")
        
        required_files = [
            "package.json",
            "src-tauri/Cargo.toml",
            "src-tauri/tauri.conf.json",
            "src-tauri/src/main.rs",
            "build-scripts/build-sidecars.py"
        ]
        
        all_exist = True
        for file_path in required_files:
            full_path = self.project_root / file_path
            exists = full_path.exists()
            self.log_result(f"Project file {file_path}", exists,
                          "Found" if exists else "Missing")
            if not exists:
                all_exist = False
        
        return all_exist
    
    def check_version_consistency(self) -> bool:
        """Check that version numbers are consistent across files."""
        print("\n🔍 Checking version consistency...")
        
        versions = {}
        
        # Check package.json
        package_json = self.project_root / "package.json"
        if package_json.exists():
            try:
                with open(package_json) as f:
                    data = json.load(f)
                    versions["package.json"] = data.get("version", "unknown")
            except Exception as e:
                self.log_result("Version package.json", False, f"Error reading: {e}")
                return False
        
        # Check Cargo.toml
        cargo_toml = self.project_root / "src-tauri" / "Cargo.toml"
        if cargo_toml.exists():
            try:
                with open(cargo_toml) as f:
                    content = f.read()
                    for line in content.split('\n'):
                        if line.strip().startswith('version = '):
                            version = line.split('=')[1].strip().strip('"')
                            versions["Cargo.toml"] = version
                            break
            except Exception as e:
                self.log_result("Version Cargo.toml", False, f"Error reading: {e}")
                return False
        
        # Check tauri.conf.json
        tauri_conf = self.project_root / "src-tauri" / "tauri.conf.json"
        if tauri_conf.exists():
            try:
                with open(tauri_conf) as f:
                    data = json.load(f)
                    versions["tauri.conf.json"] = data.get("version", "unknown")
            except Exception as e:
                self.log_result("Version tauri.conf.json", False, f"Error reading: {e}")
                return False
        
        # Check consistency
        unique_versions = set(versions.values())
        consistent = len(unique_versions) == 1
        
        if consistent:
            version = list(unique_versions)[0]
            self.log_result("Version consistency", True, f"All files use version {version}")
        else:
            self.log_result("Version consistency", False, f"Inconsistent versions: {versions}")
        
        return consistent
    
    def check_build_dependencies(self) -> bool:
        """Check that build dependencies are available."""
        print("\n🔍 Checking build dependencies...")
        
        dependencies = {
            "node": ["node", "--version"],
            "npm": ["npm", "--version"],
            "cargo": ["cargo", "--version"],
            "python3": ["python3", "--version"]
        }
        
        all_available = True
        for dep_name, cmd in dependencies.items():
            try:
                result = subprocess.run(cmd, capture_output=True, text=True, timeout=10)
                available = result.returncode == 0
                version = result.stdout.strip().split('\n')[0] if available else "Not found"
                self.log_result(f"Dependency {dep_name}", available, version)
                if not available:
                    all_available = False
            except Exception as e:
                self.log_result(f"Dependency {dep_name}", False, f"Error checking: {e}")
                all_available = False
        
        return all_available
    
    def check_sidecar_build_setup(self) -> bool:
        """Check that sidecar build setup is correct."""
        print("\n🔍 Checking sidecar build setup...")
        
        build_script = self.project_root / "build-scripts" / "build-sidecars.py"
        if not build_script.exists():
            self.log_result("Sidecar build script", False, "build-sidecars.py not found")
            return False
        
        sidecars_dir = self.project_root / "src-tauri" / "sidecars"
        if not sidecars_dir.exists():
            self.log_result("Sidecars directory", False, "Directory does not exist")
            return False
        
        # Check for existing sidecar binaries
        sidecar_files = list(sidecars_dir.glob("gytmdl-*"))
        has_sidecars = len(sidecar_files) > 0
        
        self.log_result("Sidecar build script", True, "Found")
        self.log_result("Sidecars directory", True, "Found")
        self.log_result("Existing sidecars", has_sidecars, 
                       f"Found {len(sidecar_files)} sidecar files" if has_sidecars else "No sidecars found")
        
        return True
    
    def check_tauri_configuration(self) -> bool:
        """Check Tauri configuration for release pipeline."""
        print("\n🔍 Checking Tauri configuration...")
        
        tauri_conf = self.project_root / "src-tauri" / "tauri.conf.json"
        if not tauri_conf.exists():
            self.log_result("Tauri config", False, "tauri.conf.json not found")
            return False
        
        try:
            with open(tauri_conf) as f:
                config = json.load(f)
            
            # Check bundle configuration
            bundle = config.get("bundle", {})
            has_bundle = bundle.get("active", False)
            has_external_bin = "externalBin" in bundle
            has_targets = bundle.get("targets") is not None
            
            self.log_result("Bundle active", has_bundle, "Bundle is enabled")
            self.log_result("External binaries", has_external_bin, "Sidecar binaries configured")
            self.log_result("Bundle targets", has_targets, "Build targets configured")
            
            # Check updater configuration
            plugins = config.get("plugins", {})
            updater = plugins.get("updater", {})
            has_updater = updater.get("active", False)
            
            self.log_result("Updater plugin", has_updater, 
                          "Updater is enabled" if has_updater else "Updater is disabled")
            
            return has_bundle and has_external_bin
            
        except Exception as e:
            self.log_result("Tauri config", False, f"Error reading config: {e}")
            return False
    
    def generate_report(self) -> Dict:
        """Generate a comprehensive validation report."""
        passed_checks = sum(1 for _, passed, _ in self.validation_results if passed)
        total_checks = len(self.validation_results)
        
        report = {
            "summary": {
                "total_checks": total_checks,
                "passed_checks": passed_checks,
                "failed_checks": total_checks - passed_checks,
                "success_rate": (passed_checks / total_checks * 100) if total_checks > 0 else 0
            },
            "checks": [
                {
                    "name": check,
                    "passed": passed,
                    "message": message
                }
                for check, passed, message in self.validation_results
            ]
        }
        
        return report
    
    def run_validation(self) -> bool:
        """Run all validation checks."""
        print("🚀 Starting release pipeline validation...")
        print(f"Project root: {self.project_root}")
        
        checks = [
            self.check_workflow_files,
            self.check_workflow_syntax,
            self.check_project_structure,
            self.check_version_consistency,
            self.check_build_dependencies,
            self.check_sidecar_build_setup,
            self.check_tauri_configuration
        ]
        
        all_passed = True
        for check in checks:
            try:
                result = check()
                if not result:
                    all_passed = False
            except Exception as e:
                print(f"❌ Error during validation: {e}")
                all_passed = False
        
        # Generate and display report
        report = self.generate_report()
        
        print(f"\n📊 Validation Summary:")
        print(f"   Total checks: {report['summary']['total_checks']}")
        print(f"   Passed: {report['summary']['passed_checks']}")
        print(f"   Failed: {report['summary']['failed_checks']}")
        print(f"   Success rate: {report['summary']['success_rate']:.1f}%")
        
        if all_passed:
            print("\n🎉 All validation checks passed! The release pipeline is ready.")
        else:
            print("\n⚠️ Some validation checks failed. Please address the issues above.")
        
        return all_passed


def main():
    """Main entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description="Validate release pipeline setup")
    parser.add_argument("--project-root", type=Path,
                       default=Path(__file__).parent.parent,
                       help="Project root directory")
    parser.add_argument("--json-output", type=Path,
                       help="Save validation report as JSON")
    
    args = parser.parse_args()
    
    validator = ReleasePipelineValidator(args.project_root)
    success = validator.run_validation()
    
    if args.json_output:
        report = validator.generate_report()
        with open(args.json_output, 'w') as f:
            json.dump(report, f, indent=2)
        print(f"\n📄 Validation report saved to: {args.json_output}")
    
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()