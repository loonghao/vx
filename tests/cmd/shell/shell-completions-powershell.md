# Test: vx shell completions powershell

Verify that `vx shell completions powershell` generates PowerShell completion script.

```console
$ vx shell completions powershell
# VX PowerShell Completion
[..]
Register-ArgumentCompleter -Native -CommandName vx -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commands = @(
        'install', 'remove', 'list', 'update', 'search', 'sync', 'init', 'cleanup', 'stats',
        'venv', 'config', 'global', 'plugin', 'shell-init', 'completion', 'version', 'help'
    )

    $tools = @('node', 'npm', 'npx', 'go', 'cargo', 'uv', 'uvx', 'python')
    $shells = @('bash', 'zsh', 'fish', 'powershell')
    $formats = @('table', 'json', 'yaml')
    $templates = @('node', 'python', 'rust', 'go', 'fullstack', 'minimal')

    $tokens = $commandAst.CommandElements
    $command = $tokens[1].Value

    switch ($command) {
        { $_ -in @('install', 'remove', 'switch', 'fetch') } {
            $tools | Where-Object { $_ -like "$wordToComplete*" }
        }
        'venv' {
            if ($tokens.Count -eq 2) {
                @('create', 'list', 'activate', 'remove', 'current') | Where-Object { $_ -like "$wordToComplete*" }
            }
        }
        'config' {
            if ($tokens.Count -eq 2) {
                @('show', 'set', 'get', 'reset', 'edit') | Where-Object { $_ -like "$wordToComplete*" }
            }
        }
        { $_ -in @('completion', 'shell-init') } {
            $shells | Where-Object { $_ -like "$wordToComplete*" }
        }
        default {
            if ($tokens.Count -eq 1) {
                $commands | Where-Object { $_ -like "$wordToComplete*" }
            }
        }
    }
}


```
