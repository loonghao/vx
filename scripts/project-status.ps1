# VX Project Status Checker
# This script analyzes the current state of the VX project and provides actionable insights

param(
    [switch]$Detailed = $false,
    [switch]$TodoOnly = $false,
    [switch]$Export = $false
)

Write-Host "🔍 VX Project Status Analysis" -ForegroundColor Cyan
Write-Host "=============================" -ForegroundColor Cyan

$ProjectRoot = Get-Location
$Results = @{
    TotalCrates = 0
    CompletedFeatures = 0
    PendingTodos = 0
    TestCoverage = 0
    QualityScore = 0
    CriticalIssues = @()
    Recommendations = @()
}

# 1. Analyze Cargo workspace
Write-Host "📦 Analyzing Cargo workspace..." -ForegroundColor Blue

if (Test-Path "Cargo.toml") {
    $CargoContent = Get-Content "Cargo.toml" -Raw
    $Members = ($CargoContent | Select-String -Pattern 'members\s*=\s*\[(.*?)\]' -AllMatches).Matches
    if ($Members) {
        $MemberList = $Members[0].Groups[1].Value -split ',' | ForEach-Object { $_.Trim().Trim('"') }
        $Results.TotalCrates = $MemberList.Count
        Write-Host "  ✅ Found $($Results.TotalCrates) crates" -ForegroundColor Green
        
        if ($Detailed) {
            Write-Host "  📋 Crates:" -ForegroundColor Yellow
            $MemberList | ForEach-Object { Write-Host "    - $_" -ForegroundColor Gray }
        }
    }
}

# 2. Scan for TODO/FIXME items
Write-Host "📝 Scanning for TODO/FIXME items..." -ForegroundColor Blue

$TodoPatterns = @("TODO", "FIXME", "HACK", "XXX", "BUG")
$TodoFiles = @()

foreach ($Pattern in $TodoPatterns) {
    $Matches = Get-ChildItem -Path "." -Recurse -Include "*.rs" | 
        Select-String -Pattern $Pattern | 
        Where-Object { $_.Line -notmatch "^//" -or $_.Line -match "//.*$Pattern" }
    
    $TodoFiles += $Matches
}

$Results.PendingTodos = $TodoFiles.Count

if ($Results.PendingTodos -gt 0) {
    Write-Host "  ⚠️  Found $($Results.PendingTodos) TODO/FIXME items" -ForegroundColor Yellow
    
    if ($TodoOnly -or $Detailed) {
        $GroupedTodos = $TodoFiles | Group-Object Path
        foreach ($Group in $GroupedTodos) {
            Write-Host "    📄 $($Group.Name):" -ForegroundColor Cyan
            $Group.Group | ForEach-Object { 
                Write-Host "      Line $($_.LineNumber): $($_.Line.Trim())" -ForegroundColor Gray 
            }
        }
    }
} else {
    Write-Host "  ✅ No TODO/FIXME items found" -ForegroundColor Green
}

# 3. Check command implementation status
Write-Host "🛠️  Checking command implementation..." -ForegroundColor Blue

$CommandsDir = "crates/vx-cli/src/commands"
if (Test-Path $CommandsDir) {
    $CommandFiles = Get-ChildItem -Path $CommandsDir -Filter "*.rs" | Where-Object { $_.Name -ne "mod.rs" }
    $TotalCommands = $CommandFiles.Count
    
    $IncompleteCommands = @()
    foreach ($File in $CommandFiles) {
        $Content = Get-Content $File.FullName -Raw
        if ($Content -match "TODO|FIXME|not yet implemented|unimplemented") {
            $IncompleteCommands += $File.BaseName
        }
    }
    
    $CompletedCommands = $TotalCommands - $IncompleteCommands.Count
    $Results.CompletedFeatures = [math]::Round(($CompletedCommands / $TotalCommands) * 100, 1)
    
    Write-Host "  📊 Command Status: $CompletedCommands/$TotalCommands completed ($($Results.CompletedFeatures)%)" -ForegroundColor Green
    
    if ($IncompleteCommands.Count -gt 0) {
        Write-Host "  ⚠️  Incomplete commands:" -ForegroundColor Yellow
        $IncompleteCommands | ForEach-Object { Write-Host "    - $_" -ForegroundColor Gray }
        $Results.CriticalIssues += "Incomplete commands: $($IncompleteCommands -join ', ')"
    }
}

# 4. Analyze test coverage
Write-Host "🧪 Analyzing test coverage..." -ForegroundColor Blue

$TestFiles = Get-ChildItem -Path "." -Recurse -Include "*test*.rs", "*tests*" -File
$SourceFiles = Get-ChildItem -Path "crates" -Recurse -Include "*.rs" -File | 
    Where-Object { $_.FullName -notmatch "test" -and $_.FullName -notmatch "target" }

