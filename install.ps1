# vx installer script for Windows
#
# Usage:
#   irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex
#
# With specific version:
#   $env:VX_VERSION="0.8.4"; irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex
#
# With custom install directory:
#   $env:VX_INSTALL_DIR="C:\tools\bin"; irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex
#
# Alternative package managers:
#   winget install loonghao.vx
#   scoop install vx

param(
    [string]$Version         = $env:VX_VERSION,
    [string]$InstallDir      = $env:VX_INSTALL_DIR,
    [string]$ReleaseBaseUrls = $env:VX_RELEASE_BASE_URLS
)

$ErrorActionPreference = "Stop"

$RepoOwner = "loonghao"
$RepoName  = "vx"
$BaseUrl   = "https://github.com/$RepoOwner/$RepoName/releases"

function Get-ReleaseBaseUrls {
    if (-not $ReleaseBaseUrls) {
        return @($BaseUrl)
    }

    $urls = $ReleaseBaseUrls -split '[,;]' |
        ForEach-Object { $_.Trim() } |
        Where-Object { $_ -ne "" }

    if (-not $urls -or $urls.Count -eq 0) {
        return @($BaseUrl)
    }

    return $urls
}


if (-not $InstallDir) {
    $InstallDir = "$env:USERPROFILE\.local\bin"
}

# ── Logging ──────────────────────────────────────────────────────────────────

function Write-Step  { param([string]$m) Write-Host "  $RepoName " -NoNewline -ForegroundColor Cyan; Write-Host $m }
function Write-Ok    { param([string]$m) Write-Host "  $RepoName " -NoNewline -ForegroundColor Green; Write-Host $m }
function Write-Fail  { param([string]$m) Write-Host "  $RepoName " -NoNewline -ForegroundColor Red; Write-Host $m }

# ── Platform detection ────────────────────────────────────────────────────────

