#!/usr/bin/env pwsh
# Quick test for vx check command
param(
    [switch]$Verbose
)

$ErrorActionPreference = "Continue"

# Colors
function Write-Success { Write-Host $args -ForegroundColor Green }
function Write-Fail { Write-Host $args -ForegroundColor Red }
function Write-Info { Write-Host $args -ForegroundColor Cyan }

# Find vx binary
$ProjectRoot = Split-Path -Parent $PSScriptRoot
$VxBinary = Join-Path $ProjectRoot "target\debug\vx.exe"

if (-not (Test-Path $VxBinary)) {
    $VxBinary = Join-Path $ProjectRoot "target\release\vx.exe"
}

if (-not (Test-Path $VxBinary)) {
    Write-Fail "❌ vx binary not found. Run: cargo build"
    exit 1
}

Write-Info "Testing vx check command"
Write-Info "Binary: $VxBinary"
Write-Info ""

# Test cases
$TestCases = @(
    @{
        Name = "Check node (should be available on most systems)"
        Command = @("check", "node", "--quiet")
        ExpectSuccess = $null  # Don't enforce, depends on system
    },
    @{
        Name = "Check systemctl on Windows (should fail - platform not supported)"
        Command = @("check", "systemctl", "--quiet")
        ExpectSuccess = $false
        OnlyOn = "Windows"
    },
    @{
        Name = "Check spack on Windows (should fail - platform not supported)"
        Command = @("check", "spack", "--quiet")
        ExpectSuccess = $false
        OnlyOn = "Windows"
    },
    @{
        Name = "Check go with --detailed"
        Command = @("check", "go", "--detailed")
        ExpectSuccess = $null
    },
    @{
        Name = "Check unknown runtime (should fail)"
        Command = @("check", "unknown-runtime-xyz", "--quiet")
        ExpectSuccess = $false
    }
)

$Results = @{
    Passed = 0
    Failed = 0
    Skipped = 0
}

foreach ($test in $TestCases) {
    Write-Info "Testing: $($test.Name)"

    # Check platform requirement
    if ($test.OnlyOn) {
        $currentOS = if ($IsWindows -or $env:OS -eq "Windows_NT") { "Windows" }
                     elseif ($IsMacOS) { "macOS" }
                     else { "Linux" }

        if ($currentOS -ne $test.OnlyOn) {
            Write-Host "  ⚠ Skipped (only for $($test.OnlyOn))" -ForegroundColor Yellow
            $Results.Skipped++
            continue
        }
    }

    try {
        $output = & $VxBinary $test.Command 2>&1
        $exitCode = $LASTEXITCODE

        if ($Verbose) {
            Write-Host "  Exit code: $exitCode" -ForegroundColor DarkGray
            Write-Host "  Output: $($output -join '; ')" -ForegroundColor DarkGray
        }

        # Check expectation
        $success = $exitCode -eq 0

        if ($null -eq $test.ExpectSuccess) {
            # No expectation, just report
            if ($success) {
                Write-Success "  ✓ Available (exit: $exitCode)"
            } else {
                Write-Host "  ✗ Not available (exit: $exitCode)" -ForegroundColor Yellow
            }
            $Results.Passed++
        } elseif ($test.ExpectSuccess -eq $success) {
            Write-Success "  ✓ Passed (exit: $exitCode)"
            $Results.Passed++
        } else {
            Write-Fail "  ✗ Failed - Expected success=$($test.ExpectSuccess), got exit=$exitCode"
            if ($output) {
                Write-Host "    Output: $($output -join '; ')" -ForegroundColor Red
            }
            $Results.Failed++
        }
    } catch {
        Write-Fail "  ✗ Exception: $_"
        $Results.Failed++
    }

    Write-Host ""
}

# Summary
Write-Info "=== Test Summary ==="
Write-Info "Total: $(($Results.Passed + $Results.Failed + $Results.Skipped))"
Write-Success "Passed: $($Results.Passed)"
Write-Fail "Failed: $($Results.Failed)"
Write-Host "Skipped: $($Results.Skipped)" -ForegroundColor Yellow

if ($Results.Failed -eq 0) {
    Write-Success "`n✓ All tests passed!"
    exit 0
} else {
    Write-Fail "`n✗ Some tests failed"
    exit 1
}
