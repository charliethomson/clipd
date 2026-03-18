@echo off
setlocal EnableDelayedExpansion

:: -----------------------------------------------------------------------
:: NOTE: clipd currently uses macOS-specific clipboard APIs (objc2-app-kit).
:: This script is provided for future Windows support. It will not produce
:: a working binary until the clipboard backend is ported.
:: -----------------------------------------------------------------------

set "TASK_NAME=%TASK_NAME:=clipd%"
if "%TASK_NAME%"=="" set "TASK_NAME=clipd"

set "BINARY_DEST=%BINARY_DEST%"
if "%BINARY_DEST%"=="" set "BINARY_DEST=%ProgramFiles%\clipd\clipd.exe"

set "SCRIPT_DIR=%~dp0"
set "REPO_ROOT=%SCRIPT_DIR%.."

:: --- build ---
echo Building release binary...
cargo build --release --manifest-path "%REPO_ROOT%\Cargo.toml"
if %ERRORLEVEL% neq 0 (
    echo Build failed.
    exit /b %ERRORLEVEL%
)

:: --- install binary ---
echo Installing binary to %BINARY_DEST%...
for %%F in ("%BINARY_DEST%") do set "BINARY_DIR=%%~dpF"
if not exist "%BINARY_DIR%" mkdir "%BINARY_DIR%"
copy /y "%REPO_ROOT%\target\release\clipd.exe" "%BINARY_DEST%"

:: --- substitute template and register task ---
set "TEMPLATE=%REPO_ROOT%\configs\windows\clipd-task.xml"
set "TASK_XML=%TEMP%\clipd-task.xml"

:: Use PowerShell to substitute placeholders in the XML template
powershell -NoProfile -Command ^
    "(Get-Content '%TEMPLATE%') ^
        -replace '{{BINARY_PATH}}', '%BINARY_DEST%' ^
        -replace '{{USER}}', $env:USERDOMAIN\$env:USERNAME ^
        -replace '{{TASK_NAME}}', '%TASK_NAME%' ^
    | Set-Content -Encoding Unicode '%TASK_XML%'"

schtasks /delete /tn "%TASK_NAME%" /f >nul 2>&1
schtasks /create /tn "%TASK_NAME%" /xml "%TASK_XML%"
if %ERRORLEVEL% neq 0 (
    echo Failed to register scheduled task.
    exit /b %ERRORLEVEL%
)

echo Scheduled task "%TASK_NAME%" registered. clipd will start at next logon.
echo To start now: schtasks /run /tn "%TASK_NAME%"
endlocal
