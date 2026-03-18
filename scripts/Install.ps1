# NOTE: clipd currently uses macOS-specific clipboard APIs (objc2-app-kit).
# This script is provided for future Windows support. It will not produce
# a working binary until the clipboard backend is ported.

param(
    [string]$TaskName   = "clipd",
    [string]$BinaryDest = "$env:ProgramFiles\clipd\clipd.exe"
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot  = Split-Path -Parent $ScriptDir
$Template  = Join-Path $RepoRoot "configs\windows\clipd-task.xml"

# --- build ---
Write-Host "Building release binary..."
cargo build --release --manifest-path "$RepoRoot\Cargo.toml"

# --- install binary ---
Write-Host "Installing binary to $BinaryDest..."
$BinaryDir = Split-Path -Parent $BinaryDest
if (-not (Test-Path $BinaryDir)) { New-Item -ItemType Directory -Path $BinaryDir | Out-Null }
Copy-Item -Force "$RepoRoot\target\release\clipd.exe" $BinaryDest

# --- substitute template ---
$User    = "$env:USERDOMAIN\$env:USERNAME"
$TaskXml = (Get-Content $Template -Raw) `
    -replace [regex]::Escape("{{BINARY_PATH}}"), $BinaryDest `
    -replace [regex]::Escape("{{USER}}"),        $User `
    -replace [regex]::Escape("{{TASK_NAME}}"),   $TaskName

$TempXml = Join-Path $env:TEMP "clipd-task.xml"
[System.IO.File]::WriteAllText($TempXml, $TaskXml, [System.Text.Encoding]::Unicode)

# --- register task ---
Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false -ErrorAction SilentlyContinue
Register-ScheduledTask -TaskName $TaskName -Xml $TaskXml -Force | Out-Null

Write-Host "Scheduled task '$TaskName' registered. clipd will start at next logon."
Write-Host "To start now: Start-ScheduledTask -TaskName '$TaskName'"
