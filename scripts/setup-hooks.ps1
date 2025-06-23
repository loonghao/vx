# Setup Git Hooks for VX Project (Windows)
# This script installs and configures git hooks for code quality

param(
    [switch]$Force = $false
)

Write-Host "🔧 Setting up VX Git Hooks" -ForegroundColor Cyan
Write-Host "==========================" -ForegroundColor Cyan

function Write-Error-Message($message) {
    Write-Host "❌ $message" -ForegroundColor Red
}

function Write-Success-Message($message) {
    Write-Host "✅ $message" -ForegroundColor Green
}

function Write-Warning-Message($message) {
    Write-Host "⚠️  $message" -ForegroundColor Yellow
}

function Write-Info-Message($message) {
    Write-Host "ℹ️  $message" -ForegroundColor Blue
}

# Check if we're in a git repository
if (-not (Test-Path ".git")) {
    Write-Error-Message "Not in a git repository. Please run this script from the project root."
    exit 1
}

# Check if .githooks directory exists
if (-not (Test-Path ".githooks")) {
    Write-Error-Message ".githooks directory not found. Please run this script from the project root."
    exit 1
}

# Configure git to use .githooks directory
Write-Info-Message "Configuring git to use .githooks directory..."
git config core.hooksPath .githooks

# Verify hooks are set up correctly
if (Test-Path ".githooks/pre-commit") {
    Write-Success-Message "Pre-commit hook found"
} else {
    Write-Warning-Message "Pre-commit hook not found"
}

# Install required tools if not present
Write-Info-Message "Checking required tools..."

# Check for rustfmt
try {
    $null = rustfmt --version
    Write-Success-Message "rustfmt is available"
} catch {
    Write-Warning-Message "rustfmt not found. Installing..."
    rustup component add rustfmt
}

# Check for clippy
try {
    $null = cargo clippy --version
    Write-Success-Message "clippy is available"
} catch {
    Write-Warning-Message "clippy not found. Installing..."
    rustup component add clippy
}

# Create a sample git config for the project
Write-Info-Message "Setting up recommended git configuration..."

# Set up conventional commit template
$gitmessageContent = @"
# <type>(<scope>): <subject>
#
# <body>
#
# <footer>
#
# Type should be one of the following:
# * feat: A new feature
# * fix: A bug fix
# * docs: Documentation only changes
# * style: Changes that do not affect the meaning of the code
# * refactor: A code change that neither fixes a bug nor adds a feature
# * test: Adding missing tests or correcting existing tests
# * chore: Changes to the build process or auxiliary tools
# * ci: Changes to CI configuration files and scripts
# * build: Changes that affect the build system or external dependencies
#
# Scope is optional and should be the name of the package affected
# Subject should be imperative mood, lowercase, no period at the end
# Body should explain what and why vs. how (optional)
# Footer should contain any breaking changes or issue references (optional)
"@

$gitmessageContent | Out-File -FilePath ".gitmessage" -Encoding UTF8

git config commit.template .gitmessage

# Set up other useful git configurations
git config push.default simple
git config pull.rebase true

Write-Success-Message "Git configuration updated"

# Create a local configuration file for hook settings
$hookConfigContent = @"
# VX Git Hooks Configuration
# You can override these settings by setting environment variables

# Enable quick tests on affected modules (default: false)
# `$env:VX_QUICK_TEST = "true"

# Enable strict mode - fail on any warnings (default: false)
# `$env:VX_STRICT_MODE = "true"

# Enable automatic formatting (default: true)
# `$env:VX_AUTO_FIX = "true"

# To use these settings in PowerShell:
# `$env:VX_QUICK_TEST = "true"
# git commit -m "your message"

# Or create a PowerShell profile function:
# function Git-Commit-Strict { `$env:VX_STRICT_MODE = "true"; git commit @args }
# Set-Alias gcs Git-Commit-Strict
"@

$hookConfigContent | Out-File -FilePath ".git-hooks-config.ps1" -Encoding UTF8

Write-Info-Message "Created .git-hooks-config.ps1 for local customization"

# Create helper functions for PowerShell users
$helperFunctionsContent = @"
# VX Git Helper Functions for PowerShell
# Source this file in your PowerShell profile for convenient aliases

function Git-Commit-Quick-Test {
    `$env:VX_QUICK_TEST = "true"
    git commit @args
    Remove-Item env:VX_QUICK_TEST
}

function Git-Commit-Strict {
    `$env:VX_STRICT_MODE = "true"
    git commit @args
    Remove-Item env:VX_STRICT_MODE
}

function Git-Commit-No-Format {
    `$env:VX_AUTO_FIX = "false"
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
"@

$helperFunctionsContent | Out-File -FilePath "scripts/vx-git-helpers.ps1" -Encoding UTF8

Write-Info-Message "Created PowerShell helper functions in scripts/vx-git-helpers.ps1"

# Show usage information
Write-Host ""
Write-Success-Message "🎉 Git hooks setup completed!"
Write-Host ""
Write-Info-Message "📋 Usage:"
Write-Host "  Normal commit:     git commit -m 'feat: add new feature'"
Write-Host "  Skip hooks:        git commit --no-verify -m 'message'"
Write-Host "  With quick tests:  `$env:VX_QUICK_TEST='true'; git commit -m 'message'"
Write-Host "  Strict mode:       `$env:VX_STRICT_MODE='true'; git commit -m 'message'"
Write-Host "  Disable auto-fix:  `$env:VX_AUTO_FIX='false'; git commit -m 'message'"
Write-Host ""
Write-Info-Message "📝 Configuration:"
Write-Host "  Edit .git-hooks-config.ps1 to customize hook behavior"
Write-Host "  Source scripts/vx-git-helpers.ps1 for convenient aliases"
Write-Host ""
Write-Info-Message "🔧 Manual commands:"
Write-Host "  Format code:       cargo fmt --all"
Write-Host "  Run clippy:        cargo clippy --all-targets --all-features"
Write-Host "  Quality check:     .\scripts\quality-check.ps1"
Write-Host ""
Write-Info-Message "💡 PowerShell Integration:"
Write-Host "  Add to your PowerShell profile:"
Write-Host "  . `"`$PWD\scripts\vx-git-helpers.ps1`""
Write-Host ""
Write-Warning-Message "💡 Tips:"
Write-Host "  - Use conventional commit format for better changelog generation"
Write-Host "  - The pre-commit hook will automatically format your code"
Write-Host "  - Set VX_QUICK_TEST=true to run tests on affected modules"
Write-Host "  - Use --no-verify to skip hooks in emergency situations"

exit 0
