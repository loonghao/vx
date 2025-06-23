# VX Benchmark Management Tool (PowerShell)
param(
    [Parameter(Position=0)]
    [ValidateSet("list", "summary", "compare", "set-baseline", "report")]
    [string]$Command,
    
    [string]$File,
    [string]$Output
)

$BenchmarksDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ResultsDir = Join-Path $BenchmarksDir "results"
$BaselineFile = Join-Path $BenchmarksDir "baseline.json"

function Get-BenchmarkResults {
    if (Test-Path $ResultsDir) {
        Get-ChildItem -Path $ResultsDir -Filter "benchmark_results_*.json" | Sort-Object Name
    } else {
        @()
    }
}

function Read-JsonFile {
    param([string]$Path)
    
    if (Test-Path $Path) {
        try {
            $content = Get-Content -Path $Path -Raw -Encoding UTF8
            return $content | ConvertFrom-Json
        } catch {
            Write-Error "Error reading $Path`: $_"
            return $null
        }
    }
    return $null
}

function Write-JsonFile {
    param([string]$Path, [object]$Data)
    
    try {
        $Data | ConvertTo-Json -Depth 10 | Out-File -FilePath $Path -Encoding UTF8
        return $true
    } catch {
        Write-Error "Error writing $Path`: $_"
        return $false
    }
}

function Get-BenchmarkSummary {
    param([string]$FilePath)
    
    if (-not $FilePath) {
        $files = Get-BenchmarkResults
        if ($files.Count -eq 0) {
            return @{ error = "No benchmark results found" }
        }
        $FilePath = $files[-1].FullName
    }
    
    $results = Read-JsonFile -Path $FilePath
    if (-not $results) {
        return @{ error = "No data in $FilePath" }
    }
    
    $summary = @{
        file = Split-Path -Leaf $FilePath
        timestamp = if ($results.Count -gt 0) { $results[0].timestamp } else { $null }
        total_operations = $results.Count
        operations = @{}
        tools = @{}
    }
    
    foreach ($result in $results) {
        $opType = if ($result.operation) { $result.operation } else { "unknown" }
        $tool = if ($result.tool) { $result.tool } else { "unknown" }
        $duration = if ($result.duration_ms) { $result.duration_ms } else { 0 }
        $success = if ($result.success) { $result.success } else { $false }
        
        # Operation statistics
        if (-not $summary.operations.ContainsKey($opType)) {
            $summary.operations[$opType] = @{
                count = 0
                success_count = 0
                total_duration = 0
                durations = @()
            }
        }
        
        $summary.operations[$opType].count++
        $summary.operations[$opType].total_duration += $duration
        $summary.operations[$opType].durations += $duration
        if ($success) {
            $summary.operations[$opType].success_count++
        }
        
        # Tool statistics
        if (-not $summary.tools.ContainsKey($tool)) {
            $summary.tools[$tool] = @{
                total_duration = 0
                success_count = 0
                total_count = 0
            }
        }
        
        $summary.tools[$tool].total_duration += $duration
        $summary.tools[$tool].total_count++
        if ($success) {
            $summary.tools[$tool].success_count++
        }
    }
    
    # Calculate averages
    foreach ($opType in $summary.operations.Keys) {
        $data = $summary.operations[$opType]
        if ($data.durations.Count -gt 0) {
            $data.avg_duration = ($data.durations | Measure-Object -Average).Average
            $data.min_duration = ($data.durations | Measure-Object -Minimum).Minimum
            $data.max_duration = ($data.durations | Measure-Object -Maximum).Maximum
        }
        $data.success_rate = if ($data.count -gt 0) { $data.success_count / $data.count } else { 0 }
        $data.Remove("durations")  # Remove raw data
    }
    
    return $summary
}

