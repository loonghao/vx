#!/bin/bash
# VX Shell Environment Initialization for Bash
# This script is embedded into vx-env binary at compile time

VX_PROJECT_NAME="${VX_PROJECT_NAME:-vx}"
VX_TOOLS="${VX_TOOLS:-}"

# Set custom prompt
export PS1="(${VX_PROJECT_NAME}[vx]) \w\$ "

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
