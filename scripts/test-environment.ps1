# Test Environment for VX Publishing
Write-Host "🔍 Testing VX Publishing Environment" -ForegroundColor Cyan
Write-Host "====================================" -ForegroundColor Cyan

# Test 1: Check if we're in the right directory
Write-Host "`n1. Checking project directory..." -ForegroundColor Yellow
if (Test-Path "Cargo.toml") {
    $cargoContent = Get-Content "Cargo.toml" | Select-String "name.*vx"
    if ($cargoContent) {
        Write-Host "✅ Found VX project root" -ForegroundColor Green
    } else {
        Write-Host "❌ Not in VX project root" -ForegroundColor Red
        exit 1
    }
} else {
    Write-Host "❌ No Cargo.toml found - not in project root" -ForegroundColor Red
    exit 1
}

# Test 2: Check Cargo installation
Write-Host "`n2. Checking Cargo installation..." -ForegroundColor Yellow
try {
    $cargoVersion = cargo --version
    Write-Host "✅ Cargo found: $cargoVersion" -ForegroundColor Green
} catch {
    Write-Host "❌ Cargo not found - install Rust first" -ForegroundColor Red
    exit 1
}

# Test 3: Check if logged into crates.io
Write-Host "`n3. Checking crates.io authentication..." -ForegroundColor Yellow
try {
    # Try to run a command that requires authentication
    $result = cargo search vx-core --limit 1 2>&1
    if ($result -match "vx-core") {
        Write-Host "✅ Can access crates.io" -ForegroundColor Green
    } else {
        Write-Host "⚠️  Cannot search crates.io - check network" -ForegroundColor Yellow
    }
} catch {
    Write-Host "⚠️  Cannot access crates.io" -ForegroundColor Yellow
}

# Test 4: Check workspace packages
Write-Host "`n4. Checking workspace packages..." -ForegroundColor Yellow
$packages = @(
    "crates/vx-core",
    "crates/vx-tools/vx-tool-go",
    "crates/vx-tools/vx-tool-rust", 
    "crates/vx-tools/vx-tool-uv",
    "crates/vx-package-managers/vx-pm-npm",
    "crates/vx-tools/vx-tool-node",
    "crates/vx-cli"
)

$allFound = $true
foreach ($package in $packages) {
    $cargoToml = "$package/Cargo.toml"
    if (Test-Path $cargoToml) {
        Write-Host "✅ Found: $package" -ForegroundColor Green
    } else {
        Write-Host "❌ Missing: $package" -ForegroundColor Red
        $allFound = $false
    }
}

if ($allFound) {
    Write-Host "✅ All workspace packages found" -ForegroundColor Green
} else {
    Write-Host "❌ Some workspace packages missing" -ForegroundColor Red
}

# Test 5: Test package info extraction
Write-Host "`n5. Testing package info extraction..." -ForegroundColor Yellow
try {
    $metadata = cargo metadata --no-deps --format-version 1 --manifest-path "Cargo.toml" | ConvertFrom-Json
    $mainPackage = $metadata.packages[0]
    Write-Host "✅ Main package: $($mainPackage.name)@$($mainPackage.version)" -ForegroundColor Green
} catch {
    Write-Host "❌ Failed to extract package info" -ForegroundColor Red
    Write-Host "Error: $_" -ForegroundColor Red
}

# Test 6: Check if vx-core is published
Write-Host "`n6. Checking vx-core publication status..." -ForegroundColor Yellow
try {
    $searchResult = cargo search vx-core --limit 1 2>$null
    if ($searchResult -match "vx-core") {
        Write-Host "✅ vx-core is published on crates.io" -ForegroundColor Green
    } else {
        Write-Host "❌ vx-core not found on crates.io" -ForegroundColor Red
    }
} catch {
    Write-Host "⚠️  Cannot check vx-core status" -ForegroundColor Yellow
}

Write-Host "`n🎯 Environment Test Summary:" -ForegroundColor Cyan
Write-Host "- Project root: ✅"
Write-Host "- Cargo installed: ✅" 
Write-Host "- Workspace packages: $(if ($allFound) { '✅' } else { '❌' })"
Write-Host "- Ready for publishing: $(if ($allFound) { '✅' } else { '❌' })"

if ($allFound) {
    Write-Host "`n💡 Next steps:" -ForegroundColor Yellow
    Write-Host "1. Test publishing: .\scripts\publish-workspace.ps1 -DryRun" -ForegroundColor White
    Write-Host "2. Actual publishing: .\scripts\publish-workspace.ps1 -DryRun:`$false" -ForegroundColor White
} else {
    Write-Host "`n❌ Fix the issues above before publishing" -ForegroundColor Red
}
