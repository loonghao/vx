#!/bin/bash
# VX Shell Environment Initialization for Bash
# This script is embedded into vx-env binary at compile time

VX_PROJECT_NAME="${VX_PROJECT_NAME:-vx}"
VX_TOOLS="${VX_TOOLS:-}"

# Set custom prompt
export PS1="(${VX_PROJECT_NAME}[vx]) \w\$ "

# Configure command history
# Store history in vx directory to avoid conflicts
VX_DATA_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/vx"
if [ ! -d "$VX_DATA_DIR" ]; then
    mkdir -p "$VX_DATA_DIR"
fi

export HISTFILE="$VX_DATA_DIR/bash_history"
export HISTSIZE=10000
export HISTFILESIZE=20000
export HISTCONTROL=ignoreboth:erasedups

# Save multi-line commands as one command
shopt -s cmdhist

# Append to history file instead of overwriting
shopt -s histappend

# Load vx completion script if it exists
VX_COMPLETION_SCRIPT="${VX_DATA_DIR}/vx_completion.bash"
if [ -f "$VX_COMPLETION_SCRIPT" ]; then
    source "$VX_COMPLETION_SCRIPT"
fi

# Show welcome message
echo ""
echo -e "\033[32mVX Shell Environment\033[0m"
echo -e "\033[36mProject: ${VX_PROJECT_NAME}\033[0m"
if [ -n "$VX_TOOLS" ]; then
    echo -e "\033[36mTools: ${VX_TOOLS}\033[0m"
fi
echo ""

# Define helpful aliases
alias vx-tools='echo "Configured tools: ${VX_TOOLS}"'
alias vx-exit='exit'
alias vx-history='history'
alias vx-clear-history='history -c && > "$HISTFILE" && echo "History cleared"'
