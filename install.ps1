# vx installer script for Windows with multi-channel distribution support
#
# Basic usage:
#   powershell -c "irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex"
#
# With specific version:
#   $env:VX_VERSION="0.1.0"; powershell -c "irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex"
#
# With GitHub token (to avoid rate limits):
#   $env:GITHUB_TOKEN="your_token"; powershell -c "irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex"
#
# Build from source:
#   powershell -c "irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex" -BuildFromSource
#
# Alternative package managers:
#   winget install loonghao.vx
#   scoop install vx

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
# Enable progress bars for better user experience
$ProgressPreference = "Continue"

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

function Write-ProgressInfo {
    param(
        [string]$Activity,
        [string]$Status,
        [int]$PercentComplete = -1
    )
    if ($PercentComplete -ge 0) {
        Microsoft.PowerShell.Utility\Write-Progress -Activity $Activity -Status $Status -PercentComplete $PercentComplete
    }
    else {
        Microsoft.PowerShell.Utility\Write-Progress -Activity $Activity -Status $Status
    }
}

# Detect platform and map to release naming convention
function Get-Platform {
    # Detect architecture more accurately
    $arch = switch ([System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture) {
        "X64" { "x86_64" }
        "Arm64" { "aarch64" }
        "X86" { "x86" }
        default {
            # Fallback to environment check
            if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { "x86" }
        }
    }
    return "Windows-msvc-$arch"
}

# Get latest version from GitHub API with optional authentication and fallback
function Get-LatestVersion {
    try {
        Write-ProgressInfo -Activity "Fetching latest version" -Status "Connecting to GitHub API..."
        $apiUrl = "https://api.github.com/repos/$RepoOwner/$RepoName/releases/latest"

        # Prepare headers for authentication if token is available
        $headers = @{}
        $githubToken = $env:GITHUB_TOKEN
        if ($githubToken) {
            $headers["Authorization"] = "Bearer $githubToken"
            Write-Info "Using authenticated GitHub API request"
        }
        else {
            Write-Info "Using unauthenticated GitHub API request (rate limited)"
        }

        # Make API request with optional authentication
        $response = Invoke-RestMethod -Uri $apiUrl -Method Get -Headers $headers -TimeoutSec 10
        Microsoft.PowerShell.Utility\Write-Progress -Activity "Fetching latest version" -Completed
        return $response.tag_name -replace '^v', ''
    }
    catch {
        Microsoft.PowerShell.Utility\Write-Progress -Activity "Fetching latest version" -Completed

        # Check if this is a rate limit error
        $isRateLimit = $_.Exception.Message -like "*rate limit*" -or
        $_.Exception.Message -like "*429*" -or
        $_.Exception.Message -like "*API rate limit exceeded*"

        if ($isRateLimit) {
            Write-Warn "GitHub API rate limit exceeded. Trying alternative methods..."

            # Try jsDelivr API as fallback
            try {
                Write-Info "Attempting to get version from jsDelivr API..."
                $jsdelivrUrl = "https://data.jsdelivr.com/v1/package/gh/$RepoOwner/$RepoName"
                $jsdelivrResponse = Invoke-RestMethod -Uri $jsdelivrUrl -Method Get -TimeoutSec 10
                if ($jsdelivrResponse.versions -and $jsdelivrResponse.versions.Count -gt 0) {
                    $latestVersion = $jsdelivrResponse.versions[0] -replace '^v', ''
                    Write-Success "Got version from jsDelivr: $latestVersion"
                    return $latestVersion
                }
            }
            catch {
                Write-Warn "jsDelivr API also failed: $_"
            }

            # Provide helpful error message with solutions
            Write-Error "Unable to determine latest version automatically due to rate limiting."
            Write-Host ""
            Write-Host "🔧 Solutions:" -ForegroundColor Yellow
            Write-Host "1. Set GITHUB_TOKEN environment variable:" -ForegroundColor Gray
            Write-Host "   `$env:GITHUB_TOKEN='your_token_here'; .\install.ps1" -ForegroundColor Gray
            Write-Host ""
            Write-Host "2. Specify version explicitly:" -ForegroundColor Gray
            Write-Host "   `$env:VX_VERSION='0.1.0'; .\install.ps1" -ForegroundColor Gray
            Write-Host ""
            Write-Host "3. Use package managers:" -ForegroundColor Gray
            Write-Host "   winget install loonghao.vx" -ForegroundColor Gray
            Write-Host "   scoop install vx" -ForegroundColor Gray
            Write-Host ""
            Write-Host "4. Build from source:" -ForegroundColor Gray
            Write-Host "   .\install.ps1 -BuildFromSource" -ForegroundColor Gray
            Write-Host ""
            exit 1
        }

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
    Microsoft.PowerShell.Utility\Write-Progress -Activity "Building vx from source" -Status "Compiling Rust code..."
    cargo build --release

    if ($LASTEXITCODE -ne 0) {
        Microsoft.PowerShell.Utility\Write-Progress -Activity "Building vx from source" -Completed
        Write-Error "Build failed!"
        exit 1
    }
    Microsoft.PowerShell.Utility\Write-Progress -Activity "Building vx from source" -Status "Build completed" -PercentComplete 100

    # Create installation directory
    Microsoft.PowerShell.Utility\Write-Progress -Activity "Building vx from source" -Status "Creating installation directory..."
    if (!(Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    }

    # Copy the binary
    Microsoft.PowerShell.Utility\Write-Progress -Activity "Building vx from source" -Status "Installing binary..."
    Copy-Item "target\release\vx.exe" "$InstallDir\vx.exe" -Force
    Microsoft.PowerShell.Utility\Write-Progress -Activity "Building vx from source" -Completed
    Write-Success "vx built and installed from source to: $InstallDir"
}

# Download from multiple channels with fallback
function Download-WithFallback {
    param(
        [string]$Version,
        [string]$Platform,
        [string]$ArchiveName,
        [string]$TempDir
    )

    # Define download channels in order of preference
    $channels = @(
        @{
            Name = "GitHub Releases"
            Url  = "$BaseUrl/download/v$Version/$ArchiveName"
        },
        @{
            Name = "jsDelivr CDN"
            Url  = "https://cdn.jsdelivr.net/gh/$RepoOwner/$RepoName@v$Version/$ArchiveName"
        },
        @{
            Name = "Fastly CDN"
            Url  = "https://fastly.jsdelivr.net/gh/$RepoOwner/$RepoName@v$Version/$ArchiveName"
        }
    )

    $archivePath = Join-Path $TempDir $ArchiveName

    foreach ($channel in $channels) {
        try {
            Write-Info "Trying $($channel.Name): $($channel.Url)"
            Microsoft.PowerShell.Utility\Write-Progress -Activity "Installing vx" -Status "Downloading from $($channel.Name)..." -PercentComplete 30

            Invoke-WebRequest -Uri $channel.Url -OutFile $archivePath -UseBasicParsing -TimeoutSec 30

            # Verify download
            if (Test-Path $archivePath) {
                $fileSize = (Get-Item $archivePath).Length
                if ($fileSize -gt 1024) {
                    # At least 1KB
                    Write-Success "Successfully downloaded from $($channel.Name) ($([math]::Round($fileSize/1MB, 2)) MB)"
                    return $archivePath
                }
                else {
                    Write-Warn "Downloaded file too small, trying next channel..."
                    Remove-Item $archivePath -Force -ErrorAction SilentlyContinue
                }
            }
        }
        catch {
            Write-Warn "Failed to download from $($channel.Name): $_"
            Remove-Item $archivePath -Force -ErrorAction SilentlyContinue
        }
    }

    throw "Failed to download from all channels"
}

# Download and install vx from releases with multiple channel support
function Install-FromRelease {
    $platform = Get-Platform

    if ($Version -eq "latest") {
        Write-Info "Fetching latest version..."
        Microsoft.PowerShell.Utility\Write-Progress -Activity "Installing vx" -Status "Fetching latest version..." -PercentComplete 10
        $Version = Get-LatestVersion
        if (-not $Version) {
            Microsoft.PowerShell.Utility\Write-Progress -Activity "Installing vx" -Completed
            Write-Error "Failed to get latest version"
            exit 1
        }
    }

    Write-Info "Installing vx v$Version for $platform..."

    # Construct archive name based on actual release asset naming
    # Format: vx-{OS}-{variant}-{arch}.zip
    $archiveName = "vx-$platform.zip"

    # Create temporary directory
    Microsoft.PowerShell.Utility\Write-Progress -Activity "Installing vx" -Status "Preparing download..." -PercentComplete 20
    $tempDir = New-TemporaryFile | ForEach-Object { Remove-Item $_; New-Item -ItemType Directory -Path $_ }

    try {
        # Download with fallback channels
        $archivePath = Download-WithFallback -Version $Version -Platform $platform -ArchiveName $archiveName -TempDir $tempDir

        # Extract
        Write-Info "Extracting to $InstallDir..."
        Microsoft.PowerShell.Utility\Write-Progress -Activity "Installing vx" -Status "Extracting archive..." -PercentComplete 60
        if (-not (Test-Path $InstallDir)) {
            New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        }

        Expand-Archive -Path $archivePath -DestinationPath $tempDir -Force

        # Find and copy the binary
        Microsoft.PowerShell.Utility\Write-Progress -Activity "Installing vx" -Status "Installing binary..." -PercentComplete 80
        $binaryPath = Get-ChildItem -Path $tempDir -Name "vx.exe" -Recurse | Select-Object -First 1
        if (-not $binaryPath) {
            Microsoft.PowerShell.Utility\Write-Progress -Activity "Installing vx" -Completed
            Write-Error "vx.exe not found in archive"
            exit 1
        }

        $sourcePath = Join-Path $tempDir $binaryPath
        $destPath = Join-Path $InstallDir "vx.exe"
        Copy-Item -Path $sourcePath -Destination $destPath -Force

        Microsoft.PowerShell.Utility\Write-Progress -Activity "Installing vx" -Status "Installation completed" -PercentComplete 100
        Write-Success "vx v$Version installed to $destPath"
    }
    catch {
        Microsoft.PowerShell.Utility\Write-Progress -Activity "Installing vx" -Completed
        Write-Warn "Failed to download pre-built binary: $_"
        Write-Info "Falling back to building from source..."
        Build-FromSource
        return
    }
    finally {
        # Cleanup
        Microsoft.PowerShell.Utility\Write-Progress -Activity "Installing vx" -Completed
        Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

# Update PATH environment variable
function Update-Path {
    param([string]$InstallPath)

    Microsoft.PowerShell.Utility\Write-Progress -Activity "Finalizing installation" -Status "Checking PATH environment..." -PercentComplete 90

    # Check if install directory is in PATH
    $currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    if ($currentPath -notlike "*$InstallPath*") {
        Write-Warn "Add $InstallPath to your PATH to use vx from anywhere:"
        Write-Host "  Run this command in an elevated PowerShell:"
        Write-Host "  [Environment]::SetEnvironmentVariable('PATH', `$env:PATH + ';$InstallPath', 'User')"
        Write-Host ""
        Write-Host "Or add it manually through System Properties > Environment Variables"
    }

    Microsoft.PowerShell.Utility\Write-Progress -Activity "Finalizing installation" -Completed
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
