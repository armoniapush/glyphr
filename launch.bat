@echo off
setlocal
cd /d "%~dp0"

set "EXE_PATH=%~dp0target\release\glyphr.exe"
set "CACHE_DIR=%LOCALAPPDATA%\glyphr\glyphr"

:: 1. Terminar cualquier instancia anterior que siga abierta
taskkill /F /IM glyphr.exe >nul 2>&1
timeout /t 1 /nobreak >nul 2>&1

:: 2. Limpieza de cache si se ejecuta con flag --clean
if /i "%~1"=="--clean" (
    if exist "%CACHE_DIR%" rd /s /q "%CACHE_DIR%" >nul 2>&1
)
if /i "%~1"=="-c" (
    if exist "%CACHE_DIR%" rd /s /q "%CACHE_DIR%" >nul 2>&1
)

:: 3. Verificar o compilar si no existe el binario release
if not exist "%EXE_PATH%" (
    echo [Glyphr] Compilando binario optimizado...
    cargo build --release
    if %ERRORLEVEL% neq 0 (
        echo [Error] No se pudo compilar el ejecutable.
        pause >nul
        exit /b 1
    )
)

:: 4. Lanzar nueva sesion limpia en segundo plano
start "" /D "%~dp0" "%EXE_PATH%"
exit /b 0
