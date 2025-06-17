# vx Universal Development Tool Manager Installation Script for Windows
# This script detects your platform and installs vx using the appropriate package manager

using namespace System.Runtime.InteropServices

<#
.SYNOPSIS
    Installs the vx Cross-Platform Universal Development Tool Manager.
.DESCRIPTION
    This script installs the vx tool for managing development environments across multiple platforms.
    It detects the platform and architecture, downloads the appropriate binary from GitHub, and sets it up in the user's PATH.
.PARAMETER Version
    The version of vx to install. Defaults to "latest".
.PARAMETER InstallDir
    The directory where vx will be installed. Defaults to "$env:USERPROFILE\.vx\bin".
.PARAMETER PackageManager
    The package manager to use for installation. Defaults to "auto". If set to "auto", it will try to detect the best package manager available.
.PARAMETER NoPackageManager
    If specified, the script will not use any package manager and will install vx directly.
.PARAMETER GitHubToken
    GitHub token for authentication when accessing private repositories or rate-limited API endpoints. Defaults to the environment variable `VX_GITHUB_TOKEN`.
.PARAMETER Help
    Displays help information for the script.
.EXAMPLE
    .\install-release.ps1 -Version "latest" -InstallDir "$env:USERPROFILE\.vx\bin"

    Installs the latest version of vx to the specified directory.
.EXAMPLE
    .\install-release.ps1 -Version "v1.0.0" -InstallDir "C:\vx\bin"

    Installs version 1.0.0 of vx to the specified directory.
#>

param(
    [string]$Version = "latest",
    [string]$InstallDir = "$env:USERPROFILE\.vx\bin",
    [string]$PackageManager = "auto",
    [switch]$NoPackageManager,
    [string]$GitHubToken = $env:VX_GITHUB_TOKEN,
    [switch]$Help
)

$ErrorActionPreference = "Stop"

# Display help if requested
if ($Help) {
    Write-Host "Usage: .\install-release.ps1 [-Version <version>] [-InstallDir <directory>] [-PackageManager <manager>] [-NoPackageManager] [-GitHubToken <token>]" -ForegroundColor Cyan
    Write-Host "Installs the vx tool for managing development environments." -ForegroundColor Cyan
    Write-Host "Parameters:" -ForegroundColor Cyan
    Write-Host "  -Version: The version of vx to install (default: latest)." -ForegroundColor Cyan
    Write-Host "  -InstallDir: The directory where vx will be installed (default: $env:USERPROFILE\.vx\bin)." -ForegroundColor Cyan
    Write-Host "  -PackageManager: The package manager to use for installation (default: auto)." -ForegroundColor Cyan
    Write-Host "  -NoPackageManager: If specified, the script will not use any package manager and will install vx directly." -ForegroundColor Cyan
    Write-Host "  -GitHubToken: GitHub token for authentication (default: VX_GITHUB_TOKEN environment variable)." -ForegroundColor Cyan
    exit 0
}

# GitHub repository information
$Owner = "loonghao"
$Repo = "vx"
$SupportedPlatform = @(
    [System.Tuple]::Create([OSPlatform]::Windows, [Architecture]::X64)
)

Write-Host "Installing vx..." -ForegroundColor Green

# Determine the platform and architecture
Set-Variable -Name "Platform" -Value (New-Module -AsCustomObject -ScriptBlock {
        [OSPlatform]$OS = [OSPlatform]::Linux
        [Architecture]$Arch = [Architecture]::Arm

        $OS = @(
            [OSPlatform]::Windows,
            [OSPlatform]::Linux,
            [OSPlatform]::OSX
        ) | ForEach-Object { if ([RuntimeInformation]::IsOSPlatform($_)) { return $_ } }

        if ($PSVersionTable.Major -ge 6) {
            $Arch = [RuntimeInformation]::OSArchitecture
        }
        else {
            # On Windows PowerShell
            $win_processor_arch = $env:PROCESSOR_ARCHITECTURE
            switch ($win_processor_arch) {
                "AMD64" { $Arch = [Architecture]::X64 }
                "x86" { $Arch = [Architecture]::X86 }
                "ARM64" { $Arch = [Architecture]::Arm64 }
                "ARM" { $Arch = [Architecture]::Arm }
                default { throw "Unsupported architecture: $win_processor_arch" }
            }
        }

        Export-ModuleMember -Variable *
    }) -Option Private, ReadOnly

