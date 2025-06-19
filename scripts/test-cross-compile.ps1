# Test cross-compilation for different targets on Windows
# This script helps verify that our OpenSSL fixes work

param(
    [switch]$Verbose
)

$ErrorActionPreference = "Stop"

Write-Host "🔧 Testing cross-compilation fixes..." -ForegroundColor Yellow

# Function to test a target
function Test-Target {
    param(
        [string]$Target,
        [string]$Description
    )
    
    Write-Host "`n🎯 Testing $Description ($Target)..." -ForegroundColor Cyan
    
    # Add the target if not already installed
    $installedTargets = rustup target list --installed
    if ($installedTargets -notcontains $Target) {
        Write-Host "Adding target $Target..." -ForegroundColor Yellow
        rustup target add $Target
    }
    
    # Try to build
    try {
        if ($Verbose) {
            cargo build --target $Target --release --bin vx --verbose
        } else {
            cargo build --target $Target --release --bin vx
        }
        Write-Host "✅ $Description build successful" -ForegroundColor Green
        return $true
    }
    catch {
        Write-Host "❌ $Description build failed" -ForegroundColor Red
        if ($Verbose) {
            Write-Host "Error: $_" -ForegroundColor Red
        }
        return $false
    }
}

# Test native target first
Write-Host "`n🏠 Testing native target..." -ForegroundColor Cyan
try {
    if ($Verbose) {
        cargo build --release --bin vx --verbose
    } else {
        cargo build --release --bin vx
    }
    Write-Host "✅ Native build successful" -ForegroundColor Green
}
catch {
    Write-Host "❌ Native build failed" -ForegroundColor Red
    Write-Host "Error: $_" -ForegroundColor Red
    exit 1
}

# Test targets that commonly have OpenSSL issues
$targets = @(
    @{Target = "x86_64-pc-windows-gnu"; Description = "Windows GNU"},
    @{Target = "x86_64-unknown-linux-musl"; Description = "Linux musl (static)"},
    @{Target = "aarch64-pc-windows-msvc"; Description = "Windows ARM64"}
)

$successCount = 0
$totalCount = $targets.Count

foreach ($targetInfo in $targets) {
    if (Test-Target -Target $targetInfo.Target -Description $targetInfo.Description) {
        $successCount++
    }
}

Write-Host "`n📊 Cross-compilation test summary:" -ForegroundColor Yellow
Write-Host "Successful: " -NoNewline
Write-Host "$successCount" -ForegroundColor Green -NoNewline
Write-Host "/$totalCount"

if ($successCount -eq $totalCount) {
    Write-Host "`n🎉 All cross-compilation tests passed!" -ForegroundColor Green
    Write-Host "OpenSSL dependency issues have been resolved." -ForegroundColor Green
    exit 0
} else {
    Write-Host "`n⚠️  Some cross-compilation tests failed." -ForegroundColor Yellow
    Write-Host "This might be due to missing system dependencies." -ForegroundColor Yellow
    Write-Host "Check the CI configuration for required packages." -ForegroundColor Yellow
    exit 1
}
