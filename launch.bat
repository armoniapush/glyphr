@echo off
setlocal
cd /d "%~dp0"

set "EXE_PATH=%~dp0target\release\antigravity-fonts.exe"

if exist "%EXE_PATH%" (
    start "" "%EXE_PATH%"
    exit /b 0
)

echo [Antigravity Fonts] Release binary not found. Building now...
cargo build --release
if %ERRORLEVEL% equ 0 (
    start "" "%EXE_PATH%"
) else (
    echo [Error] Build failed. Press any key to exit.
    pause >nul
)
