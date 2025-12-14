# Test: vx shell init bash

Verify that `vx shell init bash` generates shell initialization script.

```console
$ vx shell init bash
# VX Shell Integration for Bash
# Add this to your ~/.bashrc or source it directly

# Set VX environment variables
export VX_HOME="[..]"
export VX_SHELL="bash"

# Add VX bin directory to PATH if not already present
if [[ ":$PATH:" != *":$VX_HOME/bin:"* ]]; then
    export PATH="$VX_HOME/bin:$PATH"
fi

# VX project detection function
__vx_detect_project() {
    local dir="$PWD"
    while [[ "$dir" != "/" ]]; do
        if [[ -f "$dir/.vx.toml" ]]; then
            export VX_PROJECT_ROOT="$dir"
            return 0
        fi
        dir="$(dirname "$dir")"
    done
    unset VX_PROJECT_ROOT
    return 1
}

# Auto-sync on directory change
__vx_auto_sync() {
    if __vx_detect_project && [[ -f "$VX_PROJECT_ROOT/.vx.toml" ]]; then
        if command -v vx >/dev/null 2>&1; then
            vx sync --check --quiet 2>/dev/null || true
        fi
    fi
}

# Hook into cd command
__vx_original_cd=$(declare -f cd)
cd() {
    builtin cd "$@"
    __vx_auto_sync
}

# Initialize on shell startup
__vx_auto_sync

# VX prompt integration (optional)
__vx_prompt() {
    if [[ -n "$VX_PROJECT_ROOT" ]]; then
        echo "[vx]"
    fi
}

# Uncomment to add VX info to prompt
# PS1="$(__vx_prompt)$PS1"


```
