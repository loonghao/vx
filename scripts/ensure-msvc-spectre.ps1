# Discover the installed VC instance instead of assuming a VS year or edition.
# Dot-source this file to test the setup functions without changing the machine.

function Get-MsvcInstallation {
  param([string]$VsWherePath)

  $PSNativeCommandUseErrorActionPreference = $false
  if (-not (Test-Path -LiteralPath $VsWherePath -PathType Leaf)) {
    throw "Visual Studio locator not found at $VsWherePath"
  }

  $json = (& $VsWherePath -latest -products '*' `
    -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -format json -utf8) -join "`n"
  if ($LASTEXITCODE -ne 0) {
    throw "Visual Studio locator failed with exit code $LASTEXITCODE"
  }

  $instances = @($json | ConvertFrom-Json)
  if ($instances.Count -ne 1 -or [string]::IsNullOrWhiteSpace($instances[0].installationPath)) {
    throw "Expected one installed Visual Studio instance with x64 C++ tools"
  }
  $instance = $instances[0]
  $tools = Join-Path $instance.installationPath 'VC/Tools/MSVC'
  if (-not (Test-Path -LiteralPath $tools -PathType Container)) {
    throw "MSVC tools were not found in the selected instance: $tools"
  }
  return $instance
}

function Test-MsvcSpectre {
  param([string]$InstallationPath)

  # The examples build x64 targets. ARM64 libraries alone do not satisfy them.
  # MSVC places the mitigation before the target architecture: lib/spectre/x64.
  $libraryPath = Join-Path $InstallationPath 'VC/Tools/MSVC/*/lib/spectre/x64'
  $present = Test-Path -Path $libraryPath -PathType Container
  if ($present) {
    Write-Host "Found x64 Spectre libraries matching $libraryPath"
  }
  return [bool]$present
}

function Get-MsvcCMakeGenerator {
  param([string]$InstallationVersion)

  # These are CMake's documented generator names, not a name inferred from a year.
  # https://cmake.org/cmake/help/latest/manual/cmake-generators.7.html
  switch (([version]$InstallationVersion).Major) {
    17 { return 'Visual Studio 17 2022' }
    18 { return 'Visual Studio 18 2026' }
    default { throw "Unsupported Visual Studio version for the CMake example: $InstallationVersion" }
  }
}

function Ensure-MsvcSpectre {
  param(
    [string]$InstallerDirectory = (Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer')
  )

  $instance = Get-MsvcInstallation -VsWherePath (Join-Path $InstallerDirectory 'vswhere.exe')
  $vsRoot = $instance.installationPath
  Write-Host "Selected Visual Studio $($instance.installationVersion) at $vsRoot"

  if (Test-MsvcSpectre -InstallationPath $vsRoot) {
    Write-Host "x64 Spectre libraries already installed in the selected instance"
    return $instance
  }

  $installer = Join-Path $InstallerDirectory 'setup.exe'
  if (-not (Test-Path -LiteralPath $installer -PathType Leaf)) {
    throw "Visual Studio Installer not found at $installer"
  }
  if ([string]::IsNullOrWhiteSpace($instance.channelId)) {
    throw "The selected Visual Studio instance has no channelId for modification"
  }

  # Start-Process joins ArgumentList into a command line, so preserve the quotes
  # around an installation path containing spaces. --wait is bootstrapper-only;
  # use PowerShell's -Wait and the returned process ExitCode for setup.exe.
  $arguments = @(
    'modify', '--installPath', ('"{0}"' -f $vsRoot),
    '--channelId', $instance.channelId,
    '--add', 'Microsoft.VisualStudio.Component.VC.ATL.Spectre',
    '--add', 'Microsoft.VisualStudio.Component.VC.MFC.Spectre',
    '--add', 'Microsoft.VisualStudio.Component.VC.Runtimes.x86.x64.Spectre',
    '--quiet', '--norestart'
  )
  $process = Start-Process -FilePath $installer -ArgumentList $arguments -Wait -PassThru
  $exitCode = $process.ExitCode
  Write-Host "Visual Studio Installer exit code: $exitCode"
  if ($exitCode -notin @(0, 3010)) {
    throw "Spectre library installation failed for '$vsRoot' with exit code $exitCode"
  }
  if ($exitCode -eq 3010) {
    Write-Warning "Installer requested a restart; validating the installed libraries before continuing"
  }
  if (-not (Test-MsvcSpectre -InstallationPath $vsRoot)) {
    throw "Installer returned $exitCode but x64 Spectre libraries are still missing in '$vsRoot'"
  }
  return $instance
}

if ($MyInvocation.InvocationName -ne '.') {
  $ErrorActionPreference = 'Stop'
  Ensure-MsvcSpectre | Out-Null
}
