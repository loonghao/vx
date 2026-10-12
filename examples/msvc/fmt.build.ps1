$ErrorActionPreference = "Stop"
if ($null -ne (Get-Variable -Name PSNativeCommandUseErrorActionPreference -ErrorAction SilentlyContinue)) {
  $PSNativeCommandUseErrorActionPreference = $false
}

$generator = $env:CMAKE_GENERATOR
$instancePath = $env:CMAKE_GENERATOR_INSTANCE
if (-not $generator -or -not $instancePath) {
  if ($generator -or $instancePath) {
    throw "Set both CMAKE_GENERATOR and CMAKE_GENERATOR_INSTANCE, or leave both unset for discovery"
  }
  . (Join-Path $PSScriptRoot '../../scripts/ensure-msvc-spectre.ps1')
  $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
  $instance = Get-MsvcInstallation -VsWherePath $vswhere
  $generator = Get-MsvcCMakeGenerator -InstallationVersion $instance.installationVersion
  $instancePath = $instance.installationPath
}

$repoUrl  = "https://github.com/fmtlib/fmt.git"
$workRoot = Join-Path $PSScriptRoot "work"
$repoDir  = Join-Path $workRoot "fmt"
$buildDir = Join-Path $repoDir "build-vx"
$logDir   = Join-Path $PSScriptRoot "logs"
$logFile  = Join-Path $logDir "fmt.log"

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
New-Item -ItemType Directory -Force -Path $logDir   | Out-Null

if (-not (Test-Path $repoDir)) {
  Invoke-LoggedNative "clone fmtlib/fmt" {
    git clone --depth 1 $repoUrl $repoDir
  }
}

Push-Location $repoDir
try {
  "== fmt cmake + msvc build test ==" | Tee-Object -FilePath $logFile
  "Working directory: $repoDir"       | Tee-Object -FilePath $logFile -Append

  Invoke-LoggedNative "vx version" {
    vx --version
  }

  Invoke-LoggedNative "cmake version" {
    vx cmake --version
  }

  Invoke-LoggedNative "cl version" {
    vx cl /? 2>&1 | Select-Object -First 5
  }

  Invoke-LoggedNative "cmake configure" {
    vx cmake -S . -B $buildDir -G $generator -A x64 "-DCMAKE_GENERATOR_INSTANCE=$instancePath" -DFMT_TEST=OFF
  }

  Invoke-LoggedNative "cmake build" {
    vx cmake --build $buildDir --config Release
  }

  $artifact = Join-Path $buildDir "Release\fmt.lib"
  if (-not (Test-Path $artifact)) {
    throw "Expected artifact not found: $artifact"
  }

  "fmt build test passed" | Tee-Object -FilePath $logFile -Append
}
finally {
  Pop-Location
}
