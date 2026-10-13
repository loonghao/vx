# Use a real native .cmd process to exercise LASTEXITCODE through Tee-Object.
Describe 'AionUi native command failures' -Skip:(-not $IsWindows) {
  BeforeEach {
    $script:originalPath = $env:PATH
    $script:originalFailureCommand = $env:VX_AION_TEST_FAILURE_COMMAND
    $script:originalFailureCode = $env:VX_AION_TEST_FAILURE_CODE
    $script:originalCallsFile = $env:VX_AION_TEST_CALLS
    $script:originalLocation = (Get-Location).Path
    $script:fixture = Join-Path $TestDrive ([guid]::NewGuid().ToString())
    $bin = Join-Path $script:fixture 'bin'
    $repo = Join-Path $script:fixture 'work/aionui'
    New-Item -ItemType Directory -Force -Path $bin, $repo | Out-Null
    '{}' | Set-Content (Join-Path $repo 'package.json')
    $script:example = Join-Path $script:fixture 'aionui.build.ps1'
    Copy-Item (Join-Path $PSScriptRoot '../../examples/msvc/aionui.build.ps1') $script:example
    $script:logFile = Join-Path $script:fixture 'logs/aionui.log'
    $env:VX_AION_TEST_CALLS = Join-Path $script:fixture 'calls.txt'
    $env:VX_AION_TEST_FAILURE_COMMAND = ''
    $env:VX_AION_TEST_FAILURE_CODE = '0'
    @'
@echo off
echo %*>>"%VX_AION_TEST_CALLS%"
echo simulated native stdout: %*
if "%~1 %~2"=="%VX_AION_TEST_FAILURE_COMMAND%" (
  echo simulated native stderr: %VX_AION_TEST_FAILURE_CODE% 1>&2
  exit /b %VX_AION_TEST_FAILURE_CODE%
)
exit /b 0
'@ | Set-Content (Join-Path $bin 'vx.cmd') -Encoding ascii
    $env:PATH = "$bin;$env:PATH"
    (Get-Command vx).Source | Should -Be (Join-Path $bin 'vx.cmd')
    # The script must check native status even with PowerShell's default policy.
    $PSNativeCommandUseErrorActionPreference = $false
  }

  AfterEach {
    $env:PATH = $script:originalPath
    $env:VX_AION_TEST_FAILURE_COMMAND = $script:originalFailureCommand
    $env:VX_AION_TEST_FAILURE_CODE = $script:originalFailureCode
    $env:VX_AION_TEST_CALLS = $script:originalCallsFile
  }

  It 'stops after <Command> exits <Code> without claiming success' -TestCases @(
    @{ Command = 'npm install'; Code = 31; ExpectedCalls = 4 },
    @{ Command = 'npm rebuild'; Code = 32; ExpectedCalls = 5 },
    @{ Command = 'node -e'; Code = 33; ExpectedCalls = 6 }
  ) {
    param($Command, $Code, $ExpectedCalls)
    $env:VX_AION_TEST_FAILURE_COMMAND = $Command
    $env:VX_AION_TEST_FAILURE_CODE = "$Code"

    { & $script:example | Out-Null } | Should -Throw "*failed with exit code $Code*"

    $LASTEXITCODE | Should -Be $Code
    $calls = @(Get-Content $env:VX_AION_TEST_CALLS)
    $calls.Count | Should -Be $ExpectedCalls
    $calls[-1] | Should -BeLike "$Command*"
    $log = Get-Content $script:logFile -Raw
    $log | Should -Match "simulated native stderr: $Code"
    $log | Should -Not -Match 'AionUi test passed'
    (Get-Location).Path | Should -Be $script:originalLocation
  }

  It 'reports success only after all native commands succeed' {
    & $script:example | Out-Null

    $LASTEXITCODE | Should -Be 0
    $calls = @(Get-Content $env:VX_AION_TEST_CALLS)
    $calls.Count | Should -Be 6
    $calls[-1] | Should -BeLike 'node -e*'
    (Get-Content $script:logFile -Raw) | Should -Match 'AionUi test passed'
    (Get-Location).Path | Should -Be $script:originalLocation
  }
}
