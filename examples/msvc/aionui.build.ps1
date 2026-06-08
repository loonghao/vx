$ErrorActionPreference = "Stop"
$repoUrl = "https://github.com/iOfficeAI/AionUi.git"
$workRoot = Join-Path $PSScriptRoot "work"
$repoDir = Join-Path $workRoot "aionui"
$logDir = Join-Path $PSScriptRoot "logs"
$logFile = Join-Path $logDir "aionui.log"

New-Item -ItemType Directory -Force -Path $workRoot | Out-Null
New-Item -ItemType Directory -Force -Path $logDir | Out-Null

if (-not (Test-Path $repoDir)) {
  git clone --depth 1 $repoUrl $repoDir
}

Push-Location $repoDir
try {
  # Workaround: npm EOVERRIDE — @codemirror/language is both a direct dependency
  # and in overrides/resolutions. npm 10+ rejects overrides on direct deps when
  # the version specs differ. Remove the redundant override/resolution entries.
  vx npm pkg delete overrides.@codemirror/language resolutions.@codemirror/language 2>$null

  "== AionUi build test ==" | Tee-Object -FilePath $logFile
  "Working directory: $repoDir" | Tee-Object -FilePath $logFile -Append

  vx --version | Tee-Object -FilePath $logFile -Append
  vx npm --version | Tee-Object -FilePath $logFile -Append

  vx npm install 2>&1 | Tee-Object -FilePath $logFile -Append

  vx npm rebuild better-sqlite3 --build-from-source 2>&1 | Tee-Object -FilePath $logFile -Append

  vx node -e "require('better-sqlite3');require('node-pty');console.log('AionUi native modules OK')" 2>&1 | Tee-Object -FilePath $logFile -Append

  "AionUi test passed" | Tee-Object -FilePath $logFile -Append
}
finally {
  Pop-Location
}
