<#
.SYNOPSIS
    vitusOS ISO Image Builder (PowerShell wrapper for WSL2 / Linux builder).
.DESCRIPTION
    Automates generating vitusOS 8-10GB Grand Payload bootable ISOs from Windows.
.PARAMETER Channel
    Release channel: 'upstreamColor' (default) or 'upstreamOne'
.PARAMETER Version
    Release version (e.g. '0.0.1' or '1.0.0')
.PARAMETER Arch
    Target architecture (default: 'x86_64_amd64')
.EXAMPLE
    .\Build-Iso.ps1 -Channel upstreamColor -Version 0.0.1
    .\Build-Iso.ps1 -Channel upstreamOne -Version 1.0.0
#>

[CmdletBinding()]
param(
    [ValidateSet("upstreamColor", "upstreamOne")]
    [string]$Channel = "upstreamColor",

    [string]$Version = "0.0.1",

    [string]$Arch = "x86_64_amd64",

    [string]$OutputDir = "$PSScriptRoot\..\..\out"
)

$ErrorActionPreference = "Stop"

Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "                   vitusOS Grand Payload ISO Builder (PowerShell)               " -ForegroundColor Cyan
Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host " Channel:     $Channel" -ForegroundColor Green
Write-Host " Version:     $Version" -ForegroundColor Green
Write-Host " Target Arch: $Arch" -ForegroundColor Green
Write-Host " Output ISO:  $OutputDir\vitusOS_${Channel}_${Version}_${Arch}.iso" -ForegroundColor Green
Write-Host "================================================================================" -ForegroundColor Cyan

# Check for WSL2
$wslAvailable = Get-Command wsl.exe -ErrorAction SilentlyContinue
if (-not $wslAvailable) {
    Write-Error "WSL2 (Windows Subsystem for Linux) is required to build the Ubuntu-based ISO on Windows."
    exit 1
}

$scriptLinuxPath = "/mnt/c/" + ($PSScriptRoot -replace '^[A-Z]:\\', '' -replace '\\', '/') + "/build_iso.sh"
$outputLinuxPath = "/mnt/c/" + ($OutputDir -replace '^[A-Z]:\\', '' -replace '\\', '/')

Write-Host "Invoking Linux build script via WSL2..." -ForegroundColor Yellow
wsl.exe -u root bash -c "chmod +x '$scriptLinuxPath' && '$scriptLinuxPath' --channel '$Channel' --version '$Version' --arch '$Arch' --output-dir '$outputLinuxPath'"

if ($LASTEXITCODE -eq 0) {
    Write-Host "================================================================================" -ForegroundColor Green
    Write-Host " ISO Build Completed Successfully!" -ForegroundColor Green
    Write-Host " Output location: $OutputDir\vitusOS_${Channel}_${Version}_${Arch}.iso" -ForegroundColor Green
    Write-Host "================================================================================" -ForegroundColor Green
} else {
    Write-Error "ISO build failed with exit code $LASTEXITCODE"
}
