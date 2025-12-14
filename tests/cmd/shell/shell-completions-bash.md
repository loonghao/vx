# Test: vx shell completions bash

Verify that `vx shell completions bash` generates bash completion script.

```console
$ vx shell completions bash
[..]Bash[..]
# Source this file or add it to /etc/bash_completion.d/

_vx_completion() {
    local cur prev words cword
    _init_completion || return

    case $prev in
        install|remove|switch|fetch)
            # Complete with available tools
            COMPREPLY=($(compgen -W "node npm npx go cargo uv uvx python" -- "$cur"))
            return
            ;;
        --template)
            # Complete with available templates
            COMPREPLY=($(compgen -W "node python rust go fullstack minimal" -- "$cur"))
            return
            ;;
        --format)
            # Complete with output formats
            COMPREPLY=($(compgen -W "table json yaml" -- "$cur"))
            return
            ;;
        --category)
            # Complete with categories
            COMPREPLY=($(compgen -W "javascript python rust go utility" -- "$cur"))
            return
            ;;
    esac

    case ${words[1]} in
        venv)
            case ${words[2]} in
                activate|remove|use)
                    # Complete with venv names
                    local venvs=$(vx venv list --names-only 2>/dev/null || echo "")
                    COMPREPLY=($(compgen -W "$venvs" -- "$cur"))
                    return
                    ;;
                *)
                    COMPREPLY=($(compgen -W "create list activate remove current" -- "$cur"))
                    return
                    ;;
            esac
            ;;
        config)
            COMPREPLY=($(compgen -W "show set get reset edit" -- "$cur"))
            return
            ;;
        *)
            # Complete with main commands
            COMPREPLY=($(compgen -W "install remove list update search sync init cleanup stats venv config global plugin shell-init completion version help" -- "$cur"))
            return
            ;;
    esac
}

complete -F _vx_completion vx


```
