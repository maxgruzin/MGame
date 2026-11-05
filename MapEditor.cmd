@echo off
cd /d "%~dp0"
if exist "mgame.exe" (
    start "" "mgame.exe" --editor %*
    exit /b
)
call "%~dp0Play.cmd" --editor %*
