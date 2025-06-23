# VX Git Helper Functions for PowerShell
# Source this file in your PowerShell profile for convenient aliases

function Git-Commit-Quick-Test {
    $env:VX_QUICK_TEST = "true"
    git commit @args
    Remove-Item env:VX_QUICK_TEST
}

function Git-Commit-Strict {
    $env:VX_STRICT_MODE = "true"
    git commit @args
    Remove-Item env:VX_STRICT_MODE
}

function Git-Commit-No-Format {
    $env:VX_AUTO_FIX = "false"
    git commit @args
    Remove-Item env:VX_AUTO_FIX
}

function VX-Quality-Check {
    & "scripts/quality-check.ps1" @args
}

function VX-Format-Code {
    cargo fmt --all
}

function VX-Clippy-Check {
    cargo clippy --all-targets --all-features
}

# Aliases
Set-Alias gcq Git-Commit-Quick-Test
Set-Alias gcs Git-Commit-Strict
Set-Alias gcn Git-Commit-No-Format
Set-Alias vxq VX-Quality-Check
Set-Alias vxf VX-Format-Code
Set-Alias vxc VX-Clippy-Check

Write-Host "VX Git helper functions loaded!" -ForegroundColor Green
Write-Host "Available commands:" -ForegroundColor Cyan
Write-Host "  gcq  - Commit with quick tests" -ForegroundColor Yellow
Write-Host "  gcs  - Commit in strict mode" -ForegroundColor Yellow
Write-Host "  gcn  - Commit without auto-formatting" -ForegroundColor Yellow
Write-Host "  vxq  - Run quality check" -ForegroundColor Yellow
Write-Host "  vxf  - Format code" -ForegroundColor Yellow
Write-Host "  vxc  - Run clippy" -ForegroundColor Yellow
