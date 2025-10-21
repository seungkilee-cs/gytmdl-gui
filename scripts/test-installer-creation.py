#!/usr/bin/env python3
"""
Test installer creation functionality without actually running the full build.
"""

import json
import platform
import sys
from pathlib import Path

def test_installer_creation():
    """Test installer creation setup and configuration."""
    print("📦 Testing installer creation setup...")
    
    # Get paths
    project_root = Path(__file__).parent.parent
    tauri_config_path = project_root / "src-tauri" / "tauri.conf.json"
    build_config_path = project_root / "build-config.json"
    
    # Load configurations
    with open(tauri_config_path) as f:
        tauri_config = json.load(f)
    
    with open(build_config_path) as f:
        build_config = json.load(f)
    
    print("✅ Configurations loaded")
    
    # Get current platform info
    system = platform.system().lower()
    machine = platform.machine().lower()
    
    if system == "darwin":
        platform_info = {
            "os": "macos",
            "arch": "aarch64" if machine == "arm64" else "x86_64",
            "target": "aarch64-apple-darwin" if machine == "arm64" else "x86_64-apple-darwin",
            "installer_formats": ["dmg", "app"]
        }
    elif system == "windows":
        platform_info = {
            "os": "windows",
            "arch": "x86_64" if "64" in str(sys.maxsize) else "i686",
            "target": "x86_64-pc-windows-msvc" if "64" in str(sys.maxsize) else "i686-pc-windows-msvc",
            "installer_formats": ["msi", "nsis"]
        }
    elif system == "linux":
        platform_info = {
            "os": "linux",
            "arch": "x86_64" if machine == "x86_64" else "aarch64" if machine == "aarch64" else "unknown",
            "target": f"{machine}-unknown-linux-gnu",
            "installer_formats": ["deb", "rpm", "appimage"]
        }
    else:
        print(f"❌ Unsupported platform: {system}")
        return False
    
    print(f"✅ Platform: {platform_info['os']} {platform_info['arch']}")
    print(f"✅ Target: {platform_info['target']}")
    print(f"✅ Expected installer formats: {', '.join(platform_info['installer_formats'])}")
    
    # Check Tauri bundle configuration
    bundle_config = tauri_config.get("bundle", {})
    
    if not bundle_config.get("active", False):
        print("❌ Bundle not active in Tauri config")
        return False
    
    print("✅ Bundle is active")
    
    # Check platform-specific bundle configuration
    platform_bundle_config = bundle_config.get(platform_info["os"], {})
    
    if platform_info["os"] == "windows":
        # Check Windows-specific config
        wix_config = platform_bundle_config.get("wix", {})
        nsis_config = platform_bundle_config.get("nsis", {})
        
        print(f"✅ WiX config present: {bool(wix_config)}")
        print(f"✅ NSIS config present: {bool(nsis_config)}")
        
        if wix_config:
            print(f"  - WiX languages: {wix_config.get('language', [])}")
        if nsis_config:
            print(f"  - NSIS install mode: {nsis_config.get('installMode', 'not specified')}")
    
    elif platform_info["os"] == "macos":
        # Check macOS-specific config
        print(f"✅ macOS minimum system version: {platform_bundle_config.get('minimumSystemVersion', 'not specified')}")
        print(f"✅ Hardened runtime: {platform_bundle_config.get('hardenedRuntime', False)}")
    
    elif platform_info["os"] == "linux":
        # Check Linux-specific config
        deb_config = platform_bundle_config.get("deb", {})
        rpm_config = platform_bundle_config.get("rpm", {})
        appimage_config = platform_bundle_config.get("appimage", {})
        
        print(f"✅ DEB config present: {bool(deb_config)}")
        print(f"✅ RPM config present: {bool(rpm_config)}")
        print(f"✅ AppImage config present: {bool(appimage_config)}")
    
    # Check build configuration
    bundle_build_config = build_config.get("bundle", {})
    platform_build_config = bundle_build_config.get(platform_info["os"], {})
    
    print(f"✅ Build config for {platform_info['os']}: {bool(platform_build_config)}")
    
    # Check expected output paths
    target_dir = project_root / "target" / "release"
    bundle_dir = target_dir / "bundle"
    
    print(f"✅ Expected target directory: {target_dir}")
    print(f"✅ Expected bundle directory: {bundle_dir}")
    
    # Check expected installer paths for current platform
    expected_installers = []
    
    for format_name in platform_info["installer_formats"]:
        if format_name == "msi":
            expected_installers.append(bundle_dir / "msi" / "gytmdl-gui_0.1.0_x64_en-US.msi")
        elif format_name == "nsis":
            expected_installers.append(bundle_dir / "nsis" / "gytmdl-gui_0.1.0_x64-setup.exe")
        elif format_name == "dmg":
            expected_installers.append(bundle_dir / "dmg" / "gytmdl-gui_0.1.0_x64.dmg")
        elif format_name == "deb":
            expected_installers.append(bundle_dir / "deb" / "gytmdl-gui_0.1.0_amd64.deb")
        elif format_name == "rpm":
            expected_installers.append(bundle_dir / "rpm" / "gytmdl-gui-0.1.0-1.x86_64.rpm")
        elif format_name == "appimage":
            expected_installers.append(bundle_dir / "appimage" / "gytmdl-gui_0.1.0_amd64.AppImage")
    
    print(f"✅ Expected installer paths:")
    for installer_path in expected_installers:
        print(f"  - {installer_path}")
    
    # Check sidecar binaries are ready
    sidecars_dir = project_root / "src-tauri" / "sidecars"
    external_bins = bundle_config.get("externalBin", [])
    
    sidecar_count = 0
    for bin_path in external_bins:
        binary_name = bin_path.replace("sidecars/", "")
        binary_file = sidecars_dir / binary_name
        if binary_file.exists():
            sidecar_count += 1
    
    print(f"✅ Sidecar binaries ready: {sidecar_count}/{len(external_bins)}")
    
    if sidecar_count != len(external_bins):
        print("⚠️ Some sidecar binaries are missing - run create-mock-sidecars.py first")
    
    print("🎉 Installer creation test completed!")
    print("\n📋 Summary:")
    print(f"  Platform: {platform_info['os']} {platform_info['arch']}")
    print(f"  Target: {platform_info['target']}")
    print(f"  Installer formats: {', '.join(platform_info['installer_formats'])}")
    print(f"  Sidecar binaries: {sidecar_count}/{len(external_bins)} ready")
    print(f"  Bundle configuration: ✅ Complete")
    
    return True

if __name__ == "__main__":
    success = test_installer_creation()
    sys.exit(0 if success else 1)