function Set-Baseline {
    param([string]$FilePath)
    
    $summary = Get-BenchmarkSummary -FilePath $FilePath
    if ($summary.error) {
        Write-Error $summary.error
        return $false
    }
    
    $baselineData = @{
        created_at = (Get-Date).ToString("o")
        source_file = $summary.file
        operations = $summary.operations
        tools = $summary.tools
    }
    
    if (Write-JsonFile -Path $BaselineFile -Data $baselineData) {
        Write-Host "✅ Baseline set from $($summary.file)" -ForegroundColor Green
        return $true
    }
    return $false
}

function Compare-WithBaseline {
    param([string]$FilePath)
    
    $baseline = Read-JsonFile -Path $BaselineFile
    if (-not $baseline) {
        return @{ error = "No baseline found. Run 'set-baseline' first." }
    }
    
    $currentSummary = Get-BenchmarkSummary -FilePath $FilePath
    if ($currentSummary.error) {
        return $currentSummary
    }
    
    $comparison = @{
        baseline_file = if ($baseline.source_file) { $baseline.source_file } else { "unknown" }
        current_file = $currentSummary.file
        improvements = @()
        regressions = @()
    }
    
    $baselineOps = if ($baseline.operations) { $baseline.operations } else { @{} }
    $currentOps = $currentSummary.operations
    
    foreach ($opType in $currentOps.Keys) {
        if ($baselineOps.ContainsKey($opType)) {
            $baselineAvg = $baselineOps[$opType].avg_duration
            $currentAvg = $currentOps[$opType].avg_duration
            
            if ($baselineAvg -gt 0) {
                $changePercent = (($currentAvg - $baselineAvg) / $baselineAvg) * 100
                
                $changeData = @{
                    operation = $opType
                    baseline_avg = $baselineAvg
                    current_avg = $currentAvg
                    change_percent = $changePercent
                    change_ms = $currentAvg - $baselineAvg
                }
                
                if ($changePercent -lt -5) {
                    $comparison.improvements += $changeData
                } elseif ($changePercent -gt 5) {
                    $comparison.regressions += $changeData
                }
            }
        }
    }
    
    return $comparison
}

function Show-Report {
    param([string]$OutputFile)
    
    $files = Get-BenchmarkResults
    if ($files.Count -eq 0) {
        Write-Host "No benchmark results found." -ForegroundColor Yellow
        return
    }
    
    $latestFile = $files[-1].FullName
    $summary = Get-BenchmarkSummary -FilePath $latestFile
    $comparison = Compare-WithBaseline -FilePath $latestFile
    
    $report = @"
# VX Performance Report
Generated: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')
Latest Results: $($summary.file)

## Summary
- Total Operations: $($summary.total_operations)
- Timestamp: $($summary.timestamp)

## Operations Performance
| Operation | Count | Success Rate | Avg Duration (ms) |
|-----------|-------|--------------|-------------------|
"@
    
    foreach ($opType in $summary.operations.Keys) {
        $data = $summary.operations[$opType]
        $successRate = "{0:P1}" -f $data.success_rate
        $avgDur = "{0:F1}" -f $data.avg_duration
        $report += "`n| $opType | $($data.count) | $successRate | $avgDur |"
    }
    
    $report += "`n`n## Tools Performance`n"
    $report += "| Tool | Operations | Success Rate | Total Duration (ms) |`n"
    $report += "|------|------------|--------------|---------------------|`n"
    
    foreach ($tool in $summary.tools.Keys) {
        $data = $summary.tools[$tool]
        $successRate = if ($data.total_count -gt 0) { "{0:P1}" -f ($data.success_count / $data.total_count) } else { "0%" }
        $report += "| $tool | $($data.total_count) | $successRate | $($data.total_duration) |`n"
    }
    
    if (-not $comparison.error) {
        $report += "`n## Comparison with Baseline`n"
        $report += "Baseline: $($comparison.baseline_file)`n`n"
        
        if ($comparison.improvements.Count -gt 0) {
            $report += "### 🚀 Improvements`n"
            $report += "| Operation | Baseline (ms) | Current (ms) | Improvement |`n"
            $report += "|-----------|---------------|--------------|-------------|`n"
            foreach ($imp in $comparison.improvements) {
                $change = "{0:F1}% ({1:F1}ms faster)" -f [Math]::Abs($imp.change_percent), [Math]::Abs($imp.change_ms)
                $report += "| $($imp.operation) | $($imp.baseline_avg) | $($imp.current_avg) | $change |`n"
            }
            $report += "`n"
        }
        
        if ($comparison.regressions.Count -gt 0) {
            $report += "### ⚠️ Regressions`n"
            $report += "| Operation | Baseline (ms) | Current (ms) | Regression |`n"
            $report += "|-----------|---------------|--------------|------------|`n"
            foreach ($reg in $comparison.regressions) {
                $change = "{0:F1}% ({1:F1}ms slower)" -f $reg.change_percent, $reg.change_ms
                $report += "| $($reg.operation) | $($reg.baseline_avg) | $($reg.current_avg) | $change |`n"
            }
        }
    }
    
    if ($OutputFile) {
        $report | Out-File -FilePath $OutputFile -Encoding UTF8
        Write-Host "📊 Report saved to $OutputFile" -ForegroundColor Green
    } else {
        Write-Host $report
    }
}

