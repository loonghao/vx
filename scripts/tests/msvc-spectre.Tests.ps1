BeforeAll {
  . (Join-Path $PSScriptRoot '../ensure-msvc-spectre.ps1')
  function Invoke-FakeVsWhere {
    param([switch]$latest, [string]$products, [string]$version,
      [string]$requires, [string]$format, [switch]$utf8)
  }
}

Describe 'Visual Studio discovery' {
  BeforeEach {
    Mock Test-Path { $true }
    Mock Invoke-FakeVsWhere {
      $global:LASTEXITCODE = 0
      '[{"installationPath":"C:\\Visual Studio\\2022\\Enterprise","installationVersion":"17.14.37710.0","channelId":"VisualStudio.17.Release"}]'
    }
  }

  It 'selects the common supported VS17 range without assuming an edition or path' {
    $instance = Get-MsvcInstallation -VsWherePath 'Invoke-FakeVsWhere'
    $instance.installationPath | Should -Be 'C:\Visual Studio\2022\Enterprise'
    $instance.installationVersion | Should -Be '17.14.37710.0'
    Should -Invoke Invoke-FakeVsWhere -Times 1 -ParameterFilter {
      $latest -and $products -eq '*' -and $version -eq '[17.0,18.0)' -and
      $requires -eq 'Microsoft.VisualStudio.Component.VC.Tools.x86.x64'
    }
  }

  It 'rejects a locator result outside the required compatibility range' {
    Mock Invoke-FakeVsWhere {
      $global:LASTEXITCODE = 0
      '[{"installationPath":"C:\\VS18","installationVersion":"18.10.1.0"}]'
    }
    { Get-MsvcInstallation -VsWherePath 'Invoke-FakeVsWhere' } | Should -Throw '*outside required range*'
  }

  It 'allows explicit VS18 selection when consumers support it' {
    Mock Invoke-FakeVsWhere {
      $global:LASTEXITCODE = 0
      '[{"installationPath":"C:\\VS18","installationVersion":"18.10.1.0"}]'
    }
    (Get-MsvcInstallation -VsWherePath 'Invoke-FakeVsWhere' -MajorVersion 18).installationVersion |
      Should -Be '18.10.1.0'
    Should -Invoke Invoke-FakeVsWhere -Times 1 -ParameterFilter { $version -eq '[18.0,19.0)' }
  }

  It 'fails clearly when vswhere is missing' {
    Mock Test-Path { $false }
    { Get-MsvcInstallation -VsWherePath 'Invoke-FakeVsWhere' } | Should -Throw '*locator not found*'
    Should -Invoke Invoke-FakeVsWhere -Times 0
  }

  It 'reports the locator exit code' {
    Mock Invoke-FakeVsWhere { $global:LASTEXITCODE = 42; '[]' }
    { Get-MsvcInstallation -VsWherePath 'Invoke-FakeVsWhere' } | Should -Throw '*exit code 42*'
  }

  It 'rejects an empty discovery result' {
    Mock Invoke-FakeVsWhere { $global:LASTEXITCODE = 0; '[]' }
    { Get-MsvcInstallation -VsWherePath 'Invoke-FakeVsWhere' } | Should -Throw '*Expected one installed*'
  }

  It 'rejects a selected instance without an MSVC directory' {
    Mock Test-Path { param($LiteralPath) $LiteralPath -eq 'Invoke-FakeVsWhere' }
    { Get-MsvcInstallation -VsWherePath 'Invoke-FakeVsWhere' } | Should -Throw '*MSVC tools were not found*'
  }
}

Describe 'Spectre library detection' {
  It 'finds x64 libraries below the selected instance' {
    $root = Join-Path $TestDrive 'VS Enterprise'
    New-Item -ItemType Directory -Force (Join-Path $root 'VC/Tools/MSVC/14.50/lib/spectre/x64') | Out-Null
    Test-MsvcSpectre -InstallationPath $root | Should -BeTrue
  }

  It 'does not treat ARM64-only libraries as x64 support' {
    $root = Join-Path $TestDrive 'VS ARM64'
    New-Item -ItemType Directory -Force (Join-Path $root 'VC/Tools/MSVC/14.50/lib/spectre/arm64') | Out-Null
    Test-MsvcSpectre -InstallationPath $root | Should -BeFalse
  }

  It 'rejects the old inverted library directory layout' {
    $root = Join-Path $TestDrive 'VS incorrect layout'
    New-Item -ItemType Directory -Force (Join-Path $root 'VC/Tools/MSVC/14.50/lib/x64/spectre') | Out-Null
    Test-MsvcSpectre -InstallationPath $root | Should -BeFalse
  }
}

