# vx installer script for Windows
# Usage: powershell -c "irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex"
# Usage with version: $env:VX_VERSION="0.1.0"; powershell -c "irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex"

param(
    [string]$Version = $env:VX_VERSION,
    [string]$InstallDir = $env:VX_INSTALL_DIR,
    [switch]$BuildFromSource = $false
)

# Default values
if (-not $Version) {
    $Version = "latest"
}

if (-not $InstallDir) {
    $InstallDir = "$env:USERPROFILE\.local\bin"
}

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$RepoOwner = "loonghao"
$RepoName = "vx"
$BaseUrl = "https://github.com/$RepoOwner/$RepoName/releases"

# Logging functions
function Write-Info {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Blue
}

function Write-Warn {
    param([string]$Message)
    Write-Host "[WARN] $Message" -ForegroundColor Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

function Write-Success {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor Green
}

# Detect platform and map to release naming convention
function Get-Platform {
    $arch = if ([Environment]::Is64BitOperatingSystem) { "x64" } else { "x86" }
    return "windows-$arch"
}

# Get latest version from GitHub API
function Get-LatestVersion {
    try {
        $apiUrl = "https://api.github.com/repos/$RepoOwner/$RepoName/releases/latest"
        $response = Invoke-RestMethod -Uri $apiUrl -Method Get
        return $response.tag_name -replace '^v', ''
    }
    catch {
        Write-Error "Failed to get latest version: $_"
        exit 1
    }
}

# Build from source (fallback method)
function Build-FromSource {
    Write-Info "Building vx from source..."

    # Check if Rust is installed
    if (!(Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Error "Rust is not installed. Please install Rust first: https://rustup.rs/"
        exit 1
    }

    # Check if we're in the vx repository
    if (!(Test-Path "Cargo.toml")) {
        Write-Error "Not in vx repository. Please clone the repository first:"
        Write-Host "  git clone https://github.com/$RepoOwner/$RepoName.git"
        Write-Host "  cd $RepoName"
        Write-Host "  .\install.ps1 -BuildFromSource"
        exit 1
    }

    # Build the project
    Write-Info "Building vx..."
    cargo build --release

    if ($LASTEXITCODE -ne 0) {
        Write-Error "Build failed!"
        exit 1
    }

    # Create installation directory
    if (!(Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    }

    # Copy the binary
    Copy-Item "target\release\vx.exe" "$InstallDir\vx.exe" -Force
    Write-Success "vx built and installed from source to: $InstallDir"
}

# Download and install vx from GitHub releases
function Install-FromRelease {
    $platform = Get-Platform

    if ($Version -eq "latest") {
        Write-Info "Fetching latest version..."
        $Version = Get-LatestVersion
        if (-not $Version) {
            Write-Error "Failed to get latest version"
            exit 1
        }
    }

    Write-Info "Installing vx v$Version for $platform..."

    # Construct download URL
    $archiveName = "vx-$platform.zip"
    $downloadUrl = "$BaseUrl/download/v$Version/$archiveName"

    # Create temporary directory
    $tempDir = New-TemporaryFile | ForEach-Object { Remove-Item $_; New-Item -ItemType Directory -Path $_ }

    try {
        # Download
        Write-Info "Downloading from $downloadUrl..."
        $archivePath = Join-Path $tempDir $archiveName
        Invoke-WebRequest -Uri $downloadUrl -OutFile $archivePath -UseBasicParsing

        # Extract
        Write-Info "Extracting to $InstallDir..."
        if (-not (Test-Path $InstallDir)) {
            New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        }

        Expand-Archive -Path $archivePath -DestinationPath $tempDir -Force

        # Find and copy the binary
        $binaryPath = Get-ChildItem -Path $tempDir -Name "vx.exe" -Recurse | Select-Object -First 1
        if (-not $binaryPath) {
            Write-Error "vx.exe not found in archive"
            exit 1
        }

        $sourcePath = Join-Path $tempDir $binaryPath
        $destPath = Join-Path $InstallDir "vx.exe"
        Copy-Item -Path $sourcePath -Destination $destPath -Force

        Write-Success "vx v$Version installed to $destPath"
    }
    catch {
        Write-Warn "Failed to download pre-built binary: $_"
        Write-Info "Falling back to building from source..."
        Build-FromSource
        return
    }
    finally {
        # Cleanup
        Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

# Update PATH environment variable
function Update-Path {
    param([string]$InstallPath)

    # Check if install directory is in PATH
    $currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    if ($currentPath -notlike "*$InstallPath*") {
        Write-Warn "Add $InstallPath to your PATH to use vx from anywhere:"
        Write-Host "  Run this command in an elevated PowerShell:"
        Write-Host "  [Environment]::SetEnvironmentVariable('PATH', `$env:PATH + ';$InstallPath', 'User')"
        Write-Host ""
        Write-Host "Or add it manually through System Properties > Environment Variables"
    }
}

# Verify installation
function Test-Installation {
    param([string]$BinaryPath)

    try {
        & $BinaryPath --version | Out-Null
        Write-Success "Installation verified successfully!"
        Write-Host ""
        Write-Host "🎉 vx is ready to use!"
        Write-Host "📖 Try these commands:"
        Write-Host "   vx --help" -ForegroundColor Gray
        Write-Host "   vx list" -ForegroundColor Gray
        Write-Host "   vx npm --version" -ForegroundColor Gray
        Write-Host "   vx uv --version" -ForegroundColor Gray
    }
    catch {
        Write-Error "Installation verification failed: $_"
        exit 1
    }
}

# Main execution function
function Main {
    Write-Info "vx installer"
    Write-Host ""

    # Check PowerShell version
    if ($PSVersionTable.PSVersion.Major -lt 5) {
        Write-Error "PowerShell 5.0 or later is required"
        exit 1
    }

    # Install vx
    if ($BuildFromSource) {
        Build-FromSource
    }
    else {
        Install-FromRelease
    }

    # Update PATH and verify installation
    $binaryPath = Join-Path $InstallDir "vx.exe"
    Update-Path -InstallPath $InstallDir
    Test-Installation -BinaryPath $binaryPath
}

# Run main function
Main
