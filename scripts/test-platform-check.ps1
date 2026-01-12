#!/usr/bin/env pwsh
# Test platform check logic

$ErrorActionPreference = "Stop"

# Find vx binary
$VxBinary = if (Test-Path ".\target\release\vx.exe") {
    ".\target\release\vx.exe"
} elseif (Test-Path ".\target\debug\vx.exe") {
    ".\target\debug\vx.exe"
} else {
    throw "vx binary not found. Run 'cargo build' first."
}

Write-Host "Testing platform check logic..." -ForegroundColor Cyan
Write-Host "VX Binary: $VxBinary" -ForegroundColor Gray
Write-Host ""

# Test function
function Test-PlatformCheck {
    param(
        [string]$Runtime,
        [string]$ExpectedResult  # "supported" or "unsupported"
    )
    
    Write-Host "Testing: $Runtime" -ForegroundColor Yellow
    
    $output = & $VxBinary check $Runtime 2>&1 | Out-String
    $exitCode = $LASTEXITCODE
    
    $isUnsupported = $output -match "does not support the current platform"
    
    if ($ExpectedResult -eq "unsupported") {
        if ($isUnsupported) {
            Write-Host "  ✓ Correctly detected as unsupported" -ForegroundColor Green
            return $true
        } else {
            Write-Host "  ✗ Expected unsupported but got: $output" -ForegroundColor Red
            return $false
        }
    } else {
        if (-not $isUnsupported) {
            Write-Host "  ✓ Correctly detected as supported (exit: $exitCode)" -ForegroundColor Green
            return $true
        } else {
            Write-Host "  ✗ Expected supported but got: $output" -ForegroundColor Red
            return $false
        }
    }
}

# Test cases
$testCases = @(
    @{Runtime = "deno"; Expected = "supported"}
    @{Runtime = "docker"; Expected = "supported"}
    @{Runtime = "ffmpeg"; Expected = "supported"}
    @{Runtime = "go"; Expected = "supported"}
    @{Runtime = "node"; Expected = "supported"}
    @{Runtime = "spack"; Expected = "unsupported"}
    @{Runtime = "systemctl"; Expected = "unsupported"}
)

$passed = 0
$failed = 0

foreach ($test in $testCases) {
    if (Test-PlatformCheck -Runtime $test.Runtime -Expected $test.Expected) {
        $passed++
    } else {
        $failed++
    }
    Write-Host ""
}

# Summary
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Test Summary:" -ForegroundColor Cyan
Write-Host "  Passed: $passed" -ForegroundColor Green
Write-Host "  Failed: $failed" -ForegroundColor Red
Write-Host "========================================" -ForegroundColor Cyan

if ($failed -eq 0) {
    Write-Host "✓ All tests passed!" -ForegroundColor Green
    exit 0
} else {
    Write-Host "✗ Some tests failed!" -ForegroundColor Red
    exit 1
}
