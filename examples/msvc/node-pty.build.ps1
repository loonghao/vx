$ErrorActionPreference = "Stop"
$repoUrl = "https://github.com/microsoft/node-pty.git"
$workRoot = Join-Path $PSScriptRoot "work"
$repoDir = Join-Path $workRoot "node-pty"
$logDir = Join-Path $PSScriptRoot "logs"
$logFile = Join-Path $logDir "node-pty.log"

New-Item -ItemType Directory -Force -Path $workRoot | Out-Null
New-Item -ItemType Directory -Force -Path $logDir | Out-Null

if (-not (Test-Path $repoDir)) {
  git clone --depth 1 $repoUrl $repoDir
}

Push-Location $repoDir
try {
  "== node-pty build test ==" | Tee-Object -FilePath $logFile
  "Working directory: $repoDir" | Tee-Object -FilePath $logFile -Append

  vx --version | Tee-Object -FilePath $logFile -Append
  vx npm --version | Tee-Object -FilePath $logFile -Append

  vx npm install --build-from-source 2>&1 | Tee-Object -FilePath $logFile -Append

  vx npm rebuild --build-from-source 2>&1 | Tee-Object -FilePath $logFile -Append

  vx node -e "require('./'); console.log('node-pty load OK')" 2>&1 | Tee-Object -FilePath $logFile -Append

  "node-pty test passed" | Tee-Object -FilePath $logFile -Append
}
finally {
  Pop-Location
}
