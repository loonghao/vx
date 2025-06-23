# Test script to validate publishing order without actually publishing
# This script performs dry-run packaging to detect dependency issues

param(
    [switch]$Verbose
)

# Function to write colored output
function Write-Status {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Blue
}

function Write-Success {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "[WARNING] $Message" -ForegroundColor Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

# Function to test package a crate
function Test-PackageCrate {
    param(
        [string]$CratePath,
        [string]$CrateName
    )
    
    Write-Status "Testing package for $CrateName..."
    
    Push-Location $CratePath
    
    try {
        # Perform dry run packaging
        $result = cargo package --dry-run 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-Success "$CrateName packages successfully"
            return $true
        } else {
            Write-Error "$CrateName packaging failed"
            if ($Verbose) {
                Write-Host $result -ForegroundColor Red
            }
            return $false
        }
    }
    finally {
        Pop-Location
    }
}

function Main {
    Write-Status "Testing VX crates packaging in dependency order..."
    
    # Ensure we're in the project root
    if (-not (Test-Path "Cargo.toml") -or -not (Test-Path "crates")) {
        Write-Error "Please run this script from the project root directory"
        exit 1
    }
    
    # Define test order (same as publishing order)
    $TestOrder = @(
        # Layer 1: No internal dependencies
        @{ Path = "crates/vx-dependency"; Name = "vx-dependency" },
        @{ Path = "crates/vx-paths"; Name = "vx-paths" },
        
        # Layer 2: Depend on Layer 1
        @{ Path = "crates/vx-plugin"; Name = "vx-plugin" },
        @{ Path = "crates/vx-version"; Name = "vx-version" },
        
        # Layer 3: Depend on Layer 1-2
        @{ Path = "crates/vx-config"; Name = "vx-config" },
        @{ Path = "crates/vx-installer"; Name = "vx-installer" },
        @{ Path = "crates/vx-download"; Name = "vx-download" },
        
        # Layer 4: Depend on Layer 1-3
        @{ Path = "crates/vx-core"; Name = "vx-core" },
        @{ Path = "crates/vx-benchmark"; Name = "vx-benchmark" },
        
        # Layer 5: Tool standard
        @{ Path = "crates/vx-tool-standard"; Name = "vx-tool-standard" },
        
        # Layer 6: Tool implementations
        @{ Path = "crates/vx-tools/vx-tool-npm"; Name = "vx-tool-npm" },
        @{ Path = "crates/vx-tools/vx-tool-uv"; Name = "vx-tool-uv" },
        @{ Path = "crates/vx-tools/vx-tool-python"; Name = "vx-tool-python" },
        @{ Path = "crates/vx-tools/vx-tool-rust"; Name = "vx-tool-rust" },
        @{ Path = "crates/vx-tools/vx-tool-go"; Name = "vx-tool-go" },
        @{ Path = "crates/vx-tools/vx-tool-bun"; Name = "vx-tool-bun" },
        @{ Path = "crates/vx-tools/vx-tool-node"; Name = "vx-tool-node" },
        @{ Path = "crates/vx-tools/vx-tool-pnpm"; Name = "vx-tool-pnpm" },
        @{ Path = "crates/vx-tools/vx-tool-yarn"; Name = "vx-tool-yarn" },
        
        # Layer 7: CLI and main package
        @{ Path = "crates/vx-cli"; Name = "vx-cli" },
        @{ Path = "."; Name = "vx" }
    )
    
    $FailedCrates = @()
    $TotalCrates = $TestOrder.Count
    $Current = 0
    
    foreach ($Entry in $TestOrder) {
        $Current++
        Write-Status "[$Current/$TotalCrates] Testing $($Entry.Name) ($($Entry.Path))..."
        
        if (-not (Test-PackageCrate -CratePath $Entry.Path -CrateName $Entry.Name)) {
            $FailedCrates += $Entry.Name
        }
    }
    
    # Summary
    Write-Host ""
    Write-Status "Packaging Test Summary:"
    if ($FailedCrates.Count -eq 0) {
        Write-Success "All crates can be packaged successfully!"
        Write-Status "Publishing order is correct and ready for crates.io"
    } else {
        Write-Error "Some crates failed packaging tests:"
        foreach ($Crate in $FailedCrates) {
            Write-Host "  - $Crate" -ForegroundColor Red
        }
        Write-Error "Fix these issues before attempting to publish"
        exit 1
    }
}

# Run main function
Main
