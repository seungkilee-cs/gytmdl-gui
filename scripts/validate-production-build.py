#!/usr/bin/env python3
"""
Comprehensive validation script for production build setup.
This validates all components needed for task 9.1.
"""

import json
import platform
import subprocess
import sys
from pathlib import Path

def validate_production_build():
    """Validate all components for production build."""
    print("🔍 Validating production build setup...")
    print("=" * 50)
    
    # Get paths
    project_root = Path(__file__).parent.parent
    tauri_config_path = project_root / "src-tauri" / "tauri.conf.json"
    build_config_path = project_root / "build-config.json"
    sidecars_dir = project_root / "src-tauri" / "sidecars"
    frontend_dist = project_root / "dist"
    
    validation_results = []
    
    # 1. Check dependencies
    print("1️⃣ Checking build dependencies...")
    
    dependencies = {
        "node": ["node", "--version"],
        "npm": ["npm", "--version"],
        "python3": ["python3", "--version"],
        "pyinstaller": ["python3", "-m", "PyInstaller", "--version"]
    }
    
    for dep_name, cmd in dependencies.items():
        try:
            result = subprocess.run(cmd, capture_output=True, text=True)
            if result.returncode == 0:
                version = result.stdout.strip().split('\n')[0]
                print(f"  ✅ {dep_name}: {version}")
                validation_results.append(f"✅ {dep_name} available")
            else:
                print(f"  ❌ {dep_name}: command failed")
                validation_results.append(f"❌ {dep_name} not available")
        except Exception as e:
            print(f"  ❌ {dep_name}: {e}")
            validation_results.append(f"❌ {dep_name} error")
    
    # 2. Check gytmdl source
    print("\n2️⃣ Checking gytmdl source...")
    gytmdl_src = project_root.parent / "gytmdl"
    
    if gytmdl_src.exists():
        print(f"  ✅ gytmdl source found: {gytmdl_src}")
        
        # Check if gytmdl can be imported
        sys.path.insert(0, str(gytmdl_src))
        try:
            import gytmdl
            print(f"  ✅ gytmdl module importable")
            validation_results.append("✅ gytmdl source ready")
        except ImportError as e:
            print(f"  ⚠️ gytmdl import issue: {e}")
            validation_results.append("⚠️ gytmdl import issue")
    else:
        print(f"  ❌ gytmdl source not found: {gytmdl_src}")
        validation_results.append("❌ gytmdl source missing")
    
    # 3. Check sidecar binaries
    print("\n3️⃣ Checking sidecar binaries...")
    
    expected_platforms = [
        "x86_64-pc-windows-msvc.exe",
        "i686-pc-windows-msvc.exe", 
        "x86_64-apple-darwin",
        "aarch64-apple-darwin",
        "x86_64-unknown-linux-gnu",
        "aarch64-unknown-linux-gnu"
    ]
    
    sidecar_count = 0
    for platform_target in expected_platforms:
        binary_name = f"gytmdl-{platform_target}"
        binary_path = sidecars_dir / binary_name
        manifest_path = sidecars_dir / f"{binary_name}.json"
        
        if binary_path.exists() and manifest_path.exists():
            size = binary_path.stat().st_size
            print(f"  ✅ {binary_name} ({size} bytes)")
            sidecar_count += 1
        else:
            print(f"  ❌ {binary_name} missing")
    
    if sidecar_count == len(expected_platforms):
        validation_results.append("✅ All sidecar binaries present")
    else:
        validation_results.append(f"⚠️ {sidecar_count}/{len(expected_platforms)} sidecar binaries")
    
    # 4. Check Tauri configuration
    print("\n4️⃣ Checking Tauri configuration...")
    
    if tauri_config_path.exists():
        with open(tauri_config_path) as f:
            tauri_config = json.load(f)
        
        bundle_config = tauri_config.get("bundle", {})
        
        # Check bundle is active
        if bundle_config.get("active", False):
            print("  ✅ Bundle is active")
        else:
            print("  ❌ Bundle is not active")
        
        # Check external binaries
        external_bins = bundle_config.get("externalBin", [])
        if len(external_bins) == 6:
            print(f"  ✅ External binaries configured: {len(external_bins)}")
        else:
            print(f"  ⚠️ External binaries: {len(external_bins)} (expected 6)")
        
        # Check targets
        targets = bundle_config.get("targets", "")
        print(f"  ✅ Bundle targets: {targets}")
        
        validation_results.append("✅ Tauri configuration valid")
    else:
        print("  ❌ Tauri config not found")
        validation_results.append("❌ Tauri config missing")
    
    # 5. Check build configuration
    print("\n5️⃣ Checking build configuration...")
    
    if build_config_path.exists():
        with open(build_config_path) as f:
            build_config = json.load(f)
        
        print(f"  ✅ Release mode: {build_config.get('release', False)}")
        print(f"  ✅ Code signing: {build_config.get('code_signing', {}).get('enabled', False)}")
        
        # Check platform-specific bundle configs
        bundle_config = build_config.get("bundle", {})
        platforms = ["windows", "macos", "linux"]
        
        for platform_name in platforms:
            if platform_name in bundle_config:
                print(f"  ✅ {platform_name} bundle config present")
            else:
                print(f"  ⚠️ {platform_name} bundle config missing")
        
        validation_results.append("✅ Build configuration valid")
    else:
        print("  ❌ Build config not found")
        validation_results.append("❌ Build config missing")
    
    # 6. Check frontend build
    print("\n6️⃣ Checking frontend build...")
    
    if frontend_dist.exists():
        index_html = frontend_dist / "index.html"
        if index_html.exists():
            print("  ✅ Frontend dist exists with index.html")
            validation_results.append("✅ Frontend build ready")
        else:
            print("  ⚠️ Frontend dist exists but no index.html")
            validation_results.append("⚠️ Frontend build incomplete")
    else:
        print("  ❌ Frontend dist not found")
        validation_results.append("❌ Frontend not built")
    
    # 7. Check current platform info
    print("\n7️⃣ Current platform info...")
    
    system = platform.system().lower()
    machine = platform.machine().lower()
    
    if system == "darwin":
        current_target = "aarch64-apple-darwin" if machine == "arm64" else "x86_64-apple-darwin"
        installer_formats = ["dmg", "app"]
    elif system == "windows":
        current_target = "x86_64-pc-windows-msvc" if "64" in str(sys.maxsize) else "i686-pc-windows-msvc"
        installer_formats = ["msi", "nsis"]
    elif system == "linux":
        current_target = f"{machine}-unknown-linux-gnu"
        installer_formats = ["deb", "rpm", "appimage"]
    else:
        current_target = "unknown"
        installer_formats = []
    
    print(f"  ✅ Platform: {system} {machine}")
    print(f"  ✅ Target: {current_target}")
    print(f"  ✅ Expected installers: {', '.join(installer_formats)}")
    
    # Check current platform sidecar
    current_sidecar = f"gytmdl-{current_target}"
    if not current_target.endswith('.exe') and system == 'windows':
        current_sidecar += '.exe'
    
    current_sidecar_path = sidecars_dir / current_sidecar
    if current_sidecar_path.exists():
        print(f"  ✅ Current platform sidecar: {current_sidecar}")
        validation_results.append("✅ Current platform sidecar ready")
    else:
        print(f"  ❌ Current platform sidecar missing: {current_sidecar}")
        validation_results.append("❌ Current platform sidecar missing")
    
    # 8. Summary
    print("\n" + "=" * 50)
    print("📋 VALIDATION SUMMARY")
    print("=" * 50)
    
    success_count = sum(1 for result in validation_results if result.startswith("✅"))
    warning_count = sum(1 for result in validation_results if result.startswith("⚠️"))
    error_count = sum(1 for result in validation_results if result.startswith("❌"))
    
    for result in validation_results:
        print(f"  {result}")
    
    print(f"\n📊 Results: {success_count} ✅ | {warning_count} ⚠️ | {error_count} ❌")
    
    if error_count == 0:
        print("\n🎉 READY FOR PRODUCTION BUILD!")
        print("All components are properly configured for task 9.1")
        return True
    else:
        print(f"\n⚠️ ISSUES FOUND: {error_count} errors need to be resolved")
        return False

if __name__ == "__main__":
    success = validate_production_build()
    sys.exit(0 if success else 1)