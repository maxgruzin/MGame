@echo off
cd /d "%~dp0"
if exist "dist\MGame-Overdrive\mgame.exe" (
    start "" "dist\MGame-Overdrive\mgame.exe" %*
    exit /b
)
if exist "target\release\mgame.exe" (
    start "" "target\release\mgame.exe" %*
    exit /b
)
echo Build the Rust game first:
echo powershell -ExecutionPolicy Bypass -File scripts\build.ps1
pause
