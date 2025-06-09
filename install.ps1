# vx Installation Script for Windows
# This script builds and installs vx to a local directory

Write-Host "🚀 Installing vx - Universal Version Executor" -ForegroundColor Green

# Check if Rust is installed
if (!(Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Rust is not installed. Please install Rust first:" -ForegroundColor Red
    Write-Host "   Visit: https://rustup.rs/" -ForegroundColor Yellow
    exit 1
}

# Build the project
Write-Host "🔨 Building vx..." -ForegroundColor Blue
cargo build --release

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Build failed!" -ForegroundColor Red
    exit 1
}

# Create installation directory
$installDir = "$env:USERPROFILE\.vx\bin"
if (!(Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}

# Copy the binary
Copy-Item "target\release\vx.exe" "$installDir\vx.exe" -Force

Write-Host "✅ vx installed to: $installDir" -ForegroundColor Green

# Check if directory is in PATH
$currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($currentPath -notlike "*$installDir*") {
    Write-Host "💡 Adding $installDir to your PATH..." -ForegroundColor Yellow
    $newPath = "$currentPath;$installDir"
    [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
    Write-Host "✅ PATH updated. Please restart your terminal." -ForegroundColor Green
} else {
    Write-Host "✅ Directory already in PATH" -ForegroundColor Green
}

Write-Host ""
Write-Host "🎉 Installation complete!" -ForegroundColor Green
Write-Host "📖 Try these commands:" -ForegroundColor Blue
Write-Host "   vx --help" -ForegroundColor Gray
Write-Host "   vx list" -ForegroundColor Gray
Write-Host "   vx npm --version" -ForegroundColor Gray
Write-Host "   vx uv --version" -ForegroundColor Gray
