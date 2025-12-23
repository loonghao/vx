# Test runner script for vx project
# Provides various testing options with proper error handling

param(
    [string]$Type = "all",
    [switch]$Verbose,
    [switch]$Coverage,
    [switch]$Serial,
    [string]$Package = "",
    [string]$Test = ""
)

# Colors for output
$Red = "`e[31m"
$Green = "`e[32m"
$Yellow = "`e[33m"
$Blue = "`e[34m"
$Reset = "`e[0m"

function Write-Status {
    param([string]$Message, [string]$Color = $Blue)
    Write-Host "${Color}[INFO]${Reset} $Message"
}

function Write-Success {
    param([string]$Message)
    Write-Host "${Green}[SUCCESS]${Reset} $Message"
}

function Write-Error {
    param([string]$Message)
    Write-Host "${Red}[ERROR]${Reset} $Message"
}

function Write-Warning {
    param([string]$Message)
    Write-Host "${Yellow}[WARNING]${Reset} $Message"
}

# Ensure we're in the project root
if (-not (Test-Path "Cargo.toml")) {
    Write-Error "Must be run from project root directory"
    exit 1
}

# Build test command based on parameters
function Build-TestCommand {
    param([string]$TestType)

    $cmd = "cargo test"

    # Add package filter if specified
    if ($Package) {
        $cmd += " -p $Package"
    }

    # Add test filter if specified
    if ($Test) {
        $cmd += " $Test"
    }

    # Add type-specific flags
    switch ($TestType) {
        "unit" { $cmd += " --lib" }
        "integration" { $cmd += " --test '*'" }
        "doc" { $cmd += " --doc" }
        "all" { $cmd += " --all" }
    }

    # Add verbose flag if requested
    if ($Verbose) {
        $cmd += " -- --nocapture"
    }

    # Add serial execution if requested
    if ($Serial) {
        $cmd += " -- --test-threads=1"
    }

    return $cmd
}

# Run tests with error handling
function Run-Tests {
    param([string]$Command, [string]$Description)

    Write-Status "Running $Description..."
    Write-Status "Command: $Command"

    $startTime = Get-Date

    try {
        Invoke-Expression $Command
        $exitCode = $LASTEXITCODE

        $endTime = Get-Date
        $duration = $endTime - $startTime

        if ($exitCode -eq 0) {
            Write-Success "$Description completed successfully in $($duration.TotalSeconds.ToString('F2')) seconds"
            return $true
        } else {
            Write-Error "$Description failed with exit code $exitCode"
            return $false
        }
    }
    catch {
        Write-Error "Failed to run $Description`: $($_.Exception.Message)"
        return $false
    }
}

# Main execution
Write-Status "Starting vx test suite..."
Write-Status "Test type: $Type"

$success = $true

switch ($Type.ToLower()) {
    "unit" {
        $cmd = Build-TestCommand "unit"
        $success = Run-Tests $cmd "unit tests"
    }
    "integration" {
        $cmd = Build-TestCommand "integration"
        $success = Run-Tests $cmd "integration tests"
    }
    "doc" {
        $cmd = Build-TestCommand "doc"
        $success = Run-Tests $cmd "documentation tests"
    }
    "all" {
        # Run all test types
        $testTypes = @(
            @{Type = "unit"; Description = "unit tests"},
            @{Type = "integration"; Description = "integration tests"},
            @{Type = "doc"; Description = "documentation tests"}
        )

        foreach ($testType in $testTypes) {
            $cmd = Build-TestCommand $testType.Type
            $result = Run-Tests $cmd $testType.Description
            $success = $success -and $result

            if (-not $result) {
                Write-Warning "Continuing with remaining tests..."
            }
        }
    }
    "clippy" {
        $success = Run-Tests "cargo clippy --all -- -D warnings" "clippy linting"
    }
    "fmt" {
        $success = Run-Tests "cargo fmt --all -- --check" "format checking"
    }
    "check" {
        $success = Run-Tests "cargo check --all" "compilation check"
    }
    default {
        Write-Error "Unknown test type: $Type"
        Write-Status "Available types: unit, integration, doc, all, clippy, fmt, check"
        exit 1
    }
}

# Coverage report (if requested and available)
if ($Coverage -and $success) {
    Write-Status "Generating coverage report..."
    if (Get-Command "cargo-tarpaulin" -ErrorAction SilentlyContinue) {
        Run-Tests "cargo tarpaulin --out Html --output-dir target/coverage" "coverage report"
        Write-Status "Coverage report generated in target/coverage/"
    } else {
        Write-Warning "cargo-tarpaulin not found. Install with: cargo install cargo-tarpaulin"
    }
}

# Final status
if ($success) {
    Write-Success "All tests completed successfully!"
    exit 0
} else {
    Write-Error "Some tests failed!"
    exit 1
}
