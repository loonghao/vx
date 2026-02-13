# vx installer script for Windows
#
# Basic usage:
#   powershell -c "irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex"
#
# With specific version (use tag format like "v0.6.0" or just "0.6.0"):
#   $env:VX_VERSION="0.5.7"; powershell -c "irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex"
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

# Check if Windows long path support is enabled
function Test-LongPathEnabled {
    try {
        $key = Get-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\FileSystem" -Name "LongPathsEnabled" -ErrorAction SilentlyContinue
        return $key.LongPathsEnabled -eq 1
    }
    catch {
        return $false
    }
}

# Show instructions for enabling long path support
function Show-LongPathInstructions {
    Write-Host ""
    Write-Host "⚠️  Windows Long Path Support is NOT enabled" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "vx may encounter issues with deep directory paths (>260 characters)," -ForegroundColor Gray
    Write-Host "especially when installing npm packages with nested dependencies." -ForegroundColor Gray
    Write-Host ""
    Write-Host "To enable long path support (recommended):" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Option 1: Run this PowerShell command (requires Administrator):" -ForegroundColor White
    Write-Host '  New-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\FileSystem" `' -ForegroundColor Gray
    Write-Host '      -Name "LongPathsEnabled" -Value 1 -PropertyType DWORD -Force' -ForegroundColor Gray
    Write-Host ""
    Write-Host "Option 2: Via Group Policy (Windows 10 Pro/Enterprise):" -ForegroundColor White
    Write-Host "  1. Open gpedit.msc" -ForegroundColor Gray
    Write-Host "  2. Navigate to: Computer Configuration > Administrative Templates > System > Filesystem" -ForegroundColor Gray
    Write-Host "  3. Enable 'Enable Win32 long paths'" -ForegroundColor Gray
    Write-Host ""
    Write-Host "Option 3: Use a shorter VX_HOME path:" -ForegroundColor White
    Write-Host '  $env:VX_HOME = "C:\vx"' -ForegroundColor Gray
    Write-Host ""
    Write-Host "After enabling, restart your terminal or reboot Windows." -ForegroundColor Yellow
    Write-Host ""
}

