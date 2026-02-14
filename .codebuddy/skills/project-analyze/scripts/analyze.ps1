# Project Analyze Script for Windows
# Usage: .\analyze.ps1 <project-name-or-url> [temp-dir]

param(
    [Parameter(Mandatory=$true)]
    [string]$Project,

    [string]$TempDir = "c:/github"
)

$ErrorActionPreference = "Stop"

# Known project mappings
$KnownProjects = @{
    "codex" = "https://github.com/openai/codex"
    "kubectl" = "https://github.com/kubernetes/kubectl"
    "deno" = "https://github.com/denoland/deno"
    "ripgrep" = "https://github.com/BurntSushi/ripgrep"
    "uv" = "https://github.com/astral-sh/uv"
    "nextjs" = "https://github.com/vercel/next.js"
    "next.js" = "https://github.com/vercel/next.js"
    "vite" = "https://github.com/vitejs/vite"
    "ruff" = "https://github.com/astral-sh/ruff"
    "httpx" = "https://github.com/encode/httpx"
    "auroraview" = "https://github.com/loonghao/auroraview"
    "docker-cli" = "https://github.com/docker/cli"
}

# Resolve project URL
if ($Project -match "^https?://") {
    $RepoUrl = $Project
    $ProjectName = ($Project -split "/")[-1] -replace "\.git$", ""
} elseif ($KnownProjects.ContainsKey($Project.ToLower())) {
    $RepoUrl = $KnownProjects[$Project.ToLower()]
    $ProjectName = $Project.ToLower()
} else {
    Write-Host "Unknown project: $Project" -ForegroundColor Yellow
    Write-Host "Attempting to search GitHub..." -ForegroundColor Yellow
    $RepoUrl = "https://github.com/$Project"
    $ProjectName = ($Project -split "/")[-1]
}

$TestDir = Join-Path $TempDir "$ProjectName-test"
$VxRoot = "c:/github/vx"

Write-Host "`n=== Project Analyze ===" -ForegroundColor Cyan
Write-Host "Project: $ProjectName"
Write-Host "URL: $RepoUrl"
Write-Host "Test Dir: $TestDir"
Write-Host ""

# Step 1: Clone
if (Test-Path $TestDir) {
    Write-Host "Removing existing test directory..." -ForegroundColor Yellow
    Remove-Item -Recurse -Force $TestDir
}

Write-Host "Cloning repository..." -ForegroundColor Green
git clone --depth 1 $RepoUrl $TestDir
if ($LASTEXITCODE -ne 0) {
    Write-Host "Failed to clone repository!" -ForegroundColor Red
    exit 1
}

# Step 2: Analyze
Write-Host "`nRunning analysis..." -ForegroundColor Green
cargo run --manifest-path "$VxRoot/Cargo.toml" -p vx-project-analyzer --example analyze_project -- $TestDir

# Step 3: Prompt for cleanup
Write-Host "`n=== Analysis Complete ===" -ForegroundColor Cyan
Write-Host "Test directory: $TestDir"
Write-Host ""
$Cleanup = Read-Host "Clean up test directory? (y/N)"
if ($Cleanup -eq "y" -or $Cleanup -eq "Y") {
    Write-Host "Cleaning up..." -ForegroundColor Yellow
    Remove-Item -Recurse -Force $TestDir
    Write-Host "Done!" -ForegroundColor Green
} else {
    Write-Host "Test directory preserved at: $TestDir" -ForegroundColor Yellow
}
