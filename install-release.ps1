# vx Universal Development Tool Manager Installation Script for Windows
# This script detects your platform and installs vx using the appropriate package manager

param(
    [string]$Version = "latest",
    [string]$InstallDir = "$env:USERPROFILE\.vx\bin",
    [string]$PackageManager = "auto",
    [switch]$NoPackageManager,
    [switch]$Help
)

$ErrorActionPreference = "Stop"

# GitHub repository information
$Owner = "loonghao"
$Repo = "vx"

Write-Host "Installing vx..." -ForegroundColor Green

# Determine the architecture
$Arch = if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { "i386" }
$Platform = "Windows"

# Create installation directory
if (!(Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

try {
    if ($Version -eq "latest") {
        # Get the latest release
        $LatestRelease = Invoke-RestMethod -Uri "https://api.github.com/repos/$Owner/$Repo/releases/latest"
        $Version = $LatestRelease.tag_name
    }

    # Construct download URL based on platform
    if ($Platform -eq "Windows") {
        $FileName = "vx-windows-amd64.exe"
        $DownloadUrl = "https://github.com/$Owner/$Repo/releases/download/$Version/$FileName"
    }
    else {
        throw "Unsupported platform: $Platform"
    }
    
    Write-Host "Downloading vx $Version for $Platform $Arch..." -ForegroundColor Yellow

    # Download the binary directly
    $DestPath = Join-Path $InstallDir "vx.exe"
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $DestPath

    # Verify the download
    if (Test-Path $DestPath) {
        Write-Host "vx installed successfully to $DestPath" -ForegroundColor Green
    }
    else {
        throw "Failed to download binary"
    }
    
    # Add to PATH if not already there
    $CurrentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    if ($CurrentPath -notlike "*$InstallDir*") {
        [Environment]::SetEnvironmentVariable("PATH", "$CurrentPath;$InstallDir", "User")
        Write-Host "Added $InstallDir to your PATH" -ForegroundColor Green
        Write-Host "Please restart your terminal or run: `$env:PATH += ';$InstallDir'" -ForegroundColor Yellow
    }
    
    Write-Host ""
    Write-Host "Installation complete! Run 'vx --version' to verify." -ForegroundColor Green
    
}
catch {
    Write-Error "Installation failed: $_"
    exit 1
}
