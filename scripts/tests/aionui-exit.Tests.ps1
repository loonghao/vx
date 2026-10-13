# Use a real native .cmd process to exercise LASTEXITCODE through Tee-Object.
Describe 'AionUi native command failures' -Skip:(-not $IsWindows) {
  BeforeEach {
    $script:originalPath = $env:PATH
    $script:originalFailureCommand = $env:VX_AION_TEST_FAILURE_COMMAND
    $script:originalFailureCode = $env:VX_AION_TEST_FAILURE_CODE
    $script:originalCallsFile = $env:VX_AION_TEST_CALLS
    $script:originalGitRevision = $env:VX_AION_TEST_GIT_REVISION
    $script:originalLocation = (Get-Location).Path
    $script:fixture = Join-Path $TestDrive ([guid]::NewGuid().ToString())
    $bin = Join-Path $script:fixture 'bin'
    New-Item -ItemType Directory -Force -Path $bin | Out-Null
    $script:example = Join-Path $script:fixture 'aionui.build.ps1'
    Copy-Item (Join-Path $PSScriptRoot '../../examples/msvc/aionui.build.ps1') $script:example
    $pin = [regex]::Match((Get-Content $script:example -Raw), '\$repoCommit = "([a-f0-9]{40})"')
    $pin.Success | Should -BeTrue
    $script:repoCommit = $pin.Groups[1].Value
    $repo = Join-Path $script:fixture "work/aionui-$($script:repoCommit.Substring(0, 12))"
    New-Item -ItemType Directory -Force -Path $repo | Out-Null
    $script:manifest = Join-Path $repo 'package.json'
    '{"dependencies":{"@aionui/web-host":"workspace:*"},"resolutions":{"@codemirror/language":"6.12.3"}}' |
      Set-Content $script:manifest
    $script:originalManifest = Get-Content $script:manifest -Raw
    $env:VX_AION_TEST_GIT_REVISION = $script:repoCommit
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
    @'
@echo off
if "%~3"=="rev-parse" echo %VX_AION_TEST_GIT_REVISION%
exit /b 0
'@ | Set-Content (Join-Path $bin 'git.cmd') -Encoding ascii
    $env:PATH = "$bin;$env:PATH"
    (Get-Command vx).Source | Should -Be (Join-Path $bin 'vx.cmd')
    (Get-Command git).Source | Should -Be (Join-Path $bin 'git.cmd')
    # The script must check native status even with PowerShell's default policy.
    $PSNativeCommandUseErrorActionPreference = $false
  }

  AfterEach {
    $env:PATH = $script:originalPath
    $env:VX_AION_TEST_FAILURE_COMMAND = $script:originalFailureCommand
    $env:VX_AION_TEST_FAILURE_CODE = $script:originalFailureCode
    $env:VX_AION_TEST_CALLS = $script:originalCallsFile
    $env:VX_AION_TEST_GIT_REVISION = $script:originalGitRevision
  }

  It 'stops after <Command> exits <Code> without claiming success' -TestCases @(
    @{ Command = 'bun install'; Code = 31; ExpectedCalls = 4 },
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
    $calls[3] | Should -BeExactly 'bun install --frozen-lockfile'
    $calls[-1] | Should -BeLike 'node -e*'
    $calls[-1] | Should -Match 'new Database'
    $calls[-1] | Should -Not -Match 'node-pty'
    (Get-Content $script:manifest -Raw) | Should -BeExactly $script:originalManifest
    (Get-Content $script:logFile -Raw) | Should -Match 'AionUi test passed'
    (Get-Location).Path | Should -Be $script:originalLocation
  }

  It 'rejects an unexpected upstream revision before running package commands' {
    $env:VX_AION_TEST_GIT_REVISION = '0000000000000000000000000000000000000000'

    { & $script:example | Out-Null } | Should -Throw '*checkout must be at*'

    Test-Path $env:VX_AION_TEST_CALLS | Should -BeFalse
    (Get-Content $script:logFile -Raw) | Should -Not -Match 'AionUi test passed'
  }
}
