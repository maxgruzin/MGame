@echo off
setlocal
cd /d "%~dp0"
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\build.ps1" -Check %*
set "buildResult=%errorlevel%"
if "%buildResult%"=="0" (
    echo.
    echo Rebuild complete. Launch the updated game with Play.cmd.
) else (
    echo.
    echo Rebuild failed. See the error above.
)
pause
exit /b %buildResult%
