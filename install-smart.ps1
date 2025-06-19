# Smart vx installer for Windows with intelligent channel selection and fallback
# This installer automatically detects the best distribution channel based on
# geographic location, network conditions, and availability
#
# Usage: powershell -c "irm https://raw.githubusercontent.com/loonghao/vx/main/install-smart.ps1 | iex"
# Usage with version: $env:VX_VERSION="0.1.0"; powershell -c "irm https://raw.githubusercontent.com/loonghao/vx/main/install-smart.ps1 | iex"
# Usage with token: $env:GITHUB_TOKEN="token"; powershell -c "irm https://raw.githubusercontent.com/loonghao/vx/main/install-smart.ps1 | iex"

param(
    [string]$Version = $env:VX_VERSION,
    [string]$InstallDir = $env:VX_INSTALL_DIR,
    [string]$ForceChannel = $env:VX_FORCE_CHANNEL,
    [switch]$BuildFromSource = [bool]$env:VX_BUILD_FROM_SOURCE,
    [switch]$Debug = [bool]$env:VX_DEBUG
)

# Configuration
$RepoOwner = "loonghao"
$RepoName = "vx"
$DefaultInstallDir = "$env:USERPROFILE\.local\bin"

# Set defaults
if (-not $Version) { $Version = "latest" }
if (-not $InstallDir) { $InstallDir = $DefaultInstallDir }

# Logging functions
function Write-Info {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Blue
}

function Write-Success {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor Green
}

