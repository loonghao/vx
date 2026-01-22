@echo off
REM VX Shell Environment Initialization for CMD
REM This script is embedded into vx-env binary at compile time

REM The prompt is set via PROMPT environment variable before spawning cmd
REM This file is kept for consistency but cmd.exe doesn't support init scripts well

echo.
echo VX Shell Environment
echo Project: %VX_PROJECT_NAME%
if defined VX_TOOLS echo Tools: %VX_TOOLS%
echo.
