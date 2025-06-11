#!/usr/bin/env pwsh
# PGO (Profile-Guided Optimization) build script for vx
# This script automates the PGO build process for optimal performance

param(
    [switch]$Clean,
    [switch]$Verbose,
    [string]$Target = "x86_64-pc-windows-msvc"
)

$ErrorActionPreference = "Stop"

# Colors for output
$Green = "`e[32m"
$Yellow = "`e[33m"
$Red = "`e[31m"
$Blue = "`e[34m"
$Reset = "`e[0m"

function Write-Step {
    param($Message)
    Write-Host "${Blue}[STEP]${Reset} $Message"
}

function Write-Success {
    param($Message)
    Write-Host "${Green}[SUCCESS]${Reset} $Message"
}

function Write-Warning {
    param($Message)
    Write-Host "${Yellow}[WARNING]${Reset} $Message"
}

function Write-Error {
    param($Message)
    Write-Host "${Red}[ERROR]${Reset} $Message"
}

# Check if we're in the right directory
if (!(Test-Path "Cargo.toml")) {
    Write-Error "Must be run from the project root directory"
    exit 1
}

# Clean previous builds if requested
if ($Clean) {
    Write-Step "Cleaning previous builds..."
    cargo clean
    Remove-Item -Path "pgo-data" -Recurse -Force -ErrorAction SilentlyContinue
}

# Create PGO data directory
$PgoDataDir = "pgo-data"
New-Item -ItemType Directory -Path $PgoDataDir -Force | Out-Null

Write-Step "Starting PGO optimization process..."

# Step 1: Build with PGO instrumentation
Write-Step "Building instrumented binary for profile collection..."
$env:RUSTFLAGS = "-Cprofile-generate=$PgoDataDir"

try {
    if ($Verbose) {
        cargo build --release --target $Target
    } else {
        cargo build --release --target $Target 2>$null
    }
    Write-Success "Instrumented binary built successfully"
} catch {
    Write-Error "Failed to build instrumented binary: $_"
    exit 1
}

# Step 2: Run training workload to collect profile data
Write-Step "Collecting profile data with training workload..."

$BinaryPath = "target/$Target/release/vx.exe"
if (!(Test-Path $BinaryPath)) {
    Write-Error "Binary not found at $BinaryPath"
    exit 1
}

# Define training commands that represent typical usage
$TrainingCommands = @(
    "version",
    "list",
    "plugin list",
    "plugin stats",
    "config",
    "--help"
)

Write-Step "Running training workload..."
foreach ($cmd in $TrainingCommands) {
    Write-Host "  Running: vx $cmd"
    try {
        if ($cmd -eq "--help") {
            & $BinaryPath --help | Out-Null
        } else {
            $args = $cmd.Split(' ')
            & $BinaryPath @args | Out-Null
        }
    } catch {
        Write-Warning "Command 'vx $cmd' failed, continuing..."
    }
}

# Check if profile data was generated
$ProfileFiles = Get-ChildItem -Path $PgoDataDir -Filter "*.profraw" -ErrorAction SilentlyContinue
if ($ProfileFiles.Count -eq 0) {
    Write-Error "No profile data generated. Check if the binary was built correctly."
    exit 1
}

Write-Success "Profile data collected: $($ProfileFiles.Count) files"

# Step 3: Merge profile data
Write-Step "Merging profile data..."
$MergedProfile = "$PgoDataDir/merged.profdata"

try {
    # Use llvm-profdata to merge profile data
    $LlvmProfdata = Get-Command "llvm-profdata" -ErrorAction SilentlyContinue
    if ($LlvmProfdata) {
        & llvm-profdata merge -output=$MergedProfile $PgoDataDir/*.profraw
        Write-Success "Profile data merged successfully"
    } else {
        Write-Warning "llvm-profdata not found, using rustc's built-in merging"
        # Rust will automatically merge .profraw files
        $MergedProfile = $PgoDataDir
    }
} catch {
    Write-Warning "Profile merging failed, using raw data: $_"
    $MergedProfile = $PgoDataDir
}

# Step 4: Build optimized binary using profile data
Write-Step "Building PGO-optimized binary..."
$env:RUSTFLAGS = "-Cprofile-use=$MergedProfile -Cllvm-args=-pgo-warn-missing-function"

try {
    if ($Verbose) {
        cargo build --release --target $Target
    } else {
        cargo build --release --target $Target 2>$null
    }
    Write-Success "PGO-optimized binary built successfully"
} catch {
    Write-Error "Failed to build PGO-optimized binary: $_"
    exit 1
}

# Step 5: Verify the optimized binary
Write-Step "Verifying optimized binary..."
try {
    & $BinaryPath version | Out-Null
    Write-Success "Optimized binary verification passed"
} catch {
    Write-Error "Optimized binary verification failed: $_"
    exit 1
}

# Cleanup
$env:RUSTFLAGS = ""

Write-Success "PGO optimization completed successfully!"
Write-Host ""
Write-Host "${Green}Optimized binary location:${Reset} $BinaryPath"
Write-Host "${Green}Profile data location:${Reset} $PgoDataDir"
Write-Host ""
Write-Host "${Yellow}Performance improvements:${Reset}"
Write-Host "  • Faster startup time"
Write-Host "  • Better branch prediction"
Write-Host "  • Optimized hot code paths"
Write-Host "  • Reduced instruction cache misses"
