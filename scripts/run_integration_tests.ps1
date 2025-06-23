# PowerShell script to run vx integration tests
# Usage: .\scripts\run_integration_tests.ps1 [test_type]
# test_type: all, quick, single, cdn, versions

param(
    [string]$TestType = "quick",
    [string]$Tool = "",
    [switch]$Verbose = $false
)

$ErrorActionPreference = "Stop"

Write-Host "🚀 VX Integration Test Runner" -ForegroundColor Green
Write-Host "=============================" -ForegroundColor Green

# Set working directory to project root
$ProjectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $ProjectRoot

Write-Host "📁 Project root: $ProjectRoot" -ForegroundColor Cyan
Write-Host "🧪 Test type: $TestType" -ForegroundColor Cyan

# Build the project first
Write-Host "🔨 Building vx project..." -ForegroundColor Yellow
try {
    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        throw "Build failed"
    }
    Write-Host "✅ Build successful" -ForegroundColor Green
} catch {
    Write-Host "❌ Build failed: $_" -ForegroundColor Red
    exit 1
}

# Prepare test arguments
$TestArgs = @("test", "--test", "comprehensive_test")
if ($Verbose) {
    $TestArgs += "--", "--nocapture"
}

# Run specific test based on type
switch ($TestType.ToLower()) {
    "all" {
        Write-Host "🔄 Running comprehensive test suite..." -ForegroundColor Yellow
        $TestArgs += "test_all_vx_tools_comprehensive"
    }
    "quick" {
        Write-Host "⚡ Running quick tests..." -ForegroundColor Yellow
        $TestArgs += "quick_tests"
    }
    "single" {
        if ([string]::IsNullOrEmpty($Tool)) {
            $Tool = "uv"  # Default to UV
        }
        Write-Host "🎯 Running single tool test for: $Tool" -ForegroundColor Yellow
        $TestArgs += "test_single_tool_$Tool"
    }
    "cdn" {
        Write-Host "⚡ Running CDN performance tests..." -ForegroundColor Yellow
        $TestArgs += "test_cdn_performance"
    }
    "versions" {
        Write-Host "📋 Running version listing tests..." -ForegroundColor Yellow
        $TestArgs += "test_version_listing_only"
    }
    default {
        Write-Host "❌ Unknown test type: $TestType" -ForegroundColor Red
        Write-Host "Available types: all, quick, single, cdn, versions" -ForegroundColor Yellow
        exit 1
    }
}

# Run the tests
Write-Host "🏃 Executing: cargo $($TestArgs -join ' ')" -ForegroundColor Cyan
try {
    & cargo @TestArgs
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Tests completed successfully!" -ForegroundColor Green
    } else {
        Write-Host "❌ Some tests failed (exit code: $LASTEXITCODE)" -ForegroundColor Red
        exit $LASTEXITCODE
    }
} catch {
    Write-Host "❌ Test execution failed: $_" -ForegroundColor Red
    exit 1
}

Write-Host "🎉 Integration test run completed!" -ForegroundColor Green
