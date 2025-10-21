#!/usr/bin/env python3
"""
Test Tauri configuration and sidecar detection.
"""

import json
import sys
from pathlib import Path

def test_tauri_config():
    """Test that Tauri configuration is correct for sidecar bundling."""
    print("🔍 Testing Tauri configuration...")
    
    # Get paths
    project_root = Path(__file__).parent.parent
    tauri_config_path = project_root / "src-tauri" / "tauri.conf.json"
    sidecars_dir = project_root / "src-tauri" / "sidecars"
    
    # Load Tauri config
    if not tauri_config_path.exists():
        print(f"❌ Tauri config not found: {tauri_config_path}")
        return False
    
    with open(tauri_config_path) as f:
        config = json.load(f)
    
    print("✅ Tauri config loaded")
    
    # Check external binaries configuration
    external_bins = config.get("bundle", {}).get("externalBin", [])
    
    if not external_bins:
        print("❌ No external binaries configured")
        return False
    
    print(f"✅ Found {len(external_bins)} external binaries configured:")
    for bin_path in external_bins:
        print(f"  - {bin_path}")
    
    # Check that sidecar binaries exist
    missing_binaries = []
    found_binaries = []
    
    for bin_path in external_bins:
        # Remove 'sidecars/' prefix if present
        if bin_path.startswith("sidecars/"):
            binary_name = bin_path[9:]  # Remove 'sidecars/' prefix
        else:
            binary_name = bin_path
        
        binary_file = sidecars_dir / binary_name
        
        if binary_file.exists():
            size = binary_file.stat().st_size
            found_binaries.append(f"{binary_name} ({size} bytes)")
        else:
            missing_binaries.append(binary_name)
    
    if missing_binaries:
        print(f"❌ Missing sidecar binaries:")
        for binary in missing_binaries:
            print(f"  - {binary}")
        return False
    
    print(f"✅ All sidecar binaries found:")
    for binary in found_binaries:
        print(f"  - {binary}")
    
    # Check bundle configuration
    bundle_config = config.get("bundle", {})
    
    print(f"✅ Bundle active: {bundle_config.get('active', False)}")
    print(f"✅ Bundle targets: {bundle_config.get('targets', 'not specified')}")
    
    # Check platform-specific configurations
    platforms = ["windows", "macOS", "linux"]
    for platform in platforms:
        if platform in bundle_config:
            print(f"✅ {platform} bundle config present")
        else:
            print(f"⚠️ {platform} bundle config missing")
    
    print("🎉 Tauri configuration test completed successfully!")
    return True

if __name__ == "__main__":
    success = test_tauri_config()
    sys.exit(0 if success else 1)