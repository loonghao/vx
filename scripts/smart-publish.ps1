# VX Smart Publishing Script (PowerShell)
# Intelligently publishes workspace packages to crates.io
# - Checks if packages already exist on crates.io
# - Only publishes new/updated versions
# - Handles dependency order automatically
# - Provides detailed logging and error handling

param(
    [switch]$DryRun = $true,
    [switch]$ForcePublish = $false,
    [switch]$SkipTests = $false,
    [int]$WaitTime = 30
)

# Colors for output
$Red = "Red"
$Green = "Green"
$Yellow = "Yellow"
$Blue = "Cyan"
$Purple = "Magenta"

# Logging functions
function Log-Info { param($Message) Write-Host "ℹ️  $Message" -ForegroundColor $Blue }
function Log-Success { param($Message) Write-Host "✅ $Message" -ForegroundColor $Green }
function Log-Warning { param($Message) Write-Host "⚠️  $Message" -ForegroundColor $Yellow }
function Log-Error { param($Message) Write-Host "❌ $Message" -ForegroundColor $Red }
function Log-Step { param($Message) Write-Host "🔄 $Message" -ForegroundColor $Blue }

Write-Host "🚀 VX Smart Publishing Script" -ForegroundColor $Blue
Write-Host "==============================" -ForegroundColor $Blue

if ($DryRun) {
    Log-Warning "DRY RUN MODE - No actual publishing"
    Log-Info "Use -DryRun:`$false to actually publish"
}
else {
    Log-Error "LIVE MODE - Will actually publish to crates.io"
}

if ($ForcePublish) {
    Log-Warning "FORCE MODE - Will attempt to publish even if version exists"
}

if ($SkipTests) {
    Log-Warning "SKIP TESTS MODE - Will skip running tests"
}

Write-Host ""

# Publishing order based on dependencies
$packages = @(
    "crates/vx-shim", # Base dependency for vx-core
    "crates/vx-core", # Core library
    "crates/vx-tools/vx-tool-go",
    "crates/vx-tools/vx-tool-rust",
    "crates/vx-tools/vx-tool-uv",
    "crates/vx-package-managers/vx-pm-npm",
    "crates/vx-tools/vx-tool-node", # Depends on vx-pm-npm
    "crates/vx-cli", # Depends on all tools
    "."                              # Main package depends on everything
)

