# -*- mode: python ; coding: utf-8 -*-
"""
PyInstaller spec file for building gytmdl sidecar binaries
This creates platform-specific executables that can be bundled with the Tauri app
"""

import os
import sys
from pathlib import Path

# Get the gytmdl source directory - use a more reliable path detection
import os
spec_dir = os.path.dirname(os.path.abspath(SPEC))
project_root = os.path.dirname(spec_dir)
gytmdl_src = os.path.join(os.path.dirname(project_root), "gytmdl")

if not os.path.exists(gytmdl_src):
    raise FileNotFoundError(f"gytmdl source directory not found at {gytmdl_src}")

# Add gytmdl to Python path
sys.path.insert(0, gytmdl_src)

# Platform-specific binary name
platform_suffix = ""
if sys.platform == "win32":
    if "64" in str(sys.maxsize):
        platform_suffix = "-x86_64-pc-windows-msvc"
    else:
        platform_suffix = "-i686-pc-windows-msvc"
elif sys.platform == "darwin":
    import platform as plat
    if plat.machine() == "arm64":
        platform_suffix = "-aarch64-apple-darwin"
    else:
        platform_suffix = "-x86_64-apple-darwin"
elif sys.platform.startswith("linux"):
    import platform as plat
    if plat.machine() == "x86_64":
        platform_suffix = "-x86_64-unknown-linux-gnu"
    elif plat.machine() == "aarch64":
        platform_suffix = "-aarch64-unknown-linux-gnu"
    else:
        platform_suffix = "-unknown-linux-gnu"

binary_name = f"gytmdl{platform_suffix}"

a = Analysis(
    [os.path.join(spec_dir, "gytmdl_entry.py")],
    pathex=[gytmdl_src, spec_dir],
    binaries=[],
    datas=[
        # Include any data files that gytmdl might need
        (os.path.join(gytmdl_src, "gytmdl"), "gytmdl"),
    ],
    hiddenimports=[
        # Add any hidden imports that gytmdl might need
        'gytmdl',
        'gytmdl.cli',
        'gytmdl.downloader',
        'gytmdl.utils',
        'gytmdl.enums',
        'gytmdl.constants',
        'gytmdl.custom_logger_formatter',
        'gytmdl.models',
        'gytmdl.exceptions',
        # Common dependencies
        'requests',
        'urllib3',
        'certifi',
        'charset_normalizer',
        'idna',
        # Audio processing
        'mutagen',
        # Other potential dependencies
        'json',
        'base64',
        'hashlib',
        'uuid',
        'datetime',
        'pathlib',
        'argparse',
        'logging',
        'concurrent.futures',
        'threading',
        'queue',
    ],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[
        # Exclude unnecessary modules to reduce binary size
        'tkinter',
        'matplotlib',
        'numpy',
        'scipy',
        'pandas',
        'PIL',
        'cv2',
    ],
    win_no_prefer_redirects=False,
    win_private_assemblies=False,
    cipher=None,
    noarchive=False,
)

pyz = PYZ(a.pure, a.zipped_data, cipher=None)

# Set output directory to the sidecars directory
import os
output_dir = os.path.join(os.path.dirname(spec_dir), "src-tauri", "sidecars", "dist")

exe = EXE(
    pyz,
    a.scripts,
    a.binaries,
    a.zipfiles,
    a.datas,
    [],
    name=binary_name,
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,  # Enable UPX compression to reduce binary size
    upx_exclude=[],
    runtime_tmpdir=None,
    console=True,  # gytmdl is a console application
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
    icon=None,  # Add icon if desired
)