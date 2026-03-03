$ErrorActionPreference = "Stop"
if ($null -ne (Get-Variable -Name PSNativeCommandUseErrorActionPreference -ErrorAction SilentlyContinue)) {
  $PSNativeCommandUseErrorActionPreference = $false
}

$repoUrl = "https://github.com/dotnet/samples.git"
$workRoot = Join-Path $PSScriptRoot "work"
$repoDir = Join-Path $workRoot "dotnet-samples"
$logDir = Join-Path $PSScriptRoot "logs"
$logFile = Join-Path $logDir "dotnet-samples-csc.log"

function Invoke-LoggedNative {
  param(
    [string]$Description,
    [scriptblock]$Script
  )

  "`n== $Description ==" | Tee-Object -FilePath $logFile -Append
  & $Script 2>&1 | Tee-Object -FilePath $logFile -Append
  if ($LASTEXITCODE -ne 0) {
    throw "$Description failed with exit code $LASTEXITCODE"
  }
}

New-Item -ItemType Directory -Force -Path $workRoot | Out-Null
New-Item -ItemType Directory -Force -Path $logDir | Out-Null

if (-not (Test-Path $repoDir)) {
  Invoke-LoggedNative "clone dotnet/samples with sparse checkout" {
    git clone --depth 1 --filter=blob:none --sparse $repoUrl $repoDir
  }
}

Push-Location $repoDir
try {
  Invoke-LoggedNative "set sparse-checkout to core" {
    git sparse-checkout set --cone core
  }

  "== dotnet samples build test ==" | Tee-Object -FilePath $logFile
  "Working directory: $repoDir" | Tee-Object -FilePath $logFile -Append

  Invoke-LoggedNative "vx version" {
    vx --version
  }

  Invoke-LoggedNative "ensure dotnet runtime" {
    vx install dotnet
  }

  Invoke-LoggedNative "vx dotnet version" {
    vx dotnet --version
  }

  $project = Get-ChildItem -Recurse -Filter *.csproj | Sort-Object FullName | Select-Object -First 1
  if (-not $project) {
    throw "No .csproj found in sparse-checked dotnet/samples"
  }

  Invoke-LoggedNative "restore sample project" {
    vx dotnet restore $project.FullName
  }

  Invoke-LoggedNative "build sample project" {
    vx dotnet build $project.FullName -c Release -nologo
  }

  $projectName = [System.IO.Path]::GetFileNameWithoutExtension($project.Name)
  $artifact = Get-ChildItem -Path (Join-Path $project.Directory.FullName "bin\Release") -Recurse -Filter "$projectName.dll" -ErrorAction SilentlyContinue | Select-Object -First 1
  if (-not $artifact) {
    throw "Expected build artifact not found under bin/Release for project: $($project.FullName)"
  }

  "dotnet samples build test passed" | Tee-Object -FilePath $logFile -Append
}
finally {
  Pop-Location
}
