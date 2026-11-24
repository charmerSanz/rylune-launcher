@echo off
cd /d "%~dp0"
where node >nul 2>&1 || (echo Install Node.js LTS first. & pause & exit /b 1)
where cargo >nul 2>&1 || (echo Install Rust using rustup and reopen this terminal. & pause & exit /b 1)
call npm install
if errorlevel 1 (pause & exit /b 1)
call npm run tauri build
if errorlevel 1 (pause & exit /b 1)
echo Installer: src-tauri\target\release\bundle\nsis
pause
