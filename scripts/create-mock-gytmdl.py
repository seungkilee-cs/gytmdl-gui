#!/usr/bin/env python3
"""
Create a mock gytmdl binary for testing the GUI without needing the full gytmdl setup.
This mock binary simulates the behavior of gytmdl for UI testing purposes.
"""

import os
import sys
import time
import random
from pathlib import Path

def create_mock_binary():
    """Create a mock gytmdl binary that simulates download progress."""
    
    # Determine the correct binary name for the current platform
    if sys.platform == "win32":
        if "64" in str(sys.maxsize):
            binary_name = "gytmdl-x86_64-pc-windows-msvc.exe"
        else:
            binary_name = "gytmdl.exe"
    elif sys.platform == "darwin":
        import platform
        if platform.machine() == "arm64":
            binary_name = "gytmdl-aarch64-apple-darwin"
        else:
            binary_name = "gytmdl-x86_64-apple-darwin"
    elif sys.platform.startswith("linux"):
        import platform
        if platform.machine() == "x86_64":
            binary_name = "gytmdl-x86_64-unknown-linux-gnu"
        else:
            binary_name = "gytmdl"
    else:
        binary_name = "gytmdl"
    
    # Create the mock binary script
    mock_script = f'''#!/usr/bin/env python3
"""
Mock gytmdl binary for testing gytmdl-gui.
This simulates the behavior of the real gytmdl binary.
"""

import sys
import time
import random
import os

def main():
    args = sys.argv[1:]
    
    # Handle --version command
    if "--version" in args:
        print("gytmdl 1.0.0 (mock version for testing)")
        return 0
    
    # Handle --help command
    if "--help" in args or "-h" in args:
        print("Mock gytmdl - YouTube Music Downloader (Testing Version)")
        print("Usage: gytmdl [options] URL")
        print("Options:")
        print("  --version          Show version")
        print("  --help, -h         Show this help")
        print("  --output-path DIR  Output directory")
        print("  --temp-path DIR    Temporary directory")
        print("  --progress         Show progress")
        print("  --verbose          Verbose output")
        return 0
    
    # Find the URL (last argument that looks like a URL)
    url = None
    for arg in reversed(args):
        if arg.startswith("http"):
            url = arg
            break
    
    if not url:
        print("Error: No URL provided", file=sys.stderr)
        return 1
    
    # Simulate download process
    print(f"Starting download: {{url}}")
    print("Initializing...")
    time.sleep(0.5)
    
    # Simulate progress updates
    stages = [
        "Extracting video info",
        "Downloading audio",
        "Processing metadata",
        "Applying tags",
        "Finalizing"
    ]
    
    for i, stage in enumerate(stages):
        print(f"[INFO] {{stage}}...")
        
        # Simulate progress within each stage
        for progress in range(0, 101, random.randint(5, 15)):
            if progress > 100:
                progress = 100
            print(f"Progress: {{progress}}% - {{stage}}")
            time.sleep(0.1)
        
        time.sleep(0.2)
    
    print("Download completed successfully!")
    print(f"Saved to: mock_output/{{url.split('/')[-1]}}.mp3")
    
    return 0

if __name__ == "__main__":
    sys.exit(main())
'''
    
    # Create the sidecars directory
    project_root = Path(__file__).parent.parent
    sidecars_dir = project_root / "src-tauri" / "sidecars"
    sidecars_dir.mkdir(parents=True, exist_ok=True)
    
    # Write the mock binary
    binary_path = sidecars_dir / binary_name
    with open(binary_path, "w") as f:
        f.write(mock_script)
    
    # Make it executable on Unix systems
    if os.name == 'posix':
        os.chmod(binary_path, 0o755)
    
    # Create a simple manifest
    manifest_content = f'''{{
    "binary_name": "{binary_name}",
    "platform": {{
        "os": "{sys.platform}",
        "arch": "{sys.maxsize}",
        "target": "mock-target",
        "extension": "{'.exe' if sys.platform == 'win32' else ''}"
    }},
    "size_bytes": {binary_path.stat().st_size},
    "sha256": "mock-hash-for-testing",
    "build_timestamp": "2024-01-01T00:00:00Z"
}}'''
    
    manifest_path = binary_path.with_suffix(".json")
    with open(manifest_path, "w") as f:
        f.write(manifest_content)
    
    print(f"✅ Created mock gytmdl binary: {binary_path}")
    print(f"✅ Created manifest: {manifest_path}")
    print(f"📁 Sidecar directory: {sidecars_dir}")
    
    return binary_path

def main():
    """Main entry point."""
    print("🔨 Creating mock gytmdl binary for testing...")
    
    try:
        binary_path = create_mock_binary()
        print(f"\\n🎉 Mock binary created successfully!")
        print(f"\\nNow you can test the gytmdl-gui application:")
        print(f"1. Run: npm run tauri dev")
        print(f"2. Add YouTube Music URLs to the queue")
        print(f"3. Watch the mock download progress")
        print(f"\\nNote: This is a mock binary for testing UI functionality only.")
        print(f"For real downloads, you need to build the actual gytmdl sidecar binaries.")
        
    except Exception as e:
        print(f"❌ Failed to create mock binary: {e}")
        return 1
    
    return 0

if __name__ == "__main__":
    sys.exit(main())