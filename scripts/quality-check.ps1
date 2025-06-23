# VX Code Quality Check Script
# This script performs comprehensive code quality checks to prevent code smells

param(
    [switch]$Fix = $false,
    [switch]$Strict = $false,
    [string]$Path = "."
)

Write-Host "🔍 VX Code Quality Check" -ForegroundColor Cyan
Write-Host "=========================" -ForegroundColor Cyan

$ErrorCount = 0
$WarningCount = 0

function Write-Error-Message($message) {
    Write-Host "❌ $message" -ForegroundColor Red
    $script:ErrorCount++
}

function Write-Warning-Message($message) {
    Write-Host "⚠️  $message" -ForegroundColor Yellow
    $script:WarningCount++
}

function Write-Success-Message($message) {
    Write-Host "✅ $message" -ForegroundColor Green
}

function Write-Info-Message($message) {
    Write-Host "ℹ️  $message" -ForegroundColor Blue
}

# 1. Check for unwrap() in production code
Write-Info-Message "Checking for unwrap() in production code..."
$unwrapFiles = Get-ChildItem -Path $Path -Recurse -Include "*.rs" | 
    Where-Object { $_.FullName -notmatch "\\tests\\" -and $_.FullName -notmatch "test_" } |
    Select-String -Pattern "\.unwrap\(\)" |
    Group-Object Path

if ($unwrapFiles) {
    foreach ($file in $unwrapFiles) {
        Write-Error-Message "Found $($file.Count) unwrap() calls in production code: $($file.Name)"
        if ($Strict) {
            $file.Group | ForEach-Object { 
                Write-Host "  Line $($_.LineNumber): $($_.Line.Trim())" -ForegroundColor DarkRed 
            }
        }
    }
} else {
    Write-Success-Message "No unwrap() calls found in production code"
}

# 2. Check for TODO/FIXME/HACK comments
Write-Info-Message "Checking for TODO/FIXME/HACK comments..."
$todoPatterns = @("TODO", "FIXME", "HACK", "XXX")
$todoFiles = @()

foreach ($pattern in $todoPatterns) {
    $matches = Get-ChildItem -Path $Path -Recurse -Include "*.rs" |
        Select-String -Pattern $pattern |
        Where-Object { $_.Line -notmatch "^//" -or $_.Line -match "//.*$pattern" }
    
    if ($matches) {
        $todoFiles += $matches
    }
}

if ($todoFiles) {
    $groupedTodos = $todoFiles | Group-Object Path
    foreach ($file in $groupedTodos) {
        Write-Warning-Message "Found $($file.Count) TODO/FIXME items in: $($file.Name)"
        if ($Strict) {
            $file.Group | ForEach-Object { 
                Write-Host "  Line $($_.LineNumber): $($_.Line.Trim())" -ForegroundColor DarkYellow 
            }
        }
    }
} else {
    Write-Success-Message "No TODO/FIXME/HACK comments found"
}

# 3. Check for hardcoded values that should be in config
Write-Info-Message "Checking for potential hardcoded values..."
$hardcodedPatterns = @(
    'https://[^"]*\.(com|org|dev|io)',
    '"[0-9]+\.[0-9]+\.[0-9]+"',
    'localhost',
    '127\.0\.0\.1'
)

$hardcodedFiles = @()
foreach ($pattern in $hardcodedPatterns) {
    $matches = Get-ChildItem -Path $Path -Recurse -Include "*.rs" |
        Select-String -Pattern $pattern |
        Where-Object { 
            $_.Line -notmatch "//.*$pattern" -and 
            $_.Line -notmatch "test" -and
            $_.Line -notmatch "example" -and
            $_.FullName -notmatch "\\tests\\"
        }
    
    if ($matches) {
        $hardcodedFiles += $matches
    }
}

if ($hardcodedFiles) {
    $groupedHardcoded = $hardcodedFiles | Group-Object Path
    foreach ($file in $groupedHardcoded) {
        Write-Warning-Message "Found $($file.Count) potential hardcoded values in: $($file.Name)"
        if ($Strict) {
            $file.Group | ForEach-Object { 
                Write-Host "  Line $($_.LineNumber): $($_.Line.Trim())" -ForegroundColor DarkYellow 
            }
        }
    }
} else {
    Write-Success-Message "No obvious hardcoded values found"
}

