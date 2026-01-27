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
    Set-PSReadLineOption -MaximumHistoryCount 10000 -ErrorAction SilentlyContinue
    Set-PSReadLineOption -HistoryNoDuplicates -ErrorAction SilentlyContinue

    # Enable predictive IntelliSense (PowerShell 7+)
    if ($PSVersionTable.PSVersion.Major -ge 7) {
        Set-PSReadLineOption -PredictionSource HistoryAndPlugin -ErrorAction SilentlyContinue
        Set-PSReadLineOption -PredictionViewStyle ListView -ErrorAction SilentlyContinue
    }

    # Enable standard key handlers
    Set-PSReadLineKeyHandler -Key UpArrow -Function HistorySearchBackward
    Set-PSReadLineKeyHandler -Key DownArrow -Function HistorySearchForward
    Set-PSReadLineKeyHandler -Key Tab -Function Complete

    # Add Ctrl+Space to show all possible completions
    Set-PSReadLineKeyHandler -Chord "Ctrl+Space" -Function MenuComplete

    # Better text selection with Shift+Arrow
    Set-PSReadLineKeyHandler -Key "Shift+LeftArrow" -Function SelectBackwardChar
    Set-PSReadLineKeyHandler -Key "Shift+RightArrow" -Function SelectForwardChar
    Set-PSReadLineKeyHandler -Key "Shift+UpArrow" -Function SelectBackwardLine
    Set-PSReadLineKeyHandler -Key "Shift+DownArrow" -Function SelectForwardLine
}

# Set custom prompt to indicate vx environment
function global:prompt {
    "($ProjectName[vx]) PS $($executionContext.SessionState.Path.CurrentLocation)$('>' * ($nestedPromptLevel + 1)) "
}

# Define helpful aliases
function global:vx-tools { Get-Command | Where-Object { $_.Source -match "vx" } }
function global:vx-exit { exit }
function global:vx-history { Get-Content (Get-PSReadLineOption).HistorySavePath | Select-Object -Last 20 }
function global:vx-clear-history {
    $historyPath = (Get-PSReadLineOption).HistorySavePath
    Clear-Content -Path $historyPath -Force
    Write-Host "History cleared" -ForegroundColor Green
}

# Load vx completion script if it exists
$vxCompletionScript = "$historyDir\vx_completion.ps1"
if (Test-Path $vxCompletionScript) {
    . $vxCompletionScript
}