Write-Debug "Detected platform: $($Platform.OS), architecture: $($Platform.Arch)"

# Check if the platform is supported
if ($Platform.OS -eq [OSPlatform]::Unknown -or $Platform.Arch -eq [Architecture]::None) {
    Write-Error "Unsupported platform: $($Platform.OS) $($Platform.Arch)"
    exit 1
}
elseif ($SupportedPlatform -notcontains [System.Tuple]::Create($Platform.OS, $Platform.Arch)) {
    Write-Error "This script only supports the following platform/architecture combinations: $($SupportedPlatform | ForEach-Object { "$($_.Item1) $($_.Item2)" } -join ', ')"
    exit 1
}

# Create installation directory
if (!(Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

try {
    $ArtifactNameToken = [PSCustomObject]@{
        Platform            = if ($Platform.OS -eq [OSPlatform]::Windows) { "windows" } elseif ($Platform.OS -eq [OSPlatform]::Linux) { "linux" } elseif ($Platform.OS -eq [OSPlatform]::OSX) { "macos" } else { throw "Unsupported OSPlatform: $($Platform.OS)" }
        Arch                = if ($Platform.Arch -eq [Architecture]::X64) { "amd64" } elseif ($Platform.Arch -eq [Architecture]::X86) { "i386" } elseif ($Platform.Arch -eq [Architecture]::Arm64) { "arm64" } elseif ($Platform.Arch -eq [Architecture]::Arm) { "arm" } else { throw "Unsupported Architecture: $($Platform.Arch)" }
        DistributeExtension = if ($Platform.OS -eq [OSPlatform]::Windows) {
            ".zip"
        }
        elseif ($Platform.OS -eq [OSPlatform]::Linux -or $Platform.OS -eq [OSPlatform]::OSX) {
            ".tar.gz"
        }
        else {
            throw "Unsupported OSPlatform: $($Platform.OS)"
        }
        ExecutableExtension = if ($Platform.OS -eq [OSPlatform]::Windows) {
            ".exe"
        }
        elseif ($Platform.OS -eq [OSPlatform]::Linux -or $Platform.OS -eq [OSPlatform]::OSX) {
            ""
        }
        else {
            throw "Unsupported OSPlatform: $($Platform.OS)"
        }
    }
    $FileName = "vx-$($ArtifactNameToken.Platform)-$($ArtifactNameToken.Arch)$($ArtifactNameToken.DistributeExtension)"

    # If no version is specified, fetch the latest release from GitHub
    if ($Version -eq "latest" -or $Version -eq "") {
        $Version = "latest"
        Write-Host "Fetching the latest version of vx..." -ForegroundColor Yellow
        # Get the latest release
        try {
            # build the request header with GitHub token if provided
            $request_args = @{}
            if ($GitHubToken) {
                Write-Host "Using GitHub token for authentication." -ForegroundColor Yellow
                $request_args = @{
                    Headers = @{
                        Authorization = "Bearer $GitHubToken"
                    }
                }
            }
            $releases = Invoke-RestMethod "https://api.github.com/repos/$Owner/$Repo/releases" @request_args
        }
        catch {
            Write-Error "Failed to fetch releases from GitHub. Please check your network connection or GitHub API status."
            exit 1
        }
        foreach ($release in $releases) {
            $assets = if ($release.assets -is [string]) { $release.assets | ConvertFrom-Json } else { $release.assets }
            if ($assets | Where-Object { $_.name -eq $FileName }) {
                $Version = $release.tag_name
                break
            }
        }
        if ($Version -ne "latest") {
            Write-Host "Latest version found: $Version" -ForegroundColor Green
        }
    }
    if ($Version -eq "latest" -or $Version -eq "") {
        Write-Error "Could not determine the latest version of vx."
        exit 1
    }

    # Construct download URL based on platform
    $DownloadUrl = "https://github.com/$Owner/$Repo/releases/download/$Version/$FileName"
    Write-Host "Downloading vx $Version for $($Platform.OS) $($Platform.Arch)..." -ForegroundColor Yellow

    # Download the binary directly
    $DestPath = Join-Path $InstallDir "vx$($ArtifactNameToken.ExecutableExtension)"
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