# Main command handling
switch ($Command) {
    "list" {
        $files = Get-BenchmarkResults
        if ($files.Count -gt 0) {
            Write-Host "📊 Available benchmark results:" -ForegroundColor Cyan
            foreach ($file in $files) {
                Write-Host "  - $($file.Name)" -ForegroundColor White
            }
        } else {
            Write-Host "No benchmark results found." -ForegroundColor Yellow
        }
    }
    
    "summary" {
        $summary = Get-BenchmarkSummary -FilePath $File
        if ($summary.error) {
            Write-Error $summary.error
        } else {
            Write-Host "📊 Summary for $($summary.file):" -ForegroundColor Cyan
            Write-Host "  Total Operations: $($summary.total_operations)" -ForegroundColor White
            Write-Host "  Timestamp: $($summary.timestamp)" -ForegroundColor White
            Write-Host "`n  Operations:" -ForegroundColor White
            foreach ($opType in $summary.operations.Keys) {
                $data = $summary.operations[$opType]
                $successRate = "{0:P1}" -f $data.success_rate
                $avgDur = "{0:F1}" -f $data.avg_duration
                Write-Host "    $opType`: $($data.count) ops, $successRate success, ${avgDur}ms avg" -ForegroundColor Gray
            }
        }
    }
    
    "compare" {
        $comparison = Compare-WithBaseline -FilePath $File
        if ($comparison.error) {
            Write-Error $comparison.error
        } else {
            Write-Host "📊 Comparison: $($comparison.current_file) vs $($comparison.baseline_file)" -ForegroundColor Cyan
            if ($comparison.improvements.Count -gt 0) {
                Write-Host "  🚀 $($comparison.improvements.Count) improvements" -ForegroundColor Green
            }
            if ($comparison.regressions.Count -gt 0) {
                Write-Host "  ⚠️ $($comparison.regressions.Count) regressions" -ForegroundColor Yellow
            }
        }
    }
    
    "set-baseline" {
        Set-Baseline -FilePath $File
    }
    
    "report" {
        Show-Report -OutputFile $Output
    }
    
    default {
        Write-Host @"
VX Benchmark Management Tool

Usage: .\manage.ps1 <command> [options]

Commands:
  list                    List all benchmark results
  summary [-File <path>]  Generate summary of results
  compare [-File <path>]  Compare with baseline
  set-baseline [-File <path>]  Set baseline from results
  report [-Output <path>] Generate detailed report

Examples:
  .\manage.ps1 list
  .\manage.ps1 summary
  .\manage.ps1 set-baseline
  .\manage.ps1 report -Output performance_report.md
"@ -ForegroundColor White
    }
}