function Write-Warn {
    param([string]$Message)
    Write-Host "[WARN] $Message" -ForegroundColor Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

function Write-Debug {
    param([string]$Message)
    if ($Debug) {
        Write-Host "[DEBUG] $Message" -ForegroundColor Cyan
    }
}

# Detect platform
function Get-Platform {
    $arch = if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { "x86" }
    return "Windows-msvc-$arch"
}

# Detect geographic region for optimal CDN selection
function Get-Region {
    $region = "global"
    
    try {
        # Try to detect region from ipinfo.io
        $response = Invoke-RestMethod -Uri "https://ipinfo.io/country" -TimeoutSec 3 -ErrorAction SilentlyContinue
        
        switch ($response) {
            { $_ -in @("CN", "HK", "TW", "SG", "JP", "KR", "MY", "TH", "VN", "ID", "PH") } { $region = "asia" }
            { $_ -in @("US", "CA", "MX", "BR", "AR", "CL", "PE", "CO", "VE") } { $region = "americas" }
            { $_ -in @("GB", "DE", "FR", "IT", "ES", "NL", "SE", "NO", "DK", "FI", "PL", "RU") } { $region = "europe" }
            { $_ -in @("AU", "NZ") } { $region = "oceania" }
            default { $region = "global" }
        }
    }
    catch {
        Write-Debug "Region detection failed, using global"
    }
    
    Write-Debug "Detected region: $region"
    return $region
}

# Test channel speed and availability
function Test-ChannelSpeed {
    param(
        [string]$Url,
        [int]$TimeoutSec = 5
    )
    
    try {
        $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
        $response = Invoke-WebRequest -Uri $Url -Method Head -TimeoutSec $TimeoutSec -ErrorAction Stop
        $stopwatch.Stop()
        
        if ($response.StatusCode -eq 200) {
            return $stopwatch.ElapsedMilliseconds
        }
    }
    catch {
        Write-Debug "Speed test failed for $Url : $_"
    }
    
    return 999999  # Return high value for failed tests
}

# Get optimal channel order based on region and speed tests
function Get-OptimalChannels {
    param(
        [string]$Region,
        [string]$Version,
        [string]$Platform
    )
    
    # Define all available channels
    $channels = @{
        "github"   = "https://github.com/$RepoOwner/$RepoName/releases/download/v$Version"
        "jsdelivr" = "https://cdn.jsdelivr.net/gh/$RepoOwner/$RepoName@v$Version"
        "fastly"   = "https://fastly.jsdelivr.net/gh/$RepoOwner/$RepoName@v$Version"
    }
    
    # Region-specific channel preferences
    $channelOrder = switch ($Region) {
        "asia" { @("jsdelivr", "fastly", "github") }
        "europe" { @("fastly", "jsdelivr", "github") }
        "americas" { @("github", "fastly", "jsdelivr") }
        default { @("github", "jsdelivr", "fastly") }
    }
    
    # If user forced a specific channel, use it first
    if ($ForceChannel) {
        Write-Debug "Using forced channel: $ForceChannel"
        $channelOrder = @($ForceChannel) + ($channelOrder | Where-Object { $_ -ne $ForceChannel })
    }
    
    # Test channel speeds (optional, can be disabled for faster installs)
    if ($env:VX_SPEED_TEST -ne "false") {
        Write-Info "Testing channel speeds..."
        $speeds = @{}
        
        foreach ($channel in $channelOrder) {
            $testUrl = $channels[$channel]
            $speed = Test-ChannelSpeed -Url $testUrl -TimeoutSec 3
            $speeds[$channel] = $speed
            Write-Debug "Channel $channel speed: ${speed}ms"
        }
        
        # Sort channels by speed
        $channelOrder = $speeds.GetEnumerator() | Sort-Object Value | ForEach-Object { $_.Key }
    }
    
    return $channelOrder
}

# Get latest version with intelligent fallback
function Get-LatestVersion {
    param([string]$Region)
    
    # If no token is provided, prefer CDN to avoid rate limits
    if (-not $env:GITHUB_TOKEN) {
        Write-Info "🌐 No GitHub token provided, using CDN for version check..."
        
        # Try jsDelivr API first when no token
        try {
            Write-Info "Attempting to get version from jsDelivr API..."
            $jsdelivrUrl = "https://data.jsdelivr.com/v1/package/gh/$RepoOwner/$RepoName"
            $jsdelivrResponse = Invoke-RestMethod -Uri $jsdelivrUrl -TimeoutSec 10
            
            if ($jsdelivrResponse.versions -and $jsdelivrResponse.versions.Count -gt 0) {
                $latestVersion = $jsdelivrResponse.versions[0] -replace '^v', ''
                Write-Success "Got version from jsDelivr: $latestVersion"
                return $latestVersion
            }
        }
        catch {
            Write-Warn "jsDelivr API failed: $_"
            Write-Info "🔄 Falling back to GitHub API..."
        }
    }
    
    # Try GitHub API
    try {
        $apiUrl = "https://api.github.com/repos/$RepoOwner/$RepoName/releases/latest"
        $headers = @{}
        
        if ($env:GITHUB_TOKEN) {
            $headers["Authorization"] = "Bearer $env:GITHUB_TOKEN"
            Write-Info "Using authenticated GitHub API request"
        }
        
        $response = Invoke-RestMethod -Uri $apiUrl -Headers $headers -TimeoutSec 10
        $version = $response.tag_name -replace '^v', ''
        Write-Debug "Got version from GitHub API: $version"
        return $version
    }
    catch {
        if ($_.Exception.Message -like "*rate limit*" -or $_.Exception.Message -like "*429*") {
            Write-Error "GitHub API rate limit exceeded and CDN fallback failed."
            Write-Host ""
            Write-Host "🔧 Solutions:" -ForegroundColor Yellow
            Write-Host "1. Set GITHUB_TOKEN: `$env:GITHUB_TOKEN='token'; .\install-smart.ps1" -ForegroundColor Gray
            Write-Host "2. Specify version: `$env:VX_VERSION='0.1.0'; .\install-smart.ps1" -ForegroundColor Gray
            Write-Host "3. Use package managers: winget install loonghao.vx" -ForegroundColor Gray
            Write-Host "4. Build from source: .\install-smart.ps1 -BuildFromSource" -ForegroundColor Gray
            exit 1
        }
        
        throw "Failed to get latest version: $_"
    }
}

# Download with intelligent channel selection
function Invoke-SmartDownload {
    param(
        [string]$Version,
        [string]$Platform,
        [string]$ArchiveName,
        [string]$TempDir,
        [string]$Region
    )
    
    $archivePath = Join-Path $TempDir $ArchiveName
    $channels = Get-OptimalChannels -Region $Region -Version $Version -Platform $Platform
    
    Write-Info "Trying channels in optimal order for region: $Region"
    
    foreach ($channel in $channels) {
        $downloadUrl = switch ($channel) {
            "github" { "https://github.com/$RepoOwner/$RepoName/releases/download/v$Version/$ArchiveName" }
            "jsdelivr" { "https://cdn.jsdelivr.net/gh/$RepoOwner/$RepoName@v$Version/$ArchiveName" }
            "fastly" { "https://fastly.jsdelivr.net/gh/$RepoOwner/$RepoName@v$Version/$ArchiveName" }
            default { 
                Write-Warn "Unknown channel: $channel"
                continue
            }
        }
        
        Write-Info "Trying $channel : $downloadUrl"
        
        try {
            Invoke-WebRequest -Uri $downloadUrl -OutFile $archivePath -TimeoutSec 30
            
            # Verify download
            if (Test-Path $archivePath) {
                $fileSize = (Get-Item $archivePath).Length
                if ($fileSize -gt 1024) {
                    # At least 1KB
                    $fileSizeMB = [math]::Round($fileSize / 1MB, 2)
                    Write-Success "Downloaded from $channel ($fileSizeMB MB)"
                    return $archivePath
                }
                else {
                    Write-Warn "Downloaded file too small, trying next channel..."
                    Remove-Item $archivePath -Force -ErrorAction SilentlyContinue
                }
            }
        }
        catch {
            Write-Warn "Failed to download from $channel : $_"
            Remove-Item $archivePath -Force -ErrorAction SilentlyContinue
        }
    }
    
    throw "Failed to download from all channels"
}

# Install from release with smart channel selection
function Install-FromRelease {
    $platform = Get-Platform
    $region = Get-Region
    
    if ($Version -eq "latest") {
        Write-Info "Fetching latest version..."
        $Version = Get-LatestVersion -Region $region
        if (-not $Version) {
            Write-Error "Failed to get latest version"
            exit 1
        }
    }

    Write-Info "Installing vx v$Version for $platform (region: $region)"
    
    # Determine archive name based on platform
    # Try multiple naming conventions for Windows
    $possibleArchives = @(
        "vx-$platform.zip",
        "vx-Windows-x86_64.zip",
        "vx-windows-x86_64.zip",
        "vx-$platform.tar.gz",
        "vx-Windows-x86_64.tar.gz"
    )

    $archiveName = $null
    $downloadSuccess = $false
    
    # Create temporary directory
    $tempDir = New-TemporaryFile | ForEach-Object { Remove-Item $_; New-Item -ItemType Directory -Path $_ }
    
    try {
        # Try different archive names until one works
        foreach ($tryArchive in $possibleArchives) {
            Write-Debug "Trying archive name: $tryArchive"
            try {
                $archivePath = Invoke-SmartDownload -Version $Version -Platform $platform -ArchiveName $tryArchive -TempDir $tempDir -Region $region
                $archiveName = $tryArchive
                $downloadSuccess = $true
                break
            }
            catch {
                Write-Debug "Failed with archive name $tryArchive : $_"
                continue
            }
        }

        if (-not $downloadSuccess) {
            Write-Error "Failed to download vx binary with any supported archive format"
            Write-Host ""
            Write-Host "🔧 Possible solutions:" -ForegroundColor Yellow
            Write-Host "1. Check if Windows binaries are available for version $Version" -ForegroundColor Gray
            Write-Host "2. Try a different version: `$env:VX_VERSION='0.2.6'; .\install-smart.ps1" -ForegroundColor Gray
            Write-Host "3. Use package managers: winget install loonghao.vx" -ForegroundColor Gray
            Write-Host "4. Build from source (not implemented for Windows yet)" -ForegroundColor Gray
            exit 1
        }
        
        # Extract
        Write-Info "Extracting to $InstallDir..."
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null

        $binaryPath = $null

        if ($archiveName.EndsWith(".zip")) {
            # Extract ZIP
            Add-Type -AssemblyName System.IO.Compression.FileSystem
            $zip = [System.IO.Compression.ZipFile]::OpenRead($archivePath)

            foreach ($entry in $zip.Entries) {
                if ($entry.Name -eq "vx.exe" -or $entry.Name -eq "vx") {
                    $binaryPath = Join-Path $InstallDir "vx.exe"
                    [System.IO.Compression.ZipFileExtensions]::ExtractToFile($entry, $binaryPath, $true)
                    break
                }
            }
            $zip.Dispose()
        }
        elseif ($archiveName.EndsWith(".tar.gz")) {
            # Extract TAR.GZ (requires external tool or manual extraction)
            Write-Info "Extracting tar.gz archive..."

            # Try using tar command if available (Windows 10 1803+ has built-in tar)
            if (Get-Command tar -ErrorAction SilentlyContinue) {
                $extractDir = Join-Path $tempDir "extracted"
                New-Item -ItemType Directory -Path $extractDir -Force | Out-Null

                & tar -xzf $archivePath -C $extractDir

                # Find the binary
                $extractedBinary = Get-ChildItem -Path $extractDir -Recurse -Name "vx" -File | Select-Object -First 1
                if ($extractedBinary) {
                    $binaryPath = Join-Path $InstallDir "vx.exe"
                    Copy-Item (Join-Path $extractDir $extractedBinary) $binaryPath
                }
            }
            else {
                Write-Error "tar.gz extraction requires tar command (available in Windows 10 1803+)"
                Write-Info "Please use a ZIP version or install tar command"
                exit 1
            }
        }
        
        if (-not $binaryPath -or -not (Test-Path $binaryPath)) {
            Write-Error "vx binary not found in archive"
            exit 1
        }
        
        Write-Success "vx v$Version installed to $binaryPath"
    }
    finally {
        # Cleanup
        Remove-Item $tempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

# Update PATH
function Update-Path {
    param([string]$InstallPath)
    
    # Check if directory is already in PATH
    $currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    if ($currentPath -like "*$InstallPath*") {
        Write-Info "Install directory already in PATH"
        return
    }
    
    # Add to user PATH
    $newPath = "$InstallPath;$currentPath"
    [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
    
    # Update current session PATH
    $env:PATH = "$InstallPath;$env:PATH"
    
    Write-Info "Added $InstallPath to PATH"
    Write-Info "Restart your terminal or run 'refreshenv' to use vx"
}

# Test installation
function Test-Installation {
    param([string]$BinaryPath)
    
    if (Test-Path $BinaryPath) {
        try {
            $versionOutput = & $BinaryPath --version 2>$null
            if ($versionOutput) {
                Write-Success "Installation verified: $versionOutput"
            }
            else {
                Write-Warn "Binary installed but version check failed"
            }
        }
        catch {
            Write-Warn "Binary installed but version check failed: $_"
        }
    }
    else {
        Write-Error "Installation failed: binary not found"
        exit 1
    }
}

# Main execution
function Main {
    Write-Info "vx smart installer for Windows"
    Write-Host ""
    
    # Show configuration
    Write-Debug "Configuration:"
    Write-Debug "  Version: $Version"
    Write-Debug "  Install Dir: $InstallDir"
    Write-Debug "  Build from Source: $BuildFromSource"
    Write-Debug "  Force Channel: $ForceChannel"
    Write-Debug "  Speed Test: $($env:VX_SPEED_TEST -ne 'false')"
    
    # Check if we should build from source
    if ($BuildFromSource) {
        Write-Error "Build from source not implemented for Windows. Please use the binary installation."
        exit 1
    }
    else {
        Install-FromRelease
    }
    
    # Update PATH and test
    Update-Path -InstallPath $InstallDir
    Test-Installation -BinaryPath (Join-Path $InstallDir "vx.exe")
    
    Write-Host ""
    Write-Success "vx installation completed!"
    Write-Info "Run 'vx --help' to get started"
    
    # Show some helpful commands
    Write-Host ""
    Write-Host "📖 Quick start:" -ForegroundColor Yellow
    Write-Host "   vx --help          # Show help" -ForegroundColor Gray
    Write-Host "   vx list            # List available tools" -ForegroundColor Gray
    Write-Host "   vx npm --version   # Use npm through vx" -ForegroundColor Gray
    Write-Host "   vx uv --version    # Use uv through vx" -ForegroundColor Gray
}

# Run main function
Main