# Function to check if package exists on crates.io
function Test-PackageExists {
    param(
        [string]$PackageName,
        [string]$Version
    )
    
    Log-Step "Checking if $PackageName@$Version exists on crates.io..."
    
    try {
        $searchResult = cargo search $PackageName --limit 1 2>$null
        if ($LASTEXITCODE -eq 0 -and $searchResult) {
            if ($searchResult -match "^$PackageName = `"([^`"]+)`"") {
                $publishedVersion = $matches[1]
                if ($publishedVersion -eq $Version) {
                    Log-Warning "$PackageName@$Version already exists on crates.io"
                    return $true
                }
                else {
                    Log-Info "$PackageName exists but with different version: $publishedVersion (local: $Version)"
                    return $false
                }
            }
        }
        Log-Success "$PackageName not found on crates.io - ready to publish"
        return $false
    }
    catch {
        Log-Error "Failed to check package existence: $_"
        return $false
    }
}

# Function to get package metadata
function Get-PackageMetadata {
    param([string]$PackageDir)

    $manifestPath = if ($PackageDir -eq ".") { "Cargo.toml" } else { "$PackageDir/Cargo.toml" }

    if (-not (Test-Path $manifestPath)) {
        Log-Error "Cargo.toml not found at $manifestPath"
        return $null
    }

    try {
        # Change to the package directory to get the correct metadata
        $originalLocation = Get-Location
        if ($PackageDir -ne ".") {
            Set-Location $PackageDir
        }

        $metadata = cargo metadata --no-deps --format-version 1 2>$null | ConvertFrom-Json
        if ($LASTEXITCODE -ne 0) {
            Log-Error "Failed to get metadata for $PackageDir"
            return $null
        }

        # Find the package that matches the current directory
        $targetPackage = $null
        foreach ($pkg in $metadata.packages) {
            if ($PackageDir -eq "." -and $pkg.name -eq "vx") {
                $targetPackage = $pkg
                break
            }
            elseif ($PackageDir -ne "." -and $pkg.manifest_path -like "*$PackageDir*") {
                $targetPackage = $pkg
                break
            }
        }

        if (-not $targetPackage) {
            $targetPackage = $metadata.packages[0]
        }

        return @{
            Name    = $targetPackage.name
            Version = $targetPackage.version
        }
    }
    catch {
        Log-Error "Failed to parse metadata for $PackageDir`: $_"
        return $null
    }
    finally {
        Set-Location $originalLocation
    }
}

# Function to validate package before publishing
function Test-Package {
    param(
        [string]$PackageDir,
        [string]$PackageName
    )
    
    Log-Step "Validating $PackageName..."
    
    $originalLocation = Get-Location
    try {
        if ($PackageDir -ne ".") {
            Set-Location $PackageDir
        }
        
        if (-not (Test-Path "Cargo.toml")) {
            Log-Error "Cargo.toml not found in $PackageDir"
            return $false
        }
        
        # Build the package
        Log-Step "Building $PackageName..."
        cargo build --release
        if ($LASTEXITCODE -ne 0) {
            Log-Error "Build failed for $PackageName"
            return $false
        }
        
        # Run tests (unless skipped)
        if (-not $SkipTests) {
            Log-Step "Testing $PackageName..."
            cargo test
            if ($LASTEXITCODE -ne 0) {
                Log-Error "Tests failed for $PackageName"
                return $false
            }
        }
        
        # Dry run publish
        Log-Step "Dry run publish for $PackageName..."
        cargo publish --dry-run
        if ($LASTEXITCODE -ne 0) {
            Log-Error "Dry run failed for $PackageName"
            return $false
        }
        
        Log-Success "Validation passed for $PackageName"
        return $true
    }
    catch {
        Log-Error "Validation failed for $PackageName`: $_"
        return $false
    }
    finally {
        Set-Location $originalLocation
    }
}

# Function to publish a package
function Publish-Package {
    param([string]$PackageDir)
    
    $packageInfo = Get-PackageMetadata $PackageDir
    if (-not $packageInfo) {
        Log-Error "Failed to get package metadata for $PackageDir"
        return $false
    }
    
    $packageName = $packageInfo.Name
    $packageVersion = $packageInfo.Version
    
    Write-Host "📦 Processing $packageName@$packageVersion" -ForegroundColor $Purple
    Write-Host "   Directory: $PackageDir" -ForegroundColor $Purple
    
    # Check if package already exists (unless force mode)
    if (-not $ForcePublish) {
        if (Test-PackageExists $packageName $packageVersion) {
            Log-Warning "Skipping $packageName (already published)"
            Write-Host ""
            return $true
        }
    }
    
    # Validate package
    if (-not (Test-Package $PackageDir $packageName)) {
        Log-Error "Validation failed for $packageName"
        return $false
    }
    
    # Publish if not dry run
    if (-not $DryRun) {
        $originalLocation = Get-Location
        try {
            if ($PackageDir -ne ".") {
                Set-Location $PackageDir
            }
            
            Log-Step "Publishing $packageName to crates.io..."
            cargo publish
            if ($LASTEXITCODE -eq 0) {
                Log-Success "Successfully published $packageName@$packageVersion"
                
                Log-Info "Waiting $WaitTime seconds for crates.io to update..."
                Start-Sleep $WaitTime
            }
            else {
                Log-Error "Failed to publish $packageName"
                return $false
            }
        }
        catch {
            Log-Error "Publishing failed for $packageName`: $_"
            return $false
        }
        finally {
            Set-Location $originalLocation
        }
    }
    else {
        Log-Info "Dry run completed for $packageName"
    }
    
    Write-Host ""
    return $true
}

# Main execution
Log-Info "Analyzing workspace packages..."
Write-Host ""

# Display publishing plan
Write-Host "📋 Publishing Plan:" -ForegroundColor $Blue
foreach ($package in $packages) {
    $packageInfo = Get-PackageMetadata $package
    if ($packageInfo) {
        $packageName = $packageInfo.Name
        $packageVersion = $packageInfo.Version
        Write-Host "  $packageName@$packageVersion ($package)" -ForegroundColor $Green
    }
    else {
        Log-Error "Failed to get metadata for $package"
        exit 1
    }
}
Write-Host ""

# Confirmation for live mode
if (-not $DryRun) {
    Write-Host "⚠️  This will publish packages to crates.io!" -ForegroundColor $Red
    $response = Read-Host "Continue with publishing? (y/N)"
    if ($response -ne "y" -and $response -ne "Y") {
        Log-Error "Publishing cancelled by user"
        exit 1
    }
    Write-Host ""
}

# Publish each package
$failedPackages = @()
foreach ($package in $packages) {
    if (-not (Publish-Package $package)) {
        $packageInfo = Get-PackageMetadata $package
        if ($packageInfo) {
            $failedPackages += $packageInfo.Name
        }
        
        if (-not $DryRun) {
            Log-Error "Failed to publish $($packageInfo.Name) - stopping here"
            break
        }
    }
}

# Summary
Write-Host "📊 Summary:" -ForegroundColor $Blue
if ($failedPackages.Count -eq 0) {
    if ($DryRun) {
        Log-Success "All packages passed validation!"
        Log-Info "To actually publish, run: .\scripts\smart-publish.ps1 -DryRun:`$false"
    }
    else {
        Log-Success "All packages published successfully!"
        Log-Success "Users can now install with: cargo install vx"
    }
}
else {
    Log-Error "Failed packages: $($failedPackages -join ', ')"
    exit 1
}