# Detect platform and map to release naming convention
function Get-Platform {
    # Detect architecture more accurately
    $arch = switch ([System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture) {
        "X64" { "x86_64" }
        "Arm64" { "aarch64" }
        "X86" { "i686" }
        default {
            # Fallback to environment check
            if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { "i686" }
        }
    }
    # Match Rust target triple format: {arch}-pc-windows-msvc
    return "$arch-pc-windows-msvc"
}

# Get latest version from GitHub API with optional authentication and fallback
# Returns the full tag name (e.g., "v0.5.7") of a release that has assets
function Get-LatestVersion {
    try {
        Write-ProgressInfo -Activity "Fetching latest version" -Status "Connecting to GitHub API..."
        $apiUrl = "https://api.github.com/repos/$RepoOwner/$RepoName/releases?per_page=30"

        # Prepare headers for authentication if token is available
        $headers = @{
            "Accept" = "application/vnd.github.v3+json"
        }
        $githubToken = $env:GITHUB_TOKEN
        if ($githubToken) {
            $headers["Authorization"] = "Bearer $githubToken"
            Write-Info "Using authenticated GitHub API request"
        }
        else {
            Write-Info "Using unauthenticated GitHub API request (rate limited)"
        }

        # Make API request with optional authentication
        $response = Invoke-RestMethod -Uri $apiUrl -Method Get -Headers $headers -TimeoutSec 30
        Microsoft.PowerShell.Utility\Write-Progress -Activity "Fetching latest version" -Completed
        
        # Find first non-prerelease release with assets
        foreach ($release in $response) {
            if (-not $release.prerelease -and $release.assets.Count -gt 0) {
                Write-Info "Found version with assets: $($release.tag_name)"
                return $release.tag_name
            }
        }
        
        # If no release with assets found, return the first non-prerelease
        $firstRelease = $response | Where-Object { -not $_.prerelease } | Select-Object -First 1
        if ($firstRelease) {
            return $firstRelease.tag_name
        }
        
        throw "No releases found"
    }
    catch {
        Microsoft.PowerShell.Utility\Write-Progress -Activity "Fetching latest version" -Completed

        # Check if this is a rate limit error
        $errorMessage = $_.Exception.Message
        $isRateLimit = $errorMessage -like "*rate limit*" -or
        $errorMessage -like "*429*" -or
        $errorMessage -like "*API rate limit exceeded*"

        if ($isRateLimit) {
            Write-Warn "GitHub API rate limit exceeded. Trying fallback method..."

            # Fallback: Try to find version with assets from releases page
            try {
                $foundVersion = Find-VersionWithAssetsFromPage
                if ($foundVersion) {
                    Write-Success "Found version with assets: $foundVersion"
                    return $foundVersion
                }
            }
            catch {
                Write-Warn "Fallback method failed: $($_.Exception.Message)"
            }

            # If fallback also fails, provide helpful error message
            Write-Error "Unable to determine latest version automatically due to rate limiting."
            Write-Host ""
            Write-Host "Solutions:" -ForegroundColor Yellow
            Write-Host "1. Set GITHUB_TOKEN environment variable:" -ForegroundColor Gray
            Write-Host "   `$env:GITHUB_TOKEN='your_token_here'; irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex" -ForegroundColor Gray
            Write-Host ""
            Write-Host "2. Specify version explicitly:" -ForegroundColor Gray
            Write-Host "   `$env:VX_VERSION='0.6.7'; irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex" -ForegroundColor Gray
            Write-Host ""
            Write-Host "3. Use package managers:" -ForegroundColor Gray
            Write-Host "   winget install loonghao.vx" -ForegroundColor Gray
            Write-Host "   scoop install vx" -ForegroundColor Gray
            Write-Host ""
            Write-Host "4. Download directly from:" -ForegroundColor Gray
            Write-Host "   https://github.com/loonghao/vx/releases/latest" -ForegroundColor Gray
            Write-Host ""
            exit 1
        }

        Write-Error "Failed to get latest version: $_"
        exit 1
    }
}

# Find a version with assets from GitHub releases page (fallback when API is rate limited)
# Parses the releases page HTML to find versions that have downloadable assets
function Find-VersionWithAssetsFromPage {
    Write-Info "Fetching releases page to find version with assets..."
    
    try {
        $releasesUrl = "https://github.com/$RepoOwner/$RepoName/releases"
        $response = Invoke-WebRequest -Uri $releasesUrl -UseBasicParsing -TimeoutSec 30
        $html = $response.Content
        
        # Extract version tags from release links
        # Pattern: href="/loonghao/vx/releases/tag/TAG_NAME"
        $tagPattern = 'href="/[^"]+/releases/tag/([^"]+)"'
        $matches = [regex]::Matches($html, $tagPattern)
        
        $seenTags = @{}
        foreach ($match in $matches) {
            $tag = $match.Groups[1].Value
            
            # Skip if already seen
            if ($seenTags.ContainsKey($tag)) { continue }
            $seenTags[$tag] = $true
            
            # Skip pre-release tags
            if ($tag -match '-(alpha|beta|rc|pre|dev)') { continue }
            
            # Check if this release has assets
            $releaseUrl = "https://github.com/$RepoOwner/$RepoName/releases/tag/$tag"
            try {
                $releaseResponse = Invoke-WebRequest -Uri $releaseUrl -UseBasicParsing -TimeoutSec 10
                $releaseHtml = $releaseResponse.Content
                
                # Check for .tar.gz or .zip in the release page
                if ($releaseHtml -match '\.(tar\.gz|zip)') {
                    Write-Info "Found version with assets: $tag"
                    return $tag
                }
            }
            catch {
                # Continue to next tag if this release page fails
                continue
            }
        }
        
        # Fallback: return the first tag found
        if ($seenTags.Keys.Count -gt 0) {
            $firstTag = $seenTags.Keys | Select-Object -First 1
            return $firstTag
        }
        
        return $null
    }
    catch {
        Write-Warn "Failed to fetch releases page: $($_.Exception.Message)"
        return $null
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

# Download from multiple channels with fallback and retry
# Note: jsDelivr CDN doesn't support GitHub Release assets, only use GitHub Releases
function Download-WithFallback {
    param(
        [string]$TagName,
        [string]$Platform,
        [string]$ArchiveName,
        [string]$TempDir
    )

    # Only use GitHub Releases (jsDelivr doesn't support release assets)
    $channels = @(
        @{
            Name = "GitHub Releases"
            Url  = "$BaseUrl/download/$TagName/$ArchiveName"
        }
    )

    $archivePath = Join-Path $TempDir $ArchiveName
    $maxRetries = 3
    $retryDelay = 2

    foreach ($channel in $channels) {
        for ($retry = 1; $retry -le $maxRetries; $retry++) {
            try {
                if ($retry -gt 1) {
                    Write-Info "Retry attempt $retry of $maxRetries..."
                    Start-Sleep -Seconds $retryDelay
                }

                Write-Info "Trying $($channel.Name): $($channel.Url)"
                Microsoft.PowerShell.Utility\Write-Progress -Activity "Installing vx" -Status "Downloading from $($channel.Name) (attempt $retry)..." -PercentComplete 30

                # Use more robust download with timeout and retry
                $webClient = New-Object System.Net.WebClient
                $webClient.Headers.Add("User-Agent", "vx-installer/1.0")

                # Add GitHub token if available
                if ($env:GITHUB_TOKEN) {
                    $webClient.Headers.Add("Authorization", "Bearer $env:GITHUB_TOKEN")
                }

                $webClient.DownloadFile($channel.Url, $archivePath)

                # Verify download
                if (Test-Path $archivePath) {
                    $fileSize = (Get-Item $archivePath).Length
                    if ($fileSize -gt 1024) {
                        # At least 1KB
                        Write-Success "Successfully downloaded from $($channel.Name) ($([math]::Round($fileSize/1MB, 2)) MB)"
                        return $archivePath
                    }
                    else {
                        Write-Warn "Downloaded file too small, retrying..."
                        Remove-Item $archivePath -Force -ErrorAction SilentlyContinue
                    }
                }
            }
            catch {
                $errorMessage = $_.Exception.Message
                Write-Warn "Attempt $retry failed: $errorMessage"
                Remove-Item $archivePath -Force -ErrorAction SilentlyContinue

                # Check for specific network errors that warrant retry
                $shouldRetry = $errorMessage -like "*timeout*" -or
                               $errorMessage -like "*connection*" -or
                               $errorMessage -like "*network*" -or
                               $errorMessage -like "*503*" -or
                               $errorMessage -like "*502*" -or
                               $errorMessage -like "*504*"

                if (-not $shouldRetry -and $retry -lt $maxRetries) {
                    Write-Warn "Non-retryable error, trying next channel..."
                    break
                }
            }
        }
    }

    throw "Failed to download from all channels after $maxRetries retries"
}

# Download and install vx from releases with multiple channel support
function Install-FromRelease {
    $platform = Get-Platform

    if ($Version -eq "latest") {
        Write-Info "Fetching latest version..."
        Microsoft.PowerShell.Utility\Write-Progress -Activity "Installing vx" -Status "Fetching latest version..." -PercentComplete 10
        $tagName = Get-LatestVersion
        if (-not $tagName) {
            Microsoft.PowerShell.Utility\Write-Progress -Activity "Installing vx" -Completed
            Write-Error "Failed to get latest version"
            exit 1
        }
    }
    else {
        # User specified version - normalize to tag format
        # Accept: "v0.6.7", "0.6.7", "vx-v0.6.7"
        if ($Version -match '^vx-v') {
            $tagName = $Version
        }
        elseif ($Version -match '^v') {
            $tagName = $Version
        }
        else {
            $tagName = "v$Version"
        }
    }

    Write-Info "Installing vx $tagName for $platform..."

    # Extract version number from tag (e.g., "v0.5.7" -> "0.5.7", "vx-v0.6.27" -> "0.6.27")
    $versionNumber = $tagName -replace '^(vx-)?v', ''

    # Determine all possible tag formats for this version
    # v0.7.0+ uses v{ver} (cargo-dist), v0.6.x and earlier use vx-v{ver}
    $vParts = $versionNumber.Split('.')
    $major = [int]$vParts[0]
    $minor = if ($vParts.Length -gt 1) { [int]$vParts[1] } else { 0 }
    if ($major -gt 0 -or ($major -eq 0 -and $minor -ge 7)) {
        # v0.7.0+: try v{ver} first, then vx-v{ver} as fallback
        $tagCandidates = @("v$versionNumber", "vx-v$versionNumber")
    }
    else {
        # v0.6.x and earlier: try vx-v{ver} first, then v{ver} as fallback
        $tagCandidates = @("vx-v$versionNumber", "v$versionNumber")
    }
    $tagName = $tagCandidates[0]
    Write-Info "Using tag format: $tagName (fallback: $($tagCandidates[1]))"

    # Construct archive names - try both versioned and unversioned naming
    $archiveName = "vx-$versionNumber-$platform.zip"
    $unversionedArchiveName = "vx-$platform.zip"

    # Create temporary directory
    Microsoft.PowerShell.Utility\Write-Progress -Activity "Installing vx" -Status "Preparing download..." -PercentComplete 20
    $tempDir = New-TemporaryFile | ForEach-Object { Remove-Item $_; New-Item -ItemType Directory -Path $_ }

    try {
        # Build list of (tag, archive) combinations to try
        $tryCombos = @()
        foreach ($tryTag in $tagCandidates) {
            $tryCombos += @{ Tag = $tryTag; Archive = $archiveName }
            if ($unversionedArchiveName -ne $archiveName) {
                $tryCombos += @{ Tag = $tryTag; Archive = $unversionedArchiveName }
            }
        }

        $downloadSuccess = $false
        $archivePath = $null
        foreach ($combo in $tryCombos) {
            if ($downloadSuccess) { break }
            try {
                $archivePath = Download-WithFallback -TagName $combo.Tag -Platform $platform -ArchiveName $combo.Archive -TempDir $tempDir
                $archiveName = $combo.Archive
                $tagName = $combo.Tag
                $downloadSuccess = $true
            }
            catch {
                Write-Warn "Failed with tag=$($combo.Tag) archive=$($combo.Archive): $_"
            }
        }

        if (-not $downloadSuccess) {
            Microsoft.PowerShell.Utility\Write-Progress -Activity "Installing vx" -Completed
            Write-Warn "Failed to download pre-built binary"
            Write-Info "Falling back to building from source..."
            Build-FromSource
            return
        }
    }
    catch {
        Microsoft.PowerShell.Utility\Write-Progress -Activity "Installing vx" -Completed
        Write-Warn "Failed to download pre-built binary: $_"
        Write-Info "Falling back to building from source..."
        Build-FromSource
        return
    }

    try {
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
        Write-Success "vx $tagName installed to $destPath"
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
        # Automatically add to user PATH
        try {
            $newPath = "$InstallPath;$currentPath"
            [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
            Write-Success "Added $InstallPath to user PATH"

            # Update current session PATH
            $env:PATH = "$InstallPath;$env:PATH"
            Write-Info "Updated current session PATH"
        }
        catch {
            Write-Warn "Could not automatically update PATH: $_"
            Write-Host ""
            Write-Host "Please add $InstallPath to your PATH manually:" -ForegroundColor Yellow
            Write-Host "  Run this command in an elevated PowerShell:" -ForegroundColor Gray
            Write-Host "  [Environment]::SetEnvironmentVariable('PATH', `$env:PATH + ';$InstallPath', 'User')" -ForegroundColor Gray
            Write-Host ""
            Write-Host "Or add it manually through System Properties > Environment Variables" -ForegroundColor Gray
        }
    }
    else {
        Write-Info "Install directory already in PATH"
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
        Write-Host "   vx uv self version" -ForegroundColor Gray
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

    # Check Windows long path support
    if (-not (Test-LongPathEnabled)) {
        Show-LongPathInstructions
        Write-Info "Continuing with installation... (vx has built-in long path workarounds)"
        Write-Host ""
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
