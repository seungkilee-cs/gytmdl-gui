@echo off
REM Cross-platform build script for gytmdl sidecar binaries (Windows version)
REM This script should be run on Windows to generate the Windows binary

setlocal enabledelayedexpansion

echo === gytmdl Sidecar Binary Builder ===

REM Get script directory
set "SCRIPT_DIR=%~dp0"
set "PROJECT_ROOT=%SCRIPT_DIR%.."
set "GYTMDL_SRC=%PROJECT_ROOT%\..\gytmdl"
set "OUTPUT_DIR=%PROJECT_ROOT%\src-tauri\sidecars"

echo Script directory: %SCRIPT_DIR%
echo Project root: %PROJECT_ROOT%
echo gytmdl source: %GYTMDL_SRC%
echo Output directory: %OUTPUT_DIR%

REM Check if Python is available
python --version >nul 2>&1
if %errorlevel% neq 0 (
    python3 --version >nul 2>&1
    if %errorlevel% neq 0 (
        echo Error: Python is not installed or not in PATH
        exit /b 1
    ) else (
        set "PYTHON_CMD=python3"
    )
) else (
    set "PYTHON_CMD=python"
)

echo Using Python: %PYTHON_CMD%

REM Check if pip is available
%PYTHON_CMD% -m pip --version >nul 2>&1
if %errorlevel% neq 0 (
    echo Error: pip is not available
    exit /b 1
)

REM Install PyInstaller if not available
%PYTHON_CMD% -c "import PyInstaller" >nul 2>&1
if %errorlevel% neq 0 (
    echo Installing PyInstaller...
    %PYTHON_CMD% -m pip install pyinstaller
)

REM Create output directory
if not exist "%OUTPUT_DIR%" mkdir "%OUTPUT_DIR%"

REM Run the Python build script
echo Building sidecar binary for Windows...
%PYTHON_CMD% "%SCRIPT_DIR%\build-sidecars.py" --gytmdl-src "%GYTMDL_SRC%" --output-dir "%OUTPUT_DIR%"

if %errorlevel% eq 0 (
    echo Build completed successfully!
    echo Binaries are available in: %OUTPUT_DIR%
    dir "%OUTPUT_DIR%"
) else (
    echo Build failed!
    exit /b 1
)

endlocal