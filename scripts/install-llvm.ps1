# PowerShell script to install LLVM on Windows
# This script is used by GoReleaser for PGO builds

Write-Host "Installing LLVM for PGO builds..."

# Check if running on Windows
if ($IsWindows -or $env:OS -eq "Windows_NT") {
    # Try Chocolatey first
    if (Get-Command choco -ErrorAction SilentlyContinue) {
        Write-Host "Installing LLVM via Chocolatey..."
        choco install llvm -y
    }
    # Try winget as fallback
    elseif (Get-Command winget -ErrorAction SilentlyContinue) {
        Write-Host "Installing LLVM via winget..."
        winget install LLVM.LLVM
    }
    else {
        Write-Warning "Neither Chocolatey nor winget found. Please install LLVM manually."
        Write-Host "Download from: https://releases.llvm.org/"
        exit 1
    }
}
else {
    Write-Host "This script is for Windows only. Use install-llvm.sh for Unix systems."
    exit 1
}

Write-Host "LLVM installation completed."
