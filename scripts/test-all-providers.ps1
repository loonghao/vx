#!/usr/bin/env pwsh
# Test All VX Providers
# This script tests all VX providers by executing their commands in a clean temporary environment

param(
    [switch]$KeepCache,  # Don't delete cache after testing
    [switch]$Verbose,    # Verbose output
    [string]$Filter = "" # Filter providers by name (e.g., "node", "go")
)

$ErrorActionPreference = "Continue"
$ProgressPreference = "SilentlyContinue"

# Colors
function Write-Success { Write-Host $args -ForegroundColor Green }
function Write-Info { Write-Host $args -ForegroundColor Cyan }
function Write-Warning { Write-Host $args -ForegroundColor Yellow }
function Write-Error { Write-Host $args -ForegroundColor Red }
function Write-Section { Write-Host "`n=== $args ===" -ForegroundColor Magenta }

# Configuration
# Find project root by looking for Cargo.toml
$ScriptPath = if ($PSScriptRoot) { 
    $PSScriptRoot 
} elseif ($MyInvocation.MyCommand.Path) { 
    Split-Path -Parent $MyInvocation.MyCommand.Path 
} else { 
    Get-Location 
}

# Navigate up to find project root (where Cargo.toml exists)
$ProjectRoot = $ScriptPath
while ($ProjectRoot -and -not (Test-Path (Join-Path $ProjectRoot "Cargo.toml"))) {
    $ProjectRoot = Split-Path -Parent $ProjectRoot
}

if (-not $ProjectRoot) {
    Write-Error "❌ Could not find project root (no Cargo.toml found)"
    exit 1
}

$ProvidersDir = Join-Path $ProjectRoot "crates\vx-providers"
$TempVxHome = Join-Path ([System.IO.Path]::GetTempPath()) "vx-test-$(Get-Date -Format 'yyyyMMdd-HHmmss')"
$VxBinary = Join-Path $ProjectRoot "target\debug\vx.exe"

# Check if vx is built
if (-not (Test-Path $VxBinary)) {
    Write-Error "❌ VX binary not found at: $VxBinary"
    Write-Info "Run: cargo build"
    exit 1
}

Write-Section "VX Provider Test Suite"
Write-Info "Project Root: $ProjectRoot"
Write-Info "Providers Dir: $ProvidersDir"
Write-Info "Temp VX_HOME: $TempVxHome"
Write-Info "VX Binary: $VxBinary"

# Create temporary VX_HOME
New-Item -ItemType Directory -Path $TempVxHome -Force | Out-Null
$env:VX_HOME = $TempVxHome

# Test results
$TestResults = @{
    Total = 0
    Passed = 0
    Failed = 0
    Skipped = 0
    Providers = @()
}

# Parse provider.toml to extract runtime names
# NOTE: Function must be defined before it's called
function Get-RuntimesFromToml {
    param([string]$TomlPath)
    
    $runtimes = @()
    $content = Get-Content $TomlPath -Raw
    
    # Simple TOML parsing for [[runtimes]] sections
    $pattern = '\[\[runtimes\]\]\s+name\s*=\s*"([^"]+)"'
    $matches = [regex]::Matches($content, $pattern)
    
    foreach ($match in $matches) {
        $runtimes += $match.Groups[1].Value
    }
    
    return $runtimes
}

# Test a single command
function Test-VxCommand {
    param(
        [string]$Provider,
        [string]$Runtime,
        [string]$Command
    )
    
    $TestResults.Total++
    
    $cmdArgs = @($Command)
    if ($Command -ne "list") {
        $cmdArgs = @($Runtime, "--version")
    }
    
    try {
        $output = & $VxBinary $cmdArgs 2>&1
        $exitCode = $LASTEXITCODE
        
        if ($exitCode -eq 0) {
            $TestResults.Passed++
            Write-Success "  ✓ vx $($cmdArgs -join ' ')"
            if ($Verbose) {
                Write-Host "    Output: $($output -join '; ')" -ForegroundColor DarkGray
            }
            return @{ Success = $true; Output = $output }
        } else {
            $TestResults.Failed++
            Write-Error "  ✗ vx $($cmdArgs -join ' ') (exit: $exitCode)"
            # Always show error output for failed tests
            if ($output) {
                $errorLines = $output | Select-Object -First 3
                Write-Host "    Error: $($errorLines -join '; ')" -ForegroundColor Red
            }
            return @{ Success = $false; Output = $output; ExitCode = $exitCode }
        }
    } catch {
        $TestResults.Failed++
        Write-Error "  ✗ vx $($cmdArgs -join ' ') - Exception: $_"
        return @{ Success = $false; Error = $_ }
    }
}

# Check if runtime supports the current platform
# Returns: $true if platform is supported, $false if not supported
function Test-RuntimePlatformSupported {
    param([string]$Runtime)
    
    try {
        # Use 'vx test' command with --platform-only flag
        $output = & $VxBinary test $Runtime --platform-only 2>&1 | Out-String
        $exitCode = $LASTEXITCODE
        
        # Exit code 0 means platform is supported
        return $exitCode -eq 0
    } catch {
        # On error, assume platform is supported (avoid false negatives)
        return $true
    }
}

# Discover all providers
Write-Section "Discovering Providers"
$AllProviders = Get-ChildItem -Path $ProvidersDir -Directory | Where-Object {
    Test-Path (Join-Path $_.FullName "provider.toml")
}

if ($Filter) {
    $AllProviders = $AllProviders | Where-Object { $_.Name -like "*$Filter*" }
    Write-Info "Filtered to providers matching: $Filter"
}