Describe 'Spectre setup' {
  BeforeEach {
    Mock Get-MsvcInstallation {
      [pscustomobject]@{
        installationPath = 'C:\Visual Studio\2022\Enterprise'
        installationVersion = '17.14.37710.0'
        channelId = 'VisualStudio.17.Release'
      }
    }
    Mock Test-Path { $true }
    Mock Test-MsvcSpectre { $true }
    Mock Start-Process { [pscustomobject]@{ ExitCode = 0 } }
  }

  It 'reuses installed libraries without running the installer' {
    (Ensure-MsvcSpectre -InstallerDirectory 'C:\VS Installer').installationPath |
      Should -Be 'C:\Visual Studio\2022\Enterprise'
    Should -Invoke Start-Process -Times 0
    Should -Invoke Test-MsvcSpectre -Times 1 -ParameterFilter {
      $InstallationPath -eq 'C:\Visual Studio\2022\Enterprise'
    }
  }

  It 'modifies and rechecks the same instance after a successful install' {
    $script:checks = 0
    Mock Test-MsvcSpectre { $script:checks++; $script:checks -gt 1 }
    Ensure-MsvcSpectre -InstallerDirectory 'C:\VS Installer' | Out-Null
    Should -Invoke Start-Process -Times 1 -ParameterFilter {
      $FilePath -eq 'C:\VS Installer\setup.exe' -and $Wait -and $PassThru -and
      $ArgumentList -contains '"C:\Visual Studio\2022\Enterprise"' -and
      $ArgumentList -contains 'VisualStudio.17.Release' -and
      $ArgumentList -notcontains '--wait' -and
      $ArgumentList -contains 'Microsoft.VisualStudio.Component.VC.Runtimes.x86.x64.Spectre'
    }
    Should -Invoke Test-MsvcSpectre -Times 2 -ParameterFilter {
      $InstallationPath -eq 'C:\Visual Studio\2022\Enterprise'
    }
  }

  It 'reports an unsuccessful installer exit code' {
    Mock Test-MsvcSpectre { $false }
    Mock Start-Process { [pscustomobject]@{ ExitCode = 1603 } }
    { Ensure-MsvcSpectre -InstallerDirectory 'C:\VS Installer' } | Should -Throw '*exit code 1603*'
  }

  It 'accepts 3010 only after installed libraries are verified' {
    $script:checks = 0
    Mock Test-MsvcSpectre { $script:checks++; $script:checks -gt 1 }
    Mock Start-Process { [pscustomobject]@{ ExitCode = 3010 } }
    Ensure-MsvcSpectre -InstallerDirectory 'C:\VS Installer' | Out-Null
    Should -Invoke Test-MsvcSpectre -Times 2
  }

  It 'rejects successful exit codes when libraries are still absent' -TestCases @(
    @{ Code = 0 }, @{ Code = 3010 }
  ) {
    param($Code)
    Mock Test-MsvcSpectre { $false }
    $script:installerExitCode = $Code
    Mock Start-Process { [pscustomobject]@{ ExitCode = $script:installerExitCode } }
    { Ensure-MsvcSpectre -InstallerDirectory 'C:\VS Installer' } | Should -Throw '*libraries are still missing*'
  }

  It 'fails before modification if the installer is missing' {
    Mock Test-MsvcSpectre { $false }
    Mock Test-Path { $false }
    { Ensure-MsvcSpectre -InstallerDirectory 'C:\VS Installer' } | Should -Throw '*Installer not found*'
    Should -Invoke Start-Process -Times 0
  }

  It 'does not guess an instance when discovery fails' {
    Mock Get-MsvcInstallation { throw 'No VC instance' }
    { Ensure-MsvcSpectre -InstallerDirectory 'C:\VS Installer' } | Should -Throw '*No VC instance*'
    Should -Invoke Start-Process -Times 0
  }
}

