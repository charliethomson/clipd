# NOTE: clipd currently uses macOS-specific clipboard APIs (objc2-app-kit).
# This script is provided for future Windows support.

param(
    [string]$TaskName   = "clipd",
    [string]$BinaryDest = "$env:ProgramFiles\clipd\clipd.exe"
)

$ErrorActionPreference = "Stop"

$Repo  = "charliethomson/clipd"
$Asset = "clipd-x86_64-pc-windows-msvc.exe"
$Raw   = "https://raw.githubusercontent.com/$Repo/main"

# --- download latest release binary ---
Write-Host "Fetching latest release ($Asset)..."
$Release = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest"
$Url = $Release.assets | Where-Object { $_.name -eq $Asset } | Select-Object -ExpandProperty browser_download_url

if (-not $Url) { Write-Error "Could not find release asset $Asset"; exit 1 }

$BinaryDir = Split-Path -Parent $BinaryDest
if (-not (Test-Path $BinaryDir)) { New-Item -ItemType Directory -Path $BinaryDir | Out-Null }
Invoke-WebRequest -Uri $Url -OutFile $BinaryDest
Write-Host "Installed binary to $BinaryDest"

# --- download and substitute task template ---
$User    = "$env:USERDOMAIN\$env:USERNAME"
$TaskXml = (Invoke-WebRequest "$Raw/configs/windows/clipd-task.xml").Content `
    -replace [regex]::Escape("{{BINARY_PATH}}"), $BinaryDest `
    -replace [regex]::Escape("{{USER}}"),        $User `
    -replace [regex]::Escape("{{TASK_NAME}}"),   $TaskName

# --- register task ---
Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false -ErrorAction SilentlyContinue
Register-ScheduledTask -TaskName $TaskName -Xml $TaskXml -Force | Out-Null

Write-Host "Scheduled task '$TaskName' registered. clipd will start at next logon."
Write-Host "To start now: Start-ScheduledTask -TaskName '$TaskName'"