Write-Info "Found $($AllProviders.Count) providers"

# Count total runtimes for progress tracking
$TotalRuntimes = 0
foreach ($provider in $AllProviders) {
    $tomlPath = Join-Path $provider.FullName "provider.toml"
    $runtimes = Get-RuntimesFromToml -TomlPath $tomlPath
    $TotalRuntimes += $runtimes.Count
}
Write-Info "Total runtimes to test: $TotalRuntimes"
$CurrentRuntime = 0

# Test each provider
foreach ($provider in $AllProviders) {
    Write-Section "Testing Provider: $($provider.Name)"
    
    $tomlPath = Join-Path $provider.FullName "provider.toml"
    $runtimes = Get-RuntimesFromToml -TomlPath $tomlPath
    
    if ($runtimes.Count -eq 0) {
        Write-Warning "  ⚠ No runtimes found in provider.toml"
        $TestResults.Skipped++
        continue
    }
    
    Write-Info "  Runtimes: $($runtimes -join ', ')"
    
    $providerResult = @{
        Name = $provider.Name
        Runtimes = $runtimes
        Tests = @()
    }
    
    # Test: vx list <runtime>
    foreach ($runtime in $runtimes) {
        $CurrentRuntime++
        $Remaining = $TotalRuntimes - $CurrentRuntime
        Write-Info "  [$CurrentRuntime/$TotalRuntimes] Testing: $runtime (remaining: $Remaining)"
        
        # Check if runtime supports the current platform
        $platformSupported = Test-RuntimePlatformSupported -Runtime $runtime
        if (-not $platformSupported) {
            Write-Warning "  ⚠ Runtime '$runtime' does not support the current platform (skipped)"
            $TestResults.Skipped++
            $providerResult.Tests += @{
                Command = "check $runtime"
                Result = @{ Success = $false; Skipped = $true; Reason = "Platform not supported" }
            }
            continue
        }
        
        # Test list command
        $listResult = Test-VxCommand -Provider $provider.Name -Runtime $runtime -Command "list"
        $providerResult.Tests += @{
            Command = "list $runtime"
            Result = $listResult
        }
        
        # Test --version (will trigger auto-install on first run)
        $versionResult = Test-VxCommand -Provider $provider.Name -Runtime $runtime -Command "--version"
        $providerResult.Tests += @{
            Command = "$runtime --version"
            Result = $versionResult
        }
        
        # Small delay to avoid rate limiting
        Start-Sleep -Milliseconds 100
    }
    
    $TestResults.Providers += $providerResult
}

# Generate summary report
Write-Section "Test Summary"
Write-Info "Total Tests: $($TestResults.Total)"
Write-Success "Passed: $($TestResults.Passed)"
Write-Error "Failed: $($TestResults.Failed)"
Write-Warning "Skipped: $($TestResults.Skipped)"

$successRate = if ($TestResults.Total -gt 0) { 
    [math]::Round(($TestResults.Passed / $TestResults.Total) * 100, 2) 
} else { 
    0 
}
Write-Info "Success Rate: $successRate%"

# Per-provider summary
Write-Section "Provider Details"
foreach ($provider in $TestResults.Providers) {
    $passed = ($provider.Tests | Where-Object { $_.Result.Success }).Count
    $total = $provider.Tests.Count
    $status = if ($passed -eq $total) { "✓" } else { "✗" }
    
    $statusColor = if ($passed -eq $total) { "Green" } else { "Red" }
    Write-Host "  $status " -ForegroundColor $statusColor -NoNewline
    Write-Host "$($provider.Name): $passed/$total tests passed"
}

# Save detailed report
$reportPath = Join-Path $TempVxHome "test-report.json"
$TestResults | ConvertTo-Json -Depth 10 | Set-Content $reportPath
Write-Info "`nDetailed report saved to: $reportPath"

# List cache contents
Write-Section "Cache Contents"
$cacheSize = (Get-ChildItem -Path $TempVxHome -Recurse -File | Measure-Object -Property Length -Sum).Sum
$cacheSizeMB = [math]::Round($cacheSize / 1MB, 2)
Write-Info "Cache size: $cacheSizeMB MB"
Write-Info "Cache path: $TempVxHome"

if ($Verbose) {
    Write-Info "`nInstalled versions:"
    Get-ChildItem -Path $TempVxHome -Recurse -Directory -Depth 2 | 
        Where-Object { $_.Parent.Name -eq "versions" } |
        ForEach-Object { Write-Host "  - $($_.Parent.Parent.Name)/$($_.Name)" -ForegroundColor DarkGray }
}

# Cleanup
if (-not $KeepCache) {
    Write-Section "Cleaning Up"
    Write-Info "Removing temporary cache: $TempVxHome"
    try {
        Remove-Item -Path $TempVxHome -Recurse -Force -ErrorAction Stop
        Write-Success "✓ Cache cleaned"
    } catch {
        Write-Warning "⚠ Failed to clean cache: $_"
        Write-Info "You can manually delete: $TempVxHome"
    }
} else {
    Write-Section "Cache Preserved"
    Write-Info "Cache kept at: $TempVxHome"
    Write-Info "To clean up manually: Remove-Item -Recurse -Force '$TempVxHome'"
}

# Exit with appropriate code
Write-Section "Test Result"
if ($TestResults.Failed -eq 0) {
    Write-Success "✓ All tests passed!"
    exit 0
} else {
    Write-Error "✗ Some tests failed"
    exit 1
}