Describe 'CMake generator selection' {
  It 'preserves the documented VS2022 and VS2026 generators' -TestCases @(
    @{ Version = '17.14.1.0'; Generator = 'Visual Studio 17 2022' },
    @{ Version = '18.10.12217.157'; Generator = 'Visual Studio 18 2026' }
  ) {
    param($Version, $Generator)
    Get-MsvcCMakeGenerator -InstallationVersion $Version | Should -Be $Generator
  }

  It 'fails explicitly for an unknown Visual Studio major version' {
    { Get-MsvcCMakeGenerator -InstallationVersion '19.0.0.0' } | Should -Throw '*Unsupported Visual Studio version*'
  }

  It 'passes the selected generator and instance into fmt configuration' {
    $fmt = Get-Content (Join-Path $PSScriptRoot '../../examples/msvc/fmt.build.ps1') -Raw
    $fmt | Should -Match '-G \$generator -A x64'
    $fmt | Should -Match '-DCMAKE_GENERATOR_INSTANCE=\$instancePath'
    $fmt | Should -Not -Match '-G "Visual Studio 17 2022"'
  }
}

Describe 'Workflow consumer compatibility' {
  It 'initializes VX before checking the common instance and binds CMake and node-gyp to it' {
    $workflow = Get-Content (Join-Path $PSScriptRoot '../../.github/workflows/examples-msvc.yml') -Raw
    $workflow | Should -Match '(?s)vx cl /\?.*Ensure-MsvcSpectre -MajorVersion 17'
    $workflow | Should -Match 'CMAKE_GENERATOR_INSTANCE=\$\(\$instance.installationPath\)'
    $workflow | Should -Match 'npm_config_msvs_version=\$\(\$instance.installationPath\)'
    $workflow | Should -Match 'npm_package_config_node_gyp_msvs_version=\$\(\$instance.installationPath\)'
  }
}

Describe 'Workflow Pester result gate' -Skip:(-not $IsWindows) {
  It 'returns <ExpectedCode> for <Name>' -TestCases @(
    @{ Name = 'passing tests'; ExpectedCode = 0; Content = "Describe 'fixture' { It 'passes' { 1 | Should -Be 1 } }" },
    @{ Name = 'discovery failure'; ExpectedCode = 1; Content = "throw 'intentional discovery failure'" },
    @{ Name = 'BeforeAll failure'; ExpectedCode = 1; Content = "Describe 'fixture' { BeforeAll { throw 'intentional setup failure' }; It 'cannot run' { 1 | Should -Be 1 } }" },
    @{ Name = 'assertion failure'; ExpectedCode = 1; Content = "Describe 'fixture' { It 'fails' { 1 | Should -Be 2 } }" },
    @{ Name = 'zero tests'; ExpectedCode = 1; Content = "Describe 'empty fixture' {}" },
    @{ Name = 'only skipped tests'; ExpectedCode = 1; Content = "Describe 'fixture' { It 'is skipped' -Skip { 1 | Should -Be 1 } }" }
  ) {
    param($Name, $ExpectedCode, $Content)
    $workflow = Get-Content (Join-Path $PSScriptRoot '../../.github/workflows/examples-msvc.yml') -Raw
    $gate = [regex]::Match($workflow, '(?ms)^          Import-Module Pester\b.*?^          exit 0\r?$')
    $gate.Success | Should -BeTrue
    # Exercise the actual workflow gate in a child process, including discovery
    # and container failures which are not represented by FailedCount alone.
    $runnerContent = $gate.Value -replace '(?m)^          ', ''
    $originalPaths = '-Path scripts/tests/msvc-spectre.Tests.ps1,scripts/tests/aionui-exit.Tests.ps1'
    $runnerContent | Should -Match ([regex]::Escape($originalPaths))
    $runnerContent = $runnerContent.Replace($originalPaths, '-Path $args[0]')
    $fixture = Join-Path $TestDrive ([guid]::NewGuid().ToString())
    New-Item -ItemType Directory -Path $fixture | Out-Null
    $runner = Join-Path $fixture 'run-gate.ps1'
    $testFile = Join-Path $fixture 'fixture.Tests.ps1'
    $runnerContent | Set-Content $runner
    $Content | Set-Content $testFile
    $PSNativeCommandUseErrorActionPreference = $false

    & (Join-Path $PSHOME 'pwsh.exe') -NoProfile -File $runner $testFile 2>&1 |
      Set-Content (Join-Path $fixture 'gate.log')

    $LASTEXITCODE | Should -Be $ExpectedCode
  }
}
