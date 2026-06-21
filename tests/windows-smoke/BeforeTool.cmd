@echo off
if not "%~1"=="before-cmd-arg" exit /b 17
echo before-cmd:%~1>>smoke-events.txt
exit /b 0
