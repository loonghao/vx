# VX Workspace Publishing Script (PowerShell)
param(
    [switch]$DryRun = $true,
    [int]$WaitTime = 30
)

# Colors for output
$Red = "Red"
$Green = "Green" 
$Yellow = "Yellow"
$Blue = "Cyan"

Write-Host "🚀 VX Workspace Publishing Script" -ForegroundColor $Blue
Write-Host "=================================" -ForegroundColor $Blue

if ($DryRun) {
    Write-Host "⚠️  DRY RUN MODE - No actual publishing" -ForegroundColor $Yellow
    Write-Host "   Use -DryRun:`$false to actually publish" -ForegroundColor $Yellow
}
else {
    Write-Host "🔥 LIVE MODE - Will actually publish to crates.io" -ForegroundColor $Red
}

Write-Host ""

# Publishing order based on dependencies
# Level 1: No internal dependencies
$packages = @(
    "crates/vx-shim", # No internal dependencies

    # Level 2: Depends only on vx-shim
    "crates/vx-core", # Depends on vx-shim

    # Level 3: Depends on vx-core
    "crates/vx-tools/vx-tool-go", # Depends on vx-core
    "crates/vx-tools/vx-tool-rust", # Depends on vx-core
    "crates/vx-tools/vx-tool-uv", # Depends on vx-core
    "crates/vx-package-managers/vx-pm-npm", # Depends on vx-core

    # Level 4: Depends on vx-core + vx-pm-npm
    "crates/vx-tools/vx-tool-node", # Depends on vx-core + vx-pm-npm

    # Level 5: Depends on all tools
    "crates/vx-cli", # Depends on vx-core + all tools

    # Level 6: Main package depends on everything
    "."                              # Main package depends on vx-cli
)

# Function to check if package is already published
function Test-PackagePublished {
    param($PackageName, $Version)
    
    Write-Host "🔍 Checking if $PackageName@$Version is already published..." -ForegroundColor $Blue
    
    $searchResult = cargo search $PackageName --limit 1 2>$null
    if ($searchResult -match "$PackageName = `"$Version`"") {
        Write-Host "⚠️  $PackageName@$Version is already published" -ForegroundColor $Yellow
        return $true
    }
    else {
        Write-Host "✅ $PackageName@$Version is not yet published" -ForegroundColor $Green
        return $false
    }
}

# Function to get package name and version
function Get-PackageInfo {
    param($PackageDir)

    $cargoToml = if ($PackageDir -eq ".") { "Cargo.toml" } else { "$PackageDir/Cargo.toml" }

    if (-not (Test-Path $cargoToml)) {
        $cargoToml = "Cargo.toml"
    }

    # Change to the package directory to get correct metadata
    $originalLocation = Get-Location
    try {
        if ($PackageDir -ne ".") {
            Set-Location $PackageDir
        }

        # Use cargo metadata to get package info from current directory
        # Filter to only get the package in the current directory
        $metadata = cargo metadata --no-deps --format-version 1 | ConvertFrom-Json
        $currentPath = (Get-Location).Path.Replace('\', '/')

        # Find the package that matches the current directory
        $package = $metadata.packages | Where-Object {
            $packagePath = $_.manifest_path -replace '\\', '/' -replace '/Cargo\.toml$', ''
            $packagePath -eq $currentPath
        }

        if (-not $package) {
            # Fallback: if we can't find by path, use the first package
            $package = $metadata.packages[0]
        }

        return @{
            Name    = $package.name
            Version = $package.version
        }
    }
    finally {
        Set-Location $originalLocation
    }
}

# Function to publish a package
function Publish-Package {
    param($PackageDir)
    
    $packageInfo = Get-PackageInfo $PackageDir
    $packageName = $packageInfo.Name
    $packageVersion = $packageInfo.Version
    
    Write-Host "📦 Processing $packageName@$packageVersion in $PackageDir" -ForegroundColor $Blue
    
    # Check if already published
    if (Test-PackagePublished $packageName $packageVersion) {
        Write-Host "⏭️  Skipping $packageName (already published)" -ForegroundColor $Yellow
        return
    }
    
    # Change to package directory
    $originalLocation = Get-Location
    if ($PackageDir -ne ".") {
        Set-Location $PackageDir
    }
    
    try {
        Write-Host "🔨 Building $packageName..." -ForegroundColor $Blue
        cargo build --release
        if ($LASTEXITCODE -ne 0) { throw "Build failed" }
        
        Write-Host "🧪 Testing $packageName..." -ForegroundColor $Blue
        cargo test
        if ($LASTEXITCODE -ne 0) { throw "Tests failed" }
        
        if ($DryRun) {
            Write-Host "🔍 Dry run for $packageName..." -ForegroundColor $Blue
            cargo publish --dry-run
            if ($LASTEXITCODE -ne 0) { throw "Dry run failed" }
        }
        
        if (-not $DryRun) {
            Write-Host "🚀 Publishing $packageName to crates.io..." -ForegroundColor $Green
            cargo publish
            if ($LASTEXITCODE -ne 0) { throw "Publishing failed" }
            
            Write-Host "✅ Successfully published $packageName@$packageVersion" -ForegroundColor $Green
            
            Write-Host "⏳ Waiting $WaitTime seconds for crates.io to update..." -ForegroundColor $Yellow
            Start-Sleep $WaitTime
        }
        else {
            Write-Host "🔍 Dry run completed for $packageName" -ForegroundColor $Yellow
        }
    }
    catch {
        Write-Host "❌ Error processing $packageName`: $_" -ForegroundColor $Red
        throw
    }
    finally {
        # Return to original directory
        Set-Location $originalLocation
    }
    
    Write-Host ""
}

# Main execution
Write-Host "📋 Publishing order:" -ForegroundColor $Blue
foreach ($package in $packages) {
    $packageInfo = Get-PackageInfo $package
    Write-Host "  $($packageInfo.Name)@$($packageInfo.Version) ($package)" -ForegroundColor $Green
}
Write-Host ""

if (-not $DryRun) {
    $response = Read-Host "Continue with publishing? (y/N)"
    if ($response -notmatch "^[Yy]$") {
        Write-Host "❌ Publishing cancelled" -ForegroundColor $Red
        exit 1
    }
}

# Publish each package
foreach ($package in $packages) {
    try {
        Publish-Package $package
    }
    catch {
        Write-Host "❌ Failed to publish package in $package`: $_" -ForegroundColor $Red
        exit 1
    }
}

if ($DryRun) {
    Write-Host "🎉 Dry run completed successfully!" -ForegroundColor $Green
    Write-Host "💡 To actually publish, run: .\scripts\publish-workspace.ps1 -DryRun:`$false" -ForegroundColor $Yellow
}
else {
    Write-Host "🎉 All packages published successfully!" -ForegroundColor $Green
    Write-Host "🎯 Users can now install with: cargo install vx" -ForegroundColor $Green
}
