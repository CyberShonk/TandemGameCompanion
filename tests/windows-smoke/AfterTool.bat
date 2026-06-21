@echo off
if not "%~1"=="after-bat-arg" exit /b 18
echo after-bat:%~1>>smoke-events.txt
exit /b 0
