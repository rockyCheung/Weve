@echo off
setlocal
cd /d "%~dp0"

echo ===== Build: Small Business Manager (Windows) =====

where node >nul 2>nul
if errorlevel 1 (
  echo ERROR: Node.js not found in PATH.
  exit /b 1
)

where npm >nul 2>nul
if errorlevel 1 (
  echo ERROR: npm not found in PATH.
  exit /b 1
)

where cargo >nul 2>nul
if errorlevel 1 (
  echo ERROR: Rust/Cargo not found in PATH.
  exit /b 1
)

echo [1/3] Installing frontend dependencies...
call npm install
if errorlevel 1 goto :fail

echo [2/3] Building frontend...
call npm run build
if errorlevel 1 goto :fail

echo [3/3] Building Tauri app...
call npm run tauri build
if errorlevel 1 goto :fail

echo ===== Build finished =====
echo Bundles: src-tauri\target\release\bundle\
exit /b 0

:fail
echo ===== Build failed =====
exit /b 1
