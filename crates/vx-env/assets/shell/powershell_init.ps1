# VX Shell Environment Initialization
# This script is embedded into vx-env binary at compile time
# Variables $ProjectName and $Tools are set by the caller before this script

# Load user's profile first to get PSReadLine and other interactive features
if (Test-Path $PROFILE) {
    . $PROFILE
}

# Ensure PSReadLine is loaded for interactive features (arrow keys, history, etc.)
if (-not (Get-Module PSReadLine)) {
    if (Get-Module -ListAvailable -Name PSReadLine) {
        Import-Module PSReadLine -ErrorAction SilentlyContinue
    }
}

# Configure PSReadLine if available
if (Get-Module PSReadLine) {
    # Ensure history directory exists
    $historyDir = "$env:APPDATA\vx"
    if (-not (Test-Path $historyDir)) {
        New-Item -ItemType Directory -Path $historyDir -Force | Out-Null
    }
    
    # Use shared history file across sessions
    Set-PSReadLineOption -HistorySavePath "$historyDir\powershell_history.txt" -ErrorAction SilentlyContinue
    
    # Enable predictive IntelliSense (PowerShell 7+)
    if ($PSVersionTable.PSVersion.Major -ge 7) {
        Set-PSReadLineOption -PredictionSource History -ErrorAction SilentlyContinue
    }
    
    # Enable standard key handlers
    Set-PSReadLineKeyHandler -Key UpArrow -Function HistorySearchBackward
    Set-PSReadLineKeyHandler -Key DownArrow -Function HistorySearchForward
    Set-PSReadLineKeyHandler -Key Tab -Function Complete
}

# Set custom prompt to indicate vx environment
function global:prompt {
    "($ProjectName[vx]) PS $($executionContext.SessionState.Path.CurrentLocation)$('>' * ($nestedPromptLevel + 1)) "
}

# Show welcome message
Write-Host ""
Write-Host "VX Shell Environment" -ForegroundColor Green
Write-Host "Project: $ProjectName" -ForegroundColor Cyan
if ($Tools) {
    Write-Host "Tools: $Tools" -ForegroundColor Cyan
}
Write-Host ""

# Define helpful aliases
function global:vx-tools { Get-Command | Where-Object { $_.Source -match "vx" } }
function global:vx-exit { exit }
