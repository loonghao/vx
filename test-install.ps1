# Test script for vx installation improvements
param(
    [string]$TestType = "all"
)

Write-Host "🧪 Testing vx installation improvements..." -ForegroundColor Cyan
Write-Host ""

function Test-RateLimitHandling {
    Write-Host "📊 Testing GitHub API rate limit handling..." -ForegroundColor Yellow
    
    # Test without token (should hit rate limit)
    Write-Host "  Testing without GitHub token (expecting rate limit)..."
    try {
        $env:VX_VERSION = $null
        $env:GITHUB_TOKEN = $null
        & .\install.ps1 -WhatIf 2>&1 | Out-Null
    }
    catch {
        if ($_.Exception.Message -like "*rate limit*") {
            Write-Host "  ✅ Rate limit correctly detected and handled" -ForegroundColor Green
        } else {
            Write-Host "  ❌ Unexpected error: $($_.Exception.Message)" -ForegroundColor Red
        }
    }
}

function Test-VersionSpecification {
    Write-Host "📦 Testing version specification..." -ForegroundColor Yellow
    
    # Test with specific version (should bypass API call)
    Write-Host "  Testing with specific version (should bypass API)..."
    try {
        $env:VX_VERSION = "0.1.0"
        $env:GITHUB_TOKEN = $null
        
        # This should not hit the API since version is specified
        $output = & .\install.ps1 -WhatIf 2>&1
        if ($output -like "*Installing vx v0.1.0*") {
            Write-Host "  ✅ Version specification works correctly" -ForegroundColor Green
        } else {
            Write-Host "  ❌ Version specification failed" -ForegroundColor Red
        }
    }
    catch {
        Write-Host "  ❌ Error testing version specification: $($_.Exception.Message)" -ForegroundColor Red
    }
}

function Test-ScriptSyntax {
    Write-Host "📝 Testing script syntax..." -ForegroundColor Yellow
    
    # Test PowerShell script syntax
    Write-Host "  Checking PowerShell script syntax..."
    try {
        $null = [System.Management.Automation.PSParser]::Tokenize((Get-Content .\install.ps1 -Raw), [ref]$null)
        Write-Host "  ✅ PowerShell script syntax is valid" -ForegroundColor Green
    }
    catch {
        Write-Host "  ❌ PowerShell script syntax error: $($_.Exception.Message)" -ForegroundColor Red
    }
    
    # Test Bash script syntax (if available)
    if (Get-Command bash -ErrorAction SilentlyContinue) {
        Write-Host "  Checking Bash script syntax..."
        try {
            $result = bash -n .\install.sh 2>&1
            if ($LASTEXITCODE -eq 0) {
                Write-Host "  ✅ Bash script syntax is valid" -ForegroundColor Green
            } else {
                Write-Host "  ❌ Bash script syntax error: $result" -ForegroundColor Red
            }
        }
        catch {
            Write-Host "  ❌ Error checking Bash syntax: $($_.Exception.Message)" -ForegroundColor Red
        }
    } else {
        Write-Host "  ⚠️ Bash not available, skipping Bash syntax check" -ForegroundColor Yellow
    }
}

function Test-DistributionConfig {
    Write-Host "⚙️ Testing distribution configuration..." -ForegroundColor Yellow
    
    # Check if distribution.toml exists and is valid
    if (Test-Path .\distribution.toml) {
        Write-Host "  ✅ Distribution configuration file exists" -ForegroundColor Green
        
        # Basic TOML syntax check
        try {
            $content = Get-Content .\distribution.toml -Raw
            if ($content -match '\[distribution\]' -and $content -match '\[channels\.github\]') {
                Write-Host "  ✅ Distribution configuration has required sections" -ForegroundColor Green
            } else {
                Write-Host "  ❌ Distribution configuration missing required sections" -ForegroundColor Red
            }
        }
        catch {
            Write-Host "  ❌ Error reading distribution config: $($_.Exception.Message)" -ForegroundColor Red
        }
    } else {
        Write-Host "  ❌ Distribution configuration file not found" -ForegroundColor Red
    }
}

function Test-Documentation {
    Write-Host "📚 Testing documentation updates..." -ForegroundColor Yellow
    
    # Check if README has been updated with new installation methods
    if (Test-Path .\README.md) {
        $readme = Get-Content .\README.md -Raw
        
        $checks = @(
            @{ Pattern = "GITHUB_TOKEN"; Description = "GitHub token documentation" },
            @{ Pattern = "install-smart\.sh"; Description = "Smart installer documentation" },
            @{ Pattern = "Multi-Channel Distribution"; Description = "Multi-channel distribution section" },
            @{ Pattern = "Troubleshooting"; Description = "Troubleshooting section" }
        )
        
        foreach ($check in $checks) {
            if ($readme -match $check.Pattern) {
                Write-Host "  ✅ $($check.Description) found in README" -ForegroundColor Green
            } else {
                Write-Host "  ❌ $($check.Description) missing from README" -ForegroundColor Red
            }
        }
    } else {
        Write-Host "  ❌ README.md not found" -ForegroundColor Red
    }
}

# Run tests based on parameter
switch ($TestType) {
    "rate-limit" { Test-RateLimitHandling }
    "version" { Test-VersionSpecification }
    "syntax" { Test-ScriptSyntax }
    "config" { Test-DistributionConfig }
    "docs" { Test-Documentation }
    "all" {
        Test-ScriptSyntax
        Test-DistributionConfig
        Test-Documentation
        Test-VersionSpecification
        Test-RateLimitHandling
    }
    default {
        Write-Host "❌ Unknown test type: $TestType" -ForegroundColor Red
        Write-Host "Available tests: rate-limit, version, syntax, config, docs, all" -ForegroundColor Gray
    }
}

Write-Host ""
Write-Host "🎉 Testing completed!" -ForegroundColor Cyan
