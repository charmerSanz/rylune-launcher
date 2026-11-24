@echo off
cd /d "%~dp0"
where node >nul 2>&1 || (echo Install Node.js LTS first. & pause & exit /b 1)
where cargo >nul 2>&1 || (echo Install Rust using rustup and reopen this terminal. & pause & exit /b 1)
if not exist node_modules call npm install
if errorlevel 1 (pause & exit /b 1)
call npm run tauri dev
pause
