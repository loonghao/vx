$ErrorActionPreference = "Stop"
$repoUrl = "https://github.com/iOfficeAI/AionUi.git"
# Keep the upstream workspace manifests and bun.lock from one reviewed revision.
$repoCommit = "6744099b279b991c17e31c243f0920477bd31cb6"
$workRoot = Join-Path $PSScriptRoot "work"
$repoDir = Join-Path $workRoot "aionui-$($repoCommit.Substring(0, 12))"
$logDir = Join-Path $PSScriptRoot "logs"
$logFile = Join-Path $logDir "aionui.log"

New-Item -ItemType Directory -Force -Path $workRoot | Out-Null
New-Item -ItemType Directory -Force -Path $logDir | Out-Null

"== AionUi build test ==" | Tee-Object -FilePath $logFile
"Working directory: $repoDir" | Tee-Object -FilePath $logFile -Append

if (-not (Test-Path $repoDir)) {
  git init $repoDir 2>&1 | Tee-Object -FilePath $logFile -Append
  if ($LASTEXITCODE -ne 0) { throw "git init failed with exit code $LASTEXITCODE" }
  git -C $repoDir remote add origin $repoUrl 2>&1 | Tee-Object -FilePath $logFile -Append
  if ($LASTEXITCODE -ne 0) { throw "git remote add failed with exit code $LASTEXITCODE" }
  git -C $repoDir fetch --depth 1 origin $repoCommit 2>&1 | Tee-Object -FilePath $logFile -Append
  if ($LASTEXITCODE -ne 0) { throw "git fetch failed with exit code $LASTEXITCODE" }
  git -C $repoDir checkout --detach $repoCommit 2>&1 | Tee-Object -FilePath $logFile -Append
  if ($LASTEXITCODE -ne 0) { throw "git checkout failed with exit code $LASTEXITCODE" }
}

$actualCommit = git -C $repoDir rev-parse HEAD
if ($LASTEXITCODE -ne 0) { throw "git rev-parse failed with exit code $LASTEXITCODE" }
if ("$actualCommit".Trim() -ne $repoCommit) { throw "AionUi checkout must be at $repoCommit; found $actualCommit" }
git -C $repoDir diff --quiet HEAD --
if ($LASTEXITCODE -ne 0) { throw "AionUi checkout has local changes or cannot be checked (exit code $LASTEXITCODE)" }
"Source commit: $actualCommit" | Tee-Object -FilePath $logFile -Append

Push-Location $repoDir
try {
  vx --version | Tee-Object -FilePath $logFile -Append
  if ($LASTEXITCODE -ne 0) { throw "vx --version failed with exit code $LASTEXITCODE" }
  # VX discovers this repository's existing Bun pin above the upstream checkout.
  vx bun --version | Tee-Object -FilePath $logFile -Append
  if ($LASTEXITCODE -ne 0) { throw "bun --version failed with exit code $LASTEXITCODE" }
  vx npm --version | Tee-Object -FilePath $logFile -Append
  if ($LASTEXITCODE -ne 0) { throw "npm --version failed with exit code $LASTEXITCODE" }

  # Follow upstream's Bun workspace contract without rewriting dependencies.
  vx bun install --frozen-lockfile 2>&1 | Tee-Object -FilePath $logFile -Append
  if ($LASTEXITCODE -ne 0) { throw "bun install failed with exit code $LASTEXITCODE" }

  # Source-build AionUi's locked native SQLite package for the VX Node ABI.
  # Bun workspace links can reference one build directory more than once;
  # serialize npm lifecycle scripts so those builds cannot race on Windows.
  vx npm rebuild better-sqlite3 --build-from-source --foreground-scripts 2>&1 | Tee-Object -FilePath $logFile -Append
  if ($LASTEXITCODE -ne 0) { throw "npm rebuild failed with exit code $LASTEXITCODE" }

  vx node -e "const Database = require('better-sqlite3'); const db = new Database(':memory:'); if (db.prepare('SELECT 1 AS value').get().value !== 1) { throw new Error('SQLite query failed'); } db.close(); console.log('AionUi native SQLite OK');" 2>&1 | Tee-Object -FilePath $logFile -Append
  if ($LASTEXITCODE -ne 0) { throw "node module validation failed with exit code $LASTEXITCODE" }

  git diff --quiet HEAD --
  if ($LASTEXITCODE -ne 0) { throw "AionUi source or lockfile changed during the test (exit code $LASTEXITCODE)" }
  "AionUi test passed (better-sqlite3 rebuilt for Node)" | Tee-Object -FilePath $logFile -Append
}
finally {
  Pop-Location
}