# 4. Check for large functions (>100 lines)
Write-Info-Message "Checking for large functions..."
$largeFiles = Get-ChildItem -Path $Path -Recurse -Include "*.rs" |
    ForEach-Object {
        $content = Get-Content $_.FullName
        $inFunction = $false
        $functionStart = 0
        $braceCount = 0
        $functionName = ""
        
        for ($i = 0; $i -lt $content.Length; $i++) {
            $line = $content[$i]
            
            # Detect function start
            if ($line -match "^\s*(pub\s+)?(async\s+)?fn\s+(\w+)") {
                $functionName = $matches[3]
                $functionStart = $i + 1
                $inFunction = $true
                $braceCount = 0
            }
            
            if ($inFunction) {
                # Count braces
                $braceCount += ($line.ToCharArray() | Where-Object { $_ -eq '{' }).Count
                $braceCount -= ($line.ToCharArray() | Where-Object { $_ -eq '}' }).Count
                
                # Function ended
                if ($braceCount -eq 0 -and $line -match '}') {
                    $functionLength = $i - $functionStart + 1
                    if ($functionLength -gt 100) {
                        [PSCustomObject]@{
                            File = $_.FullName
                            Function = $functionName
                            StartLine = $functionStart
                            Length = $functionLength
                        }
                    }
                    $inFunction = $false
                }
            }
        }
    }

if ($largeFiles) {
    foreach ($func in $largeFiles) {
        Write-Warning-Message "Large function '$($func.Function)' ($($func.Length) lines) in: $($func.File)"
    }
} else {
    Write-Success-Message "No large functions found"
}

# 5. Run Clippy for additional checks
Write-Info-Message "Running Clippy checks..."
try {
    $clippyOutput = cargo clippy --all-targets --all-features -- -D warnings 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Success-Message "Clippy checks passed"
    } else {
        Write-Error-Message "Clippy found issues"
        if ($Strict) {
            Write-Host $clippyOutput -ForegroundColor DarkRed
        }
    }
} catch {
    Write-Error-Message "Failed to run Clippy: $($_.Exception.Message)"
}

# 6. Check formatting
Write-Info-Message "Checking code formatting..."
try {
    $fmtOutput = cargo fmt --all -- --check 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Success-Message "Code formatting is correct"
    } else {
        Write-Warning-Message "Code formatting issues found"
        if ($Fix) {
            Write-Info-Message "Fixing formatting..."
            cargo fmt --all
            Write-Success-Message "Code formatted"
        } else {
            Write-Info-Message "Run with -Fix to automatically format code"
        }
    }
} catch {
    Write-Error-Message "Failed to check formatting: $($_.Exception.Message)"
}

# 7. Check for duplicate dependencies
Write-Info-Message "Checking for duplicate dependencies..."
$cargoFiles = Get-ChildItem -Path $Path -Recurse -Name "Cargo.toml"
$allDeps = @{}

foreach ($cargoFile in $cargoFiles) {
    $content = Get-Content $cargoFile
    $inDeps = $false
    
    foreach ($line in $content) {
        if ($line -match '^\[dependencies\]') {
            $inDeps = $true
            continue
        }
        if ($line -match '^\[') {
            $inDeps = $false
            continue
        }
        
        if ($inDeps -and $line -match '^(\w+)\s*=') {
            $depName = $matches[1]
            if ($allDeps.ContainsKey($depName)) {
                $allDeps[$depName] += @($cargoFile)
            } else {
                $allDeps[$depName] = @($cargoFile)
            }
        }
    }
}

$duplicates = $allDeps.GetEnumerator() | Where-Object { $_.Value.Count -gt 1 }
if ($duplicates) {
    foreach ($dup in $duplicates) {
        Write-Warning-Message "Dependency '$($dup.Key)' found in multiple Cargo.toml files: $($dup.Value -join ', ')"
    }
} else {
    Write-Success-Message "No duplicate dependencies found"
}

# Summary
Write-Host "`n📊 Quality Check Summary" -ForegroundColor Cyan
Write-Host "========================" -ForegroundColor Cyan
Write-Host "Errors: $ErrorCount" -ForegroundColor $(if ($ErrorCount -gt 0) { "Red" } else { "Green" })
Write-Host "Warnings: $WarningCount" -ForegroundColor $(if ($WarningCount -gt 0) { "Yellow" } else { "Green" })

if ($ErrorCount -gt 0) {
    Write-Host "`n❌ Quality check failed! Please fix the errors above." -ForegroundColor Red
    exit 1
} elseif ($WarningCount -gt 0) {
    Write-Host "`n⚠️  Quality check passed with warnings. Consider addressing them." -ForegroundColor Yellow
    exit 0
} else {
    Write-Host "`n✅ All quality checks passed!" -ForegroundColor Green
    exit 0
}
