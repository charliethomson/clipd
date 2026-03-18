@echo off
setlocal EnableDelayedExpansion

:: -----------------------------------------------------------------------
:: NOTE: clipd currently uses macOS-specific clipboard APIs (objc2-app-kit).
:: This script is provided for future Windows support.
:: -----------------------------------------------------------------------

if "%TASK_NAME%"=="" set "TASK_NAME=clipd"
if "%BINARY_DEST%"=="" set "BINARY_DEST=%ProgramFiles%\clipd\clipd.exe"

set "REPO=charliethomson/clipd"
set "ASSET=clipd-x86_64-pc-windows-msvc.exe"
set "SCRIPT_DIR=%~dp0"
set "REPO_ROOT=%SCRIPT_DIR%.."

:: --- download latest release ---
echo Fetching latest release (%ASSET%)...
for %%F in ("%BINARY_DEST%") do set "BINARY_DIR=%%~dpF"
if not exist "%BINARY_DIR%" mkdir "%BINARY_DIR%"

powershell -NoProfile -Command ^
    "$url = (Invoke-RestMethod 'https://api.github.com/repos/%REPO%/releases/latest').assets" ^
    " | Where-Object { $_.name -eq '%ASSET%' } | Select-Object -ExpandProperty browser_download_url;" ^
    "if (-not $url) { Write-Error 'Asset not found'; exit 1 };" ^
    "Invoke-WebRequest -Uri $url -OutFile '%BINARY_DEST%'"
if %ERRORLEVEL% neq 0 (
    echo Download failed.
    exit /b %ERRORLEVEL%
)
echo Installed binary to %BINARY_DEST%

:: --- substitute template and register task ---
set "TEMPLATE=%REPO_ROOT%\configs\windows\clipd-task.xml"
set "TASK_XML=%TEMP%\clipd-task.xml"

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
