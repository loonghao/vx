$ErrorActionPreference = "Stop"

$scripts = @(
  "node-pty.build.ps1",
  "fmt.build.ps1",
  "dotnet-samples-csc.build.ps1"
)

$results = @()
$failed  = @()

foreach ($script in $scripts) {
  $path = Join-Path $PSScriptRoot $script

  # GitHub Actions group annotation (no-op outside GHA)
  if ($env:GITHUB_ACTIONS -eq "true") {
    Write-Host "::group::$script"
  } else {
    Write-Host "`n==== Running $script ====" -ForegroundColor Cyan
  }

  $sw = [System.Diagnostics.Stopwatch]::StartNew()
  $status = "PASS"
  $errMsg = ""

  try {
    & powershell -NoProfile -ExecutionPolicy Bypass -File $path
    if ($LASTEXITCODE -ne 0) {
      throw "Child script exited with code $LASTEXITCODE"
    }
    Write-Host "PASS $script" -ForegroundColor Green
  }
  catch {
    $status = "FAIL"
    $errMsg = "$_"
    Write-Host "FAIL $script : $_" -ForegroundColor Red
    $failed += $script
  }
  finally {
    $sw.Stop()
  }

  if ($env:GITHUB_ACTIONS -eq "true") {
    Write-Host "::endgroup::"
  }

  $results += [PSCustomObject]@{
    Script   = $script
    Status   = $status
    Duration = "$([math]::Round($sw.Elapsed.TotalSeconds, 1))s"
    Error    = $errMsg
  }
}

# ── Summary ──────────────────────────────────────────────────────────────────
Write-Host "`n======== MSVC Examples Summary ========" -ForegroundColor Cyan
$results | Format-Table -AutoSize

# GitHub Actions step summary
if ($env:GITHUB_ACTIONS -eq "true" -and $env:GITHUB_STEP_SUMMARY) {
  $md  = "## MSVC Examples Results`n`n"
  $md += "| Script | Status | Duration |`n"
  $md += "|--------|--------|----------|`n"
  foreach ($r in $results) {
    $icon = if ($r.Status -eq "PASS") { "✅" } else { "❌" }
    $md += "| $($r.Script) | $icon $($r.Status) | $($r.Duration) |`n"
  }
  if ($failed.Count -gt 0) {
    $md += "`n### Failed Scripts`n"
    $failed | ForEach-Object { $md += "- ``$_```n" }
  }
  $md | Out-File -FilePath $env:GITHUB_STEP_SUMMARY -Encoding utf8 -Append
}

if ($failed.Count -gt 0) {
  Write-Host "`nFailed scripts:" -ForegroundColor Red
  $failed | ForEach-Object { Write-Host "  - $_" -ForegroundColor Red }
  exit 1
}

Write-Host "`nAll MSVC example scripts passed." -ForegroundColor Green