if ($TestFiles.Count -gt 0 -and $SourceFiles.Count -gt 0) {
    $TestRatio = [math]::Round(($TestFiles.Count / $SourceFiles.Count) * 100, 1)
    $Results.TestCoverage = $TestRatio
    
    if ($TestRatio -ge 60) {
        Write-Host "  ✅ Test coverage: $TestRatio% (Good)" -ForegroundColor Green
    } elseif ($TestRatio -ge 40) {
        Write-Host "  ⚠️  Test coverage: $TestRatio% (Needs improvement)" -ForegroundColor Yellow
        $Results.Recommendations += "Increase test coverage to at least 60%"
    } else {
        Write-Host "  ❌ Test coverage: $TestRatio% (Poor)" -ForegroundColor Red
        $Results.CriticalIssues += "Low test coverage: $TestRatio%"
    }
} else {
    Write-Host "  ⚠️  Unable to calculate test coverage" -ForegroundColor Yellow
}

# 5. Check for code quality issues
Write-Host "🔍 Checking code quality..." -ForegroundColor Blue

$QualityIssues = 0

# Check for unwrap() in production code
$UnwrapFiles = Get-ChildItem -Path "crates" -Recurse -Include "*.rs" | 
    Where-Object { $_.FullName -notmatch "test" } |
    Select-String -Pattern "\.unwrap\(\)" |
    Group-Object Path

if ($UnwrapFiles) {
    $QualityIssues += $UnwrapFiles.Count
    Write-Host "  ⚠️  Found unwrap() calls in $($UnwrapFiles.Count) production files" -ForegroundColor Yellow
    $Results.CriticalIssues += "unwrap() calls in production code"
}

# Check for large functions (simplified check)
$LargeFiles = Get-ChildItem -Path "crates" -Recurse -Include "*.rs" | 
    Where-Object { (Get-Content $_.FullName).Count -gt 500 }

if ($LargeFiles) {
    $QualityIssues += $LargeFiles.Count
    Write-Host "  ⚠️  Found $($LargeFiles.Count) large files (>500 lines)" -ForegroundColor Yellow
    $Results.Recommendations += "Consider breaking down large files"
}

$Results.QualityScore = [math]::Max(0, 100 - ($QualityIssues * 10))
Write-Host "  📊 Code quality score: $($Results.QualityScore)/100" -ForegroundColor $(if ($Results.QualityScore -ge 80) { "Green" } elseif ($Results.QualityScore -ge 60) { "Yellow" } else { "Red" })

# 6. Generate recommendations
Write-Host "`n💡 Recommendations:" -ForegroundColor Cyan

if ($Results.PendingTodos -gt 10) {
    $Results.Recommendations += "High number of TODO items ($($Results.PendingTodos)) - consider prioritizing cleanup"
}

if ($Results.CompletedFeatures -lt 90) {
    $Results.Recommendations += "Complete remaining command implementations"
}

if ($Results.Recommendations.Count -eq 0) {
    Write-Host "  ✅ Project is in good shape!" -ForegroundColor Green
} else {
    $Results.Recommendations | ForEach-Object { Write-Host "  • $_" -ForegroundColor Yellow }
}

# 7. Show critical issues
if ($Results.CriticalIssues.Count -gt 0) {
    Write-Host "`n🚨 Critical Issues:" -ForegroundColor Red
    $Results.CriticalIssues | ForEach-Object { Write-Host "  • $_" -ForegroundColor Red }
}

# 8. Overall project health
Write-Host "`n📊 Project Health Summary:" -ForegroundColor Cyan
Write-Host "  Crates: $($Results.TotalCrates)" -ForegroundColor White
Write-Host "  Feature Completion: $($Results.CompletedFeatures)%" -ForegroundColor White
Write-Host "  Test Coverage: $($Results.TestCoverage)%" -ForegroundColor White
Write-Host "  Quality Score: $($Results.QualityScore)/100" -ForegroundColor White
Write-Host "  Pending TODOs: $($Results.PendingTodos)" -ForegroundColor White

$OverallHealth = [math]::Round(($Results.CompletedFeatures + $Results.TestCoverage + $Results.QualityScore) / 3, 1)
$HealthColor = if ($OverallHealth -ge 80) { "Green" } elseif ($OverallHealth -ge 60) { "Yellow" } else { "Red" }
$HealthStatus = if ($OverallHealth -ge 80) { "Excellent" } elseif ($OverallHealth -ge 60) { "Good" } else { "Needs Attention" }

Write-Host "`n🎯 Overall Health: $OverallHealth% ($HealthStatus)" -ForegroundColor $HealthColor

# 9. Export results if requested
if ($Export) {
    $ExportData = @{
        Timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
        ProjectHealth = $OverallHealth
        Results = $Results
    }
    
    $ExportPath = "project-status-$(Get-Date -Format 'yyyyMMdd-HHmmss').json"
    $ExportData | ConvertTo-Json -Depth 3 | Out-File -FilePath $ExportPath -Encoding UTF8
    Write-Host "`n💾 Results exported to: $ExportPath" -ForegroundColor Green
}

Write-Host "`n🚀 Next Steps:" -ForegroundColor Cyan
Write-Host "  1. Review PROJECT_STATUS.md for detailed analysis" -ForegroundColor White
Write-Host "  2. Address critical issues first" -ForegroundColor White
Write-Host "  3. Run this script weekly to track progress" -ForegroundColor White
Write-Host "  4. Use --Detailed flag for more information" -ForegroundColor White
