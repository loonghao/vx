#compdef vx
# VX Zsh completion

_vx() {
    local -a commands subcommands
    commands=(
        'install:Install tool(s)'
        'uninstall:Uninstall tool versions'
        'list:List installed tools'
        'versions:Show available versions for a tool'
        'which:Show which tool version is being used'
        'switch:Switch to a different version of a tool'
        'search:Search available tools'
        'test:Test runtime availability'
        'init:Initialize vx configuration'
        'add:Add a tool to project configuration'
        'remove:Remove a tool from project configuration'
        'sync:Sync project tools from vx.toml'
        'lock:Generate or update vx.lock'
        'bundle:Create offline development environment bundle'
        'run:Run a script defined in vx.toml'
        'analyze:Analyze project dependencies, scripts, and tools'
        'dev:Enter development environment with all project tools'
        'setup:Setup development environment'
        'env:Environment management'
        'cache:Cache management commands'
        'config:Configuration management'
        'shell:Shell integration commands'
        'ext:Extension management'
        'x:Execute an extension command'
        'plugin:Plugin management commands'
        'hook:Execute or manage lifecycle hooks'
        'services:Manage development services'
        'container:Container and Dockerfile management'
        'self-update:Update vx itself'
        'info:Show system information'
        'migrate:Migrate configuration and data'
        'auth:Authentication for external services'
    )

    if (( CURRENT == 2 )); then
        _describe 'command' commands
    else
        case $words[2] in
            install|i)
                if [[ $words[CURRENT-1] == *@* ]]; then
                    local tool=${words[CURRENT-1]%%@*}
                    _values 'version' ${(f)"$(vx versions $tool 2>/dev/null | grep -v '^=' | head -20)"}
                else
                    _values 'tool' ${(f)"$(vx list --available 2>/dev/null | awk '{print $1}')"}
                fi
                ;;
            uninstall|switch)
                _values 'tool' ${(f)"$(vx list --installed 2>/dev/null | awk '{print $1}')"}
                ;;
            list|ls|versions)
                _values 'tool' ${(f)"$(vx list 2>/dev/null | awk '{print $1}' | tail -n +3)"}
                ;;
            add)
                _values 'tool' ${(f)"$(vx list --available 2>/dev/null | awk '{print $1}')"}
                ;;
            remove|rm)
                if [[ -f vx.toml ]]; then
                    _values 'tool' ${(f)"$(grep -E '^\[tools\.' vx.toml | sed 's/\[tools\.//' | sed 's/\]//')"}
                fi
                ;;
            run)
                if [[ -f vx.toml ]]; then
                    _values 'script' ${(f)"$(awk -F'=' '/^\[scripts\]/{flag=1} flag && /^ *[a-zA-Z_][a-zA-Z0-9_-]* *=/{gsub(/ *=.*/,"",$1); print $1}' vx.toml)"}
                fi
                ;;
            env)
                subcommands=(
                    'create:Create a new environment'
                    'delete:Delete an environment'
                    'list:List all environments'
                    'show:Show environment details'
                    'shell:Enter environment shell'
                    'activate:Activate environment'
                    'deactivate:Deactivate environment'
                    'export:Export environment configuration'
                    'import:Import environment configuration'
                )
                if (( CURRENT == 3 )); then
                    _describe 'subcommand' subcommands
                fi
                ;;
            cache)
                subcommands=(
                    'info:Show cache statistics'
                    'list:List cached items'
                    'prune:Prune expired entries'
                    'purge:Purge all cache data'
                )
                if (( CURRENT == 3 )); then
                    _describe 'subcommand' subcommands
                fi
                ;;
            config|cfg)
                subcommands=(
                    'get:Get configuration value'
                    'set:Set configuration value'
                    'list:List all configuration'
                    'unset:Unset configuration value'
                    'edit:Edit configuration file'
                    'init:Initialize configuration'
                )
                if (( CURRENT == 3 )); then
                    _describe 'subcommand' subcommands
                fi
                ;;
            shell)
                subcommands=(
                    'init:Initialize shell integration'
                    'complete:Generate completion script'
                )
                if (( CURRENT == 3 )); then
                    _describe 'subcommand' subcommands
                fi
                ;;
            ext|extension)
                subcommands=(
                    'install:Install extension'
                    'uninstall:Uninstall extension'
                    'list:List extensions'
                    'update:Update extension'
                    'enable:Enable extension'
                    'disable:Disable extension'
                )
                if (( CURRENT == 3 )); then
                    _describe 'subcommand' subcommands
                fi
                ;;
            x)
                _values 'extension' ${(f)"$(vx ext list 2>/dev/null | awk '{print $1}' | tail -n +3)"}
                ;;
            plugin)
                subcommands=(
                    'install:Install plugin'
                    'uninstall:Uninstall plugin'
                    'list:List plugins'
                    'enable:Enable plugin'
                    'disable:Disable plugin'
                )
                if (( CURRENT == 3 )); then
                    _describe 'subcommand' subcommands
                fi
                ;;
            auth)
                subcommands=(
                    'login:Login to service'
                    'logout:Logout from service'
                    'status:Show authentication status'
                    'show-token:Show authentication token'
                )
                if (( CURRENT == 3 )); then
                    _describe 'subcommand' subcommands
                fi
                ;;
            bundle)
                subcommands=(
                    'create:Create bundle'
                    'install:Install bundle'
                )
                if (( CURRENT == 3 )); then
                    _describe 'subcommand' subcommands
                fi
                ;;
            *)
                # Global options
                _arguments -s \
                    '--help[Show help]' \
                    '--version[Show version]' \
                    '--use-system-path[Use system PATH]' \
                    '--inherit-env[Inherit system environment]' \
                    '--cache-mode[Cache mode]' \
                    {-v,--verbose}'[Verbose output]' \
                    '--debug[Debug output]'
                ;;
        esac
    fi
}

_vx "$@"
