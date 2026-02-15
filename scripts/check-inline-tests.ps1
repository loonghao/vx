# Check for inline tests in source files
#
# This script enforces the project convention that tests should be placed
# in separate tests/ directories, not inline in source files.
#
# Usage: .\scripts\check-inline-tests.ps1

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir

# Files that are temporarily allowed to have inline tests (whitelist)
# These should be migrated over time
$Whitelist = @(
    # Add files here that are temporarily allowed
    # Example: "crates/vx-config/src/inheritance.rs"
)

function Test-Whitelisted {
    param([string]$File)
    foreach ($allowed in $Whitelist) {
        if ($File -like "*$allowed") {
            return $true
        }
    }
    return $false
}

Write-Host "🔍 Checking for inline tests in source files..." -ForegroundColor Cyan
Write-Host ""

$foundIssues = $false
$cratesDir = Join-Path $ProjectRoot "crates"

# Find all .rs files with #[cfg(test)]
$files = Get-ChildItem -Path $cratesDir -Recurse -Filter "*.rs" |
    Where-Object { $_.FullName -notmatch "\\tests\\" } |
    ForEach-Object {
        $content = Get-Content $_.FullName -Raw -ErrorAction SilentlyContinue
        if ($content -match '#\[cfg\(test\)\]') {
            $_
        }
    }

foreach ($file in $files) {
    $relativePath = $file.FullName.Replace($ProjectRoot, "").TrimStart("\", "/")

    if (Test-Whitelisted $relativePath) {
        Write-Host "⚠️  WHITELISTED: $relativePath" -ForegroundColor Yellow
    } else {
        Write-Host "❌ INLINE TEST: $relativePath" -ForegroundColor Red
        $foundIssues = $true
    }
}

Write-Host ""

if ($foundIssues) {
    Write-Host "❌ Found inline tests in source files!" -ForegroundColor Red
    Write-Host ""
    Write-Host "Project convention requires tests to be in separate tests/ directories."
    Write-Host "Please move inline tests to: crates/<crate>/tests/<module>_tests.rs"
    Write-Host ""
    Write-Host "If a file must temporarily keep inline tests, add it to the whitelist"
    Write-Host "in scripts/check-inline-tests.ps1"
    exit 1
} else {
    Write-Host "✅ No inline tests found (or all are whitelisted)" -ForegroundColor Green
    exit 0
}
