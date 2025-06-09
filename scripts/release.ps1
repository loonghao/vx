# Release script for vx (PowerShell version)
# This script helps create and publish releases

param(
    [Parameter(Mandatory=$true)]
    [string]$VersionType
)

$ErrorActionPreference = "Stop"

# Helper functions
function Write-Info {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Green
}

function Write-Warn {
    param([string]$Message)
    Write-Host "[WARN] $Message" -ForegroundColor Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

# Check if we're on main branch
function Test-Branch {
    $currentBranch = git branch --show-current
    if ($currentBranch -ne "main") {
        Write-Error "Must be on main branch to create a release. Current branch: $currentBranch"
        exit 1
    }
}

# Check if working directory is clean
function Test-Clean {
    $status = git status --porcelain
    if ($status) {
        Write-Error "Working directory is not clean. Please commit or stash changes."
        git status --short
        exit 1
    }
}

# Get current version from Cargo.toml
function Get-CurrentVersion {
    $content = Get-Content "Cargo.toml"
    $versionLine = $content | Where-Object { $_ -match '^version = ' }
    if ($versionLine -match 'version = "([^"]+)"') {
        return $matches[1]
    }
    throw "Could not find version in Cargo.toml"
}

# Update version in Cargo.toml
function Update-Version {
    param([string]$NewVersion)
    
    Write-Info "Updating version to $NewVersion in Cargo.toml"
    $content = Get-Content "Cargo.toml"
    $content = $content -replace '^version = ".*"', "version = `"$NewVersion`""
    $content | Set-Content "Cargo.toml"
}

# Create and push tag
function New-Tag {
    param([string]$Version)
    
    $tag = "v$Version"
    
    Write-Info "Creating tag $tag"
    git add Cargo.toml
    git commit -m "chore: bump version to $Version"
    git tag -a $tag -m "Release $tag"
    
    Write-Info "Pushing tag $tag"
    git push origin main
    git push origin $tag
}

# Calculate new version
function Get-NewVersion {
    param(
        [string]$CurrentVersion,
        [string]$VersionType
    )
    
    $parts = $CurrentVersion.Split('.')
    $major = [int]$parts[0]
    $minor = [int]$parts[1]
    $patch = [int]$parts[2]
    
    switch ($VersionType.ToLower()) {
        "patch" {
            $patch++
        }
        "minor" {
            $minor++
            $patch = 0
        }
        "major" {
            $major++
            $minor = 0
            $patch = 0
        }
        default {
            # Assume it's a specific version
            return $VersionType
        }
    }
    
    return "$major.$minor.$patch"
}

# Main release function
function Start-Release {
    param([string]$VersionType)
    
    if (-not $VersionType) {
        Write-Host "Usage: .\release.ps1 <patch|minor|major|VERSION>"
        Write-Host "Examples:"
        Write-Host "  .\release.ps1 patch    # 0.1.0 -> 0.1.1"
        Write-Host "  .\release.ps1 minor    # 0.1.0 -> 0.2.0"
        Write-Host "  .\release.ps1 major    # 0.1.0 -> 1.0.0"
        Write-Host "  .\release.ps1 1.2.3    # Set specific version"
        exit 1
    }
    
    Test-Branch
    Test-Clean
    
    $currentVersion = Get-CurrentVersion
    Write-Info "Current version: $currentVersion"
    
    $newVersion = Get-NewVersion -CurrentVersion $currentVersion -VersionType $VersionType
    Write-Info "New version: $newVersion"
    
    # Confirm with user
    $confirmation = Read-Host "Create release $newVersion? (y/N)"
    if ($confirmation -ne 'y' -and $confirmation -ne 'Y') {
        Write-Info "Release cancelled"
        exit 0
    }
    
    # Update version and create tag
    Update-Version -NewVersion $newVersion
    New-Tag -Version $newVersion
    
    Write-Info "Release $newVersion created successfully!"
    Write-Info "GitHub Actions will automatically build and publish the release."
    Write-Info "Check the progress at: https://github.com/loonghao/vx/actions"
}

# Run the release
Start-Release -VersionType $VersionType
