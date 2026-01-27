# VX PowerShell Completion
# This file provides command-line completion for vx commands

# Register the argument completer
Register-ArgumentCompleter -Native -CommandName vx -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    # Get the command parts
    $commandElements = $commandAst.CommandElements
    $command = @($commandElements)[0]

    # Main commands
    $mainCommands = @(
        'install', 'uninstall', 'list', 'versions', 'which', 'switch',
        'search', 'test', 'init', 'add', 'remove', 'sync', 'lock',
        'bundle', 'run', 'analyze', 'dev', 'setup', 'env', 'cache',
        'config', 'shell', 'ext', 'x', 'plugin', 'hook', 'services',
        'container', 'self-update', 'info', 'migrate', 'auth'
    )

    # If we're still typing the main command, complete from main commands
    if ($commandElements.Count -eq 1) {
        return $mainCommands | Where-Object { $_ -like "$wordToComplete*" }
    }

    # Get the current command
    $currentCommand = $commandElements[1].Value
    $previousElement = $commandElements[$commandElements.Count - 1]

    # Handle subcommands
    switch ($currentCommand) {
        'install' {
            # Complete tool names or versions
            if ($wordToComplete -match '@') {
                $tool = $wordToComplete -replace '@.*', ''
                $versions = vx versions $tool 2>$null | Select-Object -Skip 1 | Select-Object -First 20
                return $versions | ForEach-Object { "$tool@$_" }
            } else {
                $tools = vx list --available 2>$null | Select-Object -Skip 1 | ForEach-Object { ($_ -split '\s+')[0] }
                return $tools | Where-Object { $_ -like "$wordToComplete*" }
            }
        }
        'uninstall' {
            $tools = vx list --installed 2>$null | Select-Object -Skip 1 | ForEach-Object { ($_ -split '\s+')[0] }
            return $tools | Where-Object { $_ -like "$wordToComplete*" }
        }
        'list' {
            $tools = vx list 2>$null | Select-Object -Skip 3 | ForEach-Object { ($_ -split '\s+')[0] }
            return $tools | Where-Object { $_ -like "$wordToComplete*" }
        }
        'versions' {
            $tools = vx list 2>$null | Select-Object -Skip 3 | ForEach-Object { ($_ -split '\s+')[0] }
            return $tools | Where-Object { $_ -like "$wordToComplete*" }
        }
        'switch' {
            if ($wordToComplete -match '@') {
                $tool = $wordToComplete -replace '@.*', ''
                $versions = vx versions $tool 2>$null | Select-Object -Skip 1 | Select-Object -First 20
                return $versions | ForEach-Object { "$tool@$_" }
            } else {
                $tools = vx list --installed 2>$null | Select-Object -Skip 1 | ForEach-Object { ($_ -split '\s+')[0] }
                return $tools | Where-Object { $_ -like "$wordToComplete*" }
            }
        }
        'add' {
            $tools = vx list --available 2>$null | Select-Object -Skip 1 | ForEach-Object { ($_ -split '\s+')[0] }
            return $tools | Where-Object { $_ -like "$wordToComplete*" }
        }
        'remove' {
            if (Test-Path 'vx.toml') {
                $tools = Select-String -Path 'vx.toml' -Pattern '^\[tools\.' |
                         ForEach-Object { $_.Line -replace '\[tools\.', '' -replace '\]', '' }
                return $tools | Where-Object { $_ -like "$wordToComplete*" }
            }
        }
        'run' {
            if (Test-Path 'vx.toml') {
                $content = Get-Content 'vx.toml'
                $inScripts = $false
                $scripts = @()
                foreach ($line in $content) {
                    if ($line -match '^\[scripts\]') { $inScripts = $true; continue }
                    if ($inScripts -and $line -match '^ *[a-zA-Z_][a-zA-Z0-9_-]* *=') {
                        $scripts += ($line -split '=')[0].Trim()
                    }
                }
                return $scripts | Where-Object { $_ -like "$wordToComplete*" }
            }
        }
        'env' {
            $envSubcommands = @('create', 'delete', 'list', 'show', 'shell', 'activate', 'deactivate', 'export', 'import')
            if ($commandElements.Count -eq 2) {
                return $envSubcommands | Where-Object { $_ -like "$wordToComplete*" }
            }
        }
        'cache' {
            $cacheSubcommands = @('info', 'list', 'prune', 'purge')
            if ($commandElements.Count -eq 2) {
                return $cacheSubcommands | Where-Object { $_ -like "$wordToComplete*" }
            }
        }
        'config' {
            $configSubcommands = @('get', 'set', 'list', 'unset', 'edit', 'init')
            if ($commandElements.Count -eq 2) {
                return $configSubcommands | Where-Object { $_ -like "$wordToComplete*" }
            }
        }
        'shell' {
            $shellSubcommands = @('init', 'complete')
            if ($commandElements.Count -eq 2) {
                return $shellSubcommands | Where-Object { $_ -like "$wordToComplete*" }
            }
        }
        'ext' {
            $extSubcommands = @('install', 'uninstall', 'list', 'update', 'enable', 'disable')
            if ($commandElements.Count -eq 2) {
                return $extSubcommands | Where-Object { $_ -like "$wordToComplete*" }
            }
        }
        'x' {
            $extensions = vx ext list 2>$null | Select-Object -Skip 3 | ForEach-Object { ($_ -split '\s+')[0] }
            return $extensions | Where-Object { $_ -like "$wordToComplete*" }
        }
        'plugin' {
            $pluginSubcommands = @('install', 'uninstall', 'list', 'enable', 'disable')
            if ($commandElements.Count -eq 2) {
                return $pluginSubcommands | Where-Object { $_ -like "$wordToComplete*" }
            }
        }
        'auth' {
            $authSubcommands = @('login', 'logout', 'status', 'show-token')
            if ($commandElements.Count -eq 2) {
                return $authSubcommands | Where-Object { $_ -like "$wordToComplete*" }
            }
        }
        'bundle' {
            $bundleSubcommands = @('create', 'install')
            if ($commandElements.Count -eq 2) {
                return $bundleSubcommands | Where-Object { $_ -like "$wordToComplete*" }
            }
        }
        'services' {
            $serviceSubcommands = @('start', 'stop', 'restart', 'status', 'logs')
            if ($commandElements.Count -eq 2) {
                return $serviceSubcommands | Where-Object { $_ -like "$wordToComplete*" }
            }
        }
        'container' {
            $containerSubcommands = @('build', 'run', 'exec', 'push')
            if ($commandElements.Count -eq 2) {
                return $containerSubcommands | Where-Object { $_ -like "$wordToComplete*" }
            }
        }
        'hook' {
            $hookSubcommands = @('list', 'run', 'test', 'add', 'remove', 'enable', 'disable')
            if ($commandElements.Count -eq 2) {
                return $hookSubcommands | Where-Object { $_ -like "$wordToComplete*" }
            }
        }
    }

    # Default: complete options
    @('--help', '--version', '--use-system-path', '--inherit-env', '--cache-mode', '-v', '--verbose', '--debug') |
        Where-Object { $_ -like "$wordToComplete*" }
}

# Add alias for install command
Set-Alias -Name i -Value install -ErrorAction SilentlyContinue
Set-Alias -Name ls -Value list -ErrorAction SilentlyContinue
Set-Alias -Name rm -Value remove -ErrorAction SilentlyContinue
Set-Alias -Name x -Value x -ErrorAction SilentlyContinue
Set-Alias -Name where -Value which -ErrorAction SilentlyContinue
Set-Alias -Name extension -Value ext -ErrorAction SilentlyContinue
Set-Alias -Name cfg -Value config -ErrorAction SilentlyContinue
Set-Alias -Name self-update -Value self-update -ErrorAction SilentlyContinue