function Get-Platform {
    $arch = switch ([System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture) {
        "X64"   { "x86_64" }
        "Arm64" { "aarch64" }
        default { if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { "i686" } }
    }
    return "$arch-pc-windows-msvc"
}

# ── Download with retry ───────────────────────────────────────────────────────

function Invoke-Download {
    param([string]$Url, [string]$Dest)

    $headers = @{ "User-Agent" = "vx-installer/1.0" }
    if ($env:GITHUB_TOKEN) {
        $headers["Authorization"] = "Bearer $env:GITHUB_TOKEN"
    }

    $maxRetries = 3
    for ($i = 1; $i -le $maxRetries; $i++) {
        try {
            $wc = New-Object System.Net.WebClient
            foreach ($k in $headers.Keys) { $wc.Headers.Add($k, $headers[$k]) }
            $wc.DownloadFile($Url, $Dest)
            if ((Test-Path $Dest) -and (Get-Item $Dest).Length -gt 1024) {
                return $true
            }
        } catch {
            if ($i -lt $maxRetries) { Start-Sleep -Seconds 2 }
        }
        Remove-Item $Dest -Force -ErrorAction SilentlyContinue
    }
    return $false
}

function Get-LatestVersion {
    $headers = @{
        "User-Agent" = "vx-installer/1.0"
        "Accept"     = "application/vnd.github+json"
    }
    if ($env:GITHUB_TOKEN) {
        $headers["Authorization"] = "Bearer $env:GITHUB_TOKEN"
    }

    try {
        $apiUrl = "https://api.github.com/repos/$RepoOwner/$RepoName/releases?per_page=20"
        $releases = Invoke-RestMethod -Uri $apiUrl -Headers $headers -TimeoutSec 30

        foreach ($release in $releases) {
            if (-not $release.prerelease -and -not $release.draft -and $release.assets -and $release.assets.Count -gt 0) {
                return ($release.tag_name -replace '^(vx-)?v', '')
            }
        }
    } catch {
        # fallback handled by latest/download candidate
    }

    return $null
}

# ── Main ──────────────────────────────────────────────────────────────────────

function Main {

    $platform = Get-Platform

    Write-Step "Installing vx for Windows..."
    Write-Step "Detected: Windows -> $platform"

    $releaseBaseUrls = Get-ReleaseBaseUrls

    # Resolve download URL
    if ($Version -and $Version -ne "latest") {
        # Normalize version tag
        $ver = $Version -replace '^(vx-)?v', ''
        # Try v{ver} first (v0.7.0+), then vx-v{ver} (legacy)
        $archiveCandidates = @(
            @{ Tag = "v$ver";    Archive = "vx-$ver-$platform.zip" },
            @{ Tag = "v$ver";    Archive = "vx-$platform.zip" },
            @{ Tag = "vx-v$ver"; Archive = "vx-$ver-$platform.zip" },
            @{ Tag = "vx-v$ver"; Archive = "vx-$platform.zip" }
        )

    } else {
        # latest may not always expose unversioned assets; try robust fallbacks
        $archiveCandidates = @(
            @{ Tag = "latest"; Archive = "vx-$platform.zip" }
        )

        $latestVersion = Get-LatestVersion
        if ($latestVersion) {
            Write-Step "Resolved latest stable version with assets: $latestVersion"
            $archiveCandidates += @(
                @{ Tag = "v$latestVersion";    Archive = "vx-$latestVersion-$platform.zip" },
                @{ Tag = "v$latestVersion";    Archive = "vx-$platform.zip" },
                @{ Tag = "vx-v$latestVersion"; Archive = "vx-$latestVersion-$platform.zip" },
                @{ Tag = "vx-v$latestVersion"; Archive = "vx-$platform.zip" }
            )
        }
    }


    # Create temp dir
    $tempDir = New-TemporaryFile | ForEach-Object { Remove-Item $_; New-Item -ItemType Directory -Path $_ }

    try {
        $archivePath = $null
        $usedTag     = $null


        foreach ($releaseBaseUrl in $releaseBaseUrls) {
            foreach ($combo in $archiveCandidates) {
                $tag     = $combo.Tag
                $archive = $combo.Archive

                if ($tag -eq "latest") {
                    $url = "$releaseBaseUrl/latest/download/$archive"
                } else {
                    $url = "$releaseBaseUrl/download/$tag/$archive"
                }

                Write-Step "Downloading from: $url"
                $dest = Join-Path $tempDir $archive

                if (Invoke-Download -Url $url -Dest $dest) {
                    $archivePath = $dest
                    $usedTag     = $tag

                    break
                }
            }

            if ($archivePath) { break }
        }


        if (-not $archivePath) {
            Write-Fail "Download failed. Please check your internet connection or specify a version:"
            Write-Host "  `$env:VX_VERSION='0.8.4'; irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex"
            exit 1
        }

        # Extract
        Write-Step "Extracting..."
        if (-not (Test-Path $InstallDir)) {
            New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        }
        Expand-Archive -Path $archivePath -DestinationPath $tempDir -Force

        # Find binary
        $binary = Get-ChildItem -Path $tempDir -Filter "vx.exe" -Recurse | Select-Object -First 1
        if (-not $binary) {
            Write-Fail "vx.exe not found in archive"
            exit 1
        }

        $destBin = Join-Path $InstallDir "vx.exe"
        Copy-Item -Path $binary.FullName -Destination $destBin -Force

        # Detect installed version
        $installedVersion = & $destBin --version 2>&1 | Select-String '\d+\.\d+\.\d+' | ForEach-Object { $_.Matches[0].Value } | Select-Object -First 1
        if (-not $installedVersion) { $installedVersion = $usedTag }

        Write-Ok "Installed: vx $installedVersion"

        # Update PATH
        $currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
        if ($currentPath -notlike "*$InstallDir*") {
            [Environment]::SetEnvironmentVariable("PATH", "$InstallDir;$currentPath", "User")
            $env:PATH = "$InstallDir;$env:PATH"
            Write-Ok "Added to user PATH"
        }

        Write-Host ""
        Write-Ok "vx installed successfully!"
        Write-Host ""
        Write-Host "  Run: vx --help" -ForegroundColor Gray
        Write-Host "  Docs: https://github.com/$RepoOwner/$RepoName" -ForegroundColor Gray
        Write-Host ""
        Write-Host "  Restart your terminal or run:" -ForegroundColor Gray
        Write-Host "    `$env:PATH = `"$InstallDir;`$env:PATH`"" -ForegroundColor Gray
    }
    finally {
        Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

Main
