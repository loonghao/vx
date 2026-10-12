BeforeAll {
  . (Join-Path $PSScriptRoot '../ensure-msvc-spectre.ps1')
  function Invoke-FakeVsWhere {}
}

Describe 'Visual Studio discovery' {
  BeforeEach {
    Mock Test-Path { $true }
    Mock Invoke-FakeVsWhere {
      $global:LASTEXITCODE = 0
      '[{"installationPath":"C:\\Visual Studio\\18\\Enterprise","installationVersion":"18.10.12217.157","channelId":"VisualStudio.18.Release"}]'
    }
  }

  It 'reads the locator-selected instance without assuming a year or edition' {
    $instance = Get-MsvcInstallation -VsWherePath 'Invoke-FakeVsWhere'
    $instance.installationPath | Should -Be 'C:\Visual Studio\18\Enterprise'
    $instance.installationVersion | Should -Be '18.10.12217.157'
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
    New-Item -ItemType Directory -Force (Join-Path $root 'VC/Tools/MSVC/14.50/lib/x64/spectre') | Out-Null
    Test-MsvcSpectre -InstallationPath $root | Should -BeTrue
  }

  It 'does not treat ARM64-only libraries as x64 support' {
    $root = Join-Path $TestDrive 'VS ARM64'
    New-Item -ItemType Directory -Force (Join-Path $root 'VC/Tools/MSVC/14.50/lib/arm64/spectre') | Out-Null
    Test-MsvcSpectre -InstallationPath $root | Should -BeFalse
  }
}

Describe 'Spectre setup' {
  BeforeEach {
    Mock Get-MsvcInstallation {
      [pscustomobject]@{
        installationPath = 'C:\Visual Studio\18\Enterprise'
        installationVersion = '18.10.12217.157'
        channelId = 'VisualStudio.18.Release'
      }
    }
    Mock Test-Path { $true }
    Mock Test-MsvcSpectre { $true }
    Mock Start-Process { [pscustomobject]@{ ExitCode = 0 } }
  }

  It 'reuses installed libraries without running the installer' {
    (Ensure-MsvcSpectre -InstallerDirectory 'C:\VS Installer').installationPath |
      Should -Be 'C:\Visual Studio\18\Enterprise'
    Should -Invoke Start-Process -Times 0
    Should -Invoke Test-MsvcSpectre -Times 1 -ParameterFilter {
      $InstallationPath -eq 'C:\Visual Studio\18\Enterprise'
    }
  }

  It 'modifies and rechecks the same instance after a successful install' {
    $script:checks = 0
    Mock Test-MsvcSpectre { $script:checks++; $script:checks -gt 1 }
    Ensure-MsvcSpectre -InstallerDirectory 'C:\VS Installer' | Out-Null
    Should -Invoke Start-Process -Times 1 -ParameterFilter {
      $FilePath -eq 'C:\VS Installer\setup.exe' -and $Wait -and $PassThru -and
      $ArgumentList -contains '"C:\Visual Studio\18\Enterprise"' -and
      $ArgumentList -contains 'VisualStudio.18.Release' -and
      $ArgumentList -notcontains '--wait' -and
      $ArgumentList -contains 'Microsoft.VisualStudio.Component.VC.Runtimes.x86.x64.Spectre'
    }
    Should -Invoke Test-MsvcSpectre -Times 2 -ParameterFilter {
      $InstallationPath -eq 'C:\Visual Studio\18\Enterprise'
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
