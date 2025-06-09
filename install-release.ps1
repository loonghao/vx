# PowerShell installation script for vx
# This script downloads and installs the latest version of vx from GitHub releases

param(
    [string]$Version = "latest",
    [string]$InstallDir = "$env:USERPROFILE\.vx\bin"
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

    # Construct download URL
    $FileName = "vx_${Platform}_${Arch}.zip"
    $DownloadUrl = "https://github.com/$Owner/$Repo/releases/download/$Version/$FileName"
    
    Write-Host "Downloading vx $Version for $Platform $Arch..." -ForegroundColor Yellow
    
    # Download the archive
    $TempFile = [System.IO.Path]::GetTempFileName() + ".zip"
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $TempFile
    
    # Extract the archive
    $TempDir = [System.IO.Path]::GetTempPath() + [System.Guid]::NewGuid().ToString()
    Expand-Archive -Path $TempFile -DestinationPath $TempDir -Force
    
    # Move the binary to the installation directory
    $BinaryPath = Join-Path $TempDir "vx.exe"
    $DestPath = Join-Path $InstallDir "vx.exe"
    
    if (Test-Path $BinaryPath) {
        Move-Item $BinaryPath $DestPath -Force
        Write-Host "vx installed successfully to $DestPath" -ForegroundColor Green
    } else {
        throw "Binary not found in the downloaded archive"
    }
    
    # Clean up
    Remove-Item $TempFile -Force
    Remove-Item $TempDir -Recurse -Force
    
    # Add to PATH if not already there
    $CurrentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    if ($CurrentPath -notlike "*$InstallDir*") {
        [Environment]::SetEnvironmentVariable("PATH", "$CurrentPath;$InstallDir", "User")
        Write-Host "Added $InstallDir to your PATH" -ForegroundColor Green
        Write-Host "Please restart your terminal or run: `$env:PATH += ';$InstallDir'" -ForegroundColor Yellow
    }
    
    Write-Host ""
    Write-Host "Installation complete! Run 'vx --version' to verify." -ForegroundColor Green
    
} catch {
    Write-Error "Installation failed: $_"
    exit 1
}
