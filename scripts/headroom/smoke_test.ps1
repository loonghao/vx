#!/usr/bin/env pwsh
<#
.SYNOPSIS
    headroom MCP smoke test (PIP-584 Phase 1).

.DESCRIPTION
    Validates that the headroom MCP server is reachable and the three
    core tools (headroom_compress, headroom_retrieve, headroom_stats)
    work correctly.

    Requires: vx (with mcpcall installed) or mcpcall directly.

.PARAMETER McpUrl
    MCP server URL (default: http://127.0.0.1:8765/mcp).

.PARAMETER SampleFile
    Path to a sample file for compress/retrieve testing (optional).

.PARAMETER Json
    Output results as JSON.

.EXAMPLE
    ./smoke_test.ps1
    ./smoke_test.ps1 -McpUrl http://localhost:8765/mcp -Json
#>

param(
    [string]$McpUrl = "http://127.0.0.1:8765/mcp",
    [string]$SampleFile = "",
    [switch]$Json
)

$ErrorActionPreference = "Stop"

# Determine how to invoke mcpcall
function Invoke-Mcpcall {
    param([string[]]$Args)
    $vx = Get-Command "vx" -ErrorAction SilentlyContinue
    if ($vx) {
        & $vx.Source mcpcall @Args
    }
    else {
        & "mcpcall" @Args
    }
}

$results = @{
    url          = $McpUrl
    tools_found  = @()
    compress_ok  = $false
    retrieve_ok  = $false
    stats_ok     = $false
    roundtrip_ok = $false
}

if (-not $Json) {
    Write-Host "=== headroom MCP smoke test ===" -ForegroundColor Cyan
    Write-Host "MCP URL: $McpUrl"
    Write-Host ""
}

# Step 1: List tools
if (-not $Json) {
    Write-Host "--- Listing MCP tools ---" -ForegroundColor Yellow
}

try {
    $listOut = Invoke-Mcpcall @("--url", $McpUrl, "list") 2>&1
    $listText = "$listOut"
    $toolsFound = @()

    if ($listText -match "compress|headroom_compress") { $toolsFound += "headroom_compress" }
    if ($listText -match "retrieve|headroom_retrieve")  { $toolsFound += "headroom_retrieve" }
    if ($listText -match "stats|headroom_stats")        { $toolsFound += "headroom_stats" }

    $results.tools_found = $toolsFound

    if (-not $Json) {
        if ($toolsFound.Count -gt 0) {
            Write-Host "Found MCP tools: $($toolsFound -join ', ')" -ForegroundColor Green
        }
        else {
            Write-Host "No expected MCP tools found" -ForegroundColor Red
            Write-Host "Raw output: $listText"
        }
    }
}
catch {
    if (-not $Json) {
        Write-Host "mcpcall list failed: $_" -ForegroundColor Red
    }
}

# Step 2: Read sample content
$sampleContent = if ($SampleFile -and (Test-Path $SampleFile)) {
    Get-Content $SampleFile -Raw
}
else {
    "Hello, headroom MCP smoke test! This is sample content for round-trip testing."
}

# Step 3: Test compress
if (-not $Json) {
    Write-Host ""
    Write-Host "--- Testing headroom_compress ---" -ForegroundColor Yellow
}

try {
    $compressOut = Invoke-Mcpcall @("--url", $McpUrl, "call", "headroom_compress", "--args", "{`"content`": `"$sampleContent`"}") 2>&1
    $compressText = "$compressOut"
    $hash = ($compressText -split "`n")[-1].Trim()

    if (-not [string]::IsNullOrEmpty($hash)) {
        $results.compress_ok = $true
        if (-not $Json) {
            Write-Host "compress result: $hash" -ForegroundColor Green
        }
    }
    else {
        if (-not $Json) {
            Write-Host "compress returned empty" -ForegroundColor Red
        }
    }
}
catch {
    if (-not $Json) {
        Write-Host "compress failed: $_" -ForegroundColor Red
    }
}

# Step 4: Test retrieve
if (-not $Json) {
    Write-Host ""
    Write-Host "--- Testing headroom_retrieve ---" -ForegroundColor Yellow
}

if ($results.compress_ok -and $hash) {
    try {
        $retrieveOut = Invoke-Mcpcall @("--url", $McpUrl, "call", "headroom_retrieve", "--args", "{`"hash`": `"$hash`"}") 2>&1
        $retrieveText = "$retrieveOut"

        if ($retrieveText.Contains($sampleContent)) {
            $results.retrieve_ok = $true
            $results.roundtrip_ok = $true
            if (-not $Json) {
                Write-Host "retrieve: content matches original" -ForegroundColor Green
            }
        }
        else {
            if (-not $Json) {
                Write-Host "retrieve result (content mismatch): $retrieveText" -ForegroundColor Yellow
            }
        }
    }
    catch {
        if (-not $Json) {
            Write-Host "retrieve failed: $_" -ForegroundColor Red
        }
    }
}
else {
    if (-not $Json) {
        Write-Host "Skipping retrieve: compress step failed or hash is empty" -ForegroundColor Red
    }
}

# Step 5: Test stats
if (-not $Json) {
    Write-Host ""
    Write-Host "--- Testing headroom_stats ---" -ForegroundColor Yellow
}

try {
    $statsOut = Invoke-Mcpcall @("--url", $McpUrl, "call", "headroom_stats", "--args", "{}") 2>&1
    $results.stats_ok = $true
    if (-not $Json) {
        Write-Host "stats: $statsOut" -ForegroundColor Green
    }
}
catch {
    if (-not $Json) {
        Write-Host "stats failed: $_" -ForegroundColor Red
    }
}

# Output
if ($Json) {
    Write-Output ($results | ConvertTo-Json)
}
else {
    Write-Host ""
    Write-Host "=== Summary ===" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  Tools found:     $($results.tools_found -join ', ')"
    Write-Host "  compress:        $(if ($results.compress_ok) { 'PASS' } else { 'FAIL' })"
    Write-Host "  retrieve:        $(if ($results.retrieve_ok) { 'PASS' } else { 'FAIL' })"
    Write-Host "  stats:           $(if ($results.stats_ok) { 'PASS' } else { 'FAIL' })"
    Write-Host "  round-trip:      $(if ($results.roundtrip_ok) { 'PASS' } else { 'FAIL' })"
}

# Exit code: 0 if all OK
if ($results.compress_ok -and $results.retrieve_ok -and $results.stats_ok) {
    exit 0
}
else {
    exit 1
}
