# VX Bash completion
# This file provides command-line completion for vx commands

_vx_completion() {
    local cur prev words cword
    _init_completion || return

    # Main command
    case ${prev} in
        vx)
            COMPREPLY=($(compgen -W "install uninstall list versions which switch search test init add remove sync lock bundle run analyze dev setup env cache config shell ext x plugin hook services container self-update info migrate auth --help --version" -- "${cur}"))
            ;;
        install|i)
            # Auto-complete tool names and versions
            if [[ ${cur} == *@* ]]; then
                local tool="${cur%@*}"
                COMPREPLY=($(compgen -W "$(vx versions ${tool} 2>/dev/null | grep -v "^=" | head -20)" -P "${tool}@" -- "${cur#*@}"))
            else
                COMPREPLY=($(compgen -W "$(vx list --available 2>/dev/null | awk '{print $1}')" -- "${cur}"))
            fi
            ;;
        uninstall)
            # Auto-complete installed tools
            COMPREPLY=($(compgen -W "$(vx list --installed 2>/dev/null | awk '{print $1}')" -- "${cur}"))
            ;;
        list|ls|versions)
            # Auto-complete tool names
            COMPREPLY=($(compgen -W "$(vx list 2>/dev/null | awk '{print $1}' | tail -n +3)" -- "${cur}"))
            ;;
        switch)
            # Auto-complete tool@version format
            if [[ ${cur} == *@* ]]; then
                local tool="${cur%@*}"
                COMPREPLY=($(compgen -W "$(vx versions ${tool} 2>/dev/null | grep -v "^=" | head -20)" -P "${tool}@" -- "${cur#*@}"))
            else
                COMPREPLY=($(compgen -W "$(vx list --installed 2>/dev/null | awk '{print $1}')" -- "${cur}"))
            fi
            ;;
        search)
            # Search doesn't auto-complete, it takes a query string
            ;;
        test)
            COMPREPLY=($(compgen -W "--all --extension --local --platform-only --functional --install --ci --ci-runtimes --ci-skip --timeout --keep-going --vx-root --temp-root --cleanup --installed --system --detailed --quiet --json --verbose --help" -- "${cur}"))
            ;;
        init)
            COMPREPLY=($(compgen -W "-i --interactive -t --template --tools -f --force --dry-run --list-templates --help" -- "${cur}"))
            ;;
        add)
            # Auto-complete tool names
            COMPREPLY=($(compgen -W "$(vx list --available 2>/dev/null | awk '{print $1}')" -- "${cur}"))
            ;;
        remove|rm)
            # Auto-completes from project tools
            if [ -f "vx.toml" ]; then
                COMPREPLY=($(compgen -W "$(grep -E '^\[tools\.' vx.toml | sed 's/\[tools\.//' | sed 's/\]//' | head -20)" -- "${cur}"))
            fi
            ;;
        sync|lock)
            COMPREPLY=($(compgen -W "--check --dry-run --force --verbose --no-parallel --no-auto-install --update --help" -- "${cur}"))
            ;;
        bundle)
            COMPREPLY=($(compgen -W "create install --help" -- "${cur}"))
            ;;
        run)
            # Auto-complete script names
            if [ -f "vx.toml" ]; then
                COMPREPLY=($(compgen -W "$(awk -F'=' '/^\[scripts\]/{flag=1} flag && /^ *[a-zA-Z_][a-zA-Z0-9_-]* *=/{gsub(/ *=.*/,"",$1); print $1}' vx.toml | head -20)" -- "${cur}"))
            fi
            ;;
        analyze)
            COMPREPLY=($(compgen -W "--json --verbose --help" -- "${cur}"))
            ;;
        dev)
            COMPREPLY=($(compgen -W "--shell --command --no-install --verbose --export --format --info --inherit-system --passenv --help" -- "${cur}"))
            ;;
        setup)
            COMPREPLY=($(compgen -W "--dry-run --force --verbose --no-parallel --no-hooks --ci --help" -- "${cur}"))
            ;;
        env)
            COMPREPLY=($(compgen -W "create delete list show shell activate deactivate export import --help" -- "${cur}"))
            ;;
        cache)
            COMPREPLY=($(compgen -W "info list prune purge --help" -- "${cur}"))
            ;;
        config|cfg)
            COMPREPLY=($(compgen -W "get set list unset edit init --help" -- "${cur}"))
            ;;
        shell)
            COMPREPLY=($(compgen -W "init complete --help" -- "${cur}"))
            ;;
        ext|extension)
            COMPREPLY=($(compgen -W "install uninstall list update enable disable --help" -- "${cur}"))
            ;;
        x)
            COMPREPLY=($(compgen -W "$(vx ext list 2>/dev/null | awk '{print $1}' | tail -n +3)" -- "${cur}"))
            ;;
        plugin)
            COMPREPLY=($(compgen -W "install uninstall list enable disable --help" -- "${cur}"))
            ;;
        hook)
            COMPREPLY=($(compgen -W "list run test add remove enable disable --help" -- "${cur}"))
            ;;
        services)
            COMPREPLY=($(compgen -W "start stop restart status logs --help" -- "${cur}"))
            ;;
        container)
            COMPREPLY=($(compgen -W "build run exec push --help" -- "${cur}"))
            ;;
        self-update)
            COMPREPLY=($(compgen -W "--check --version --token --prerelease --force --help" -- "${cur}"))
            ;;
        info)
            COMPREPLY=($(compgen -W "--json --help" -- "${cur}"))
            ;;
        migrate)
            COMPREPLY=($(compgen -W "--path --dry-run --backup --check --verbose --help" -- "${cur}"))
            ;;
        auth)
            COMPREPLY=($(compgen -W "login logout status show-token --help" -- "${cur}"))
            ;;
        *)
            # Try to provide helpful suggestions based on context
            if [[ ${words[@]} =~ vx\ (dev|env\ shell) ]]; then
                # In vx dev or vx env shell, try to complete project tools
                if [ -f "vx.toml" ]; then
                    COMPREPLY=($(compgen -W "$(grep -E '^\[tools\.' vx.toml | sed 's/\[tools\.//' | sed 's/\]//' | head -20)" -- "${cur}"))
                fi
            fi
            ;;
    esac
}

complete -F _vx_completion vx
