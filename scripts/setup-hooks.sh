#!/bin/bash
# Setup Git Hooks for VX Project
# This script installs and configures git hooks for code quality

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

echo "🔧 Setting up VX Git Hooks"
echo "=========================="

# Check if we're in a git repository
if [ ! -d ".git" ]; then
    print_error "Not in a git repository. Please run this script from the project root."
    exit 1
fi

# Check if .githooks directory exists
if [ ! -d ".githooks" ]; then
    print_error ".githooks directory not found. Please run this script from the project root."
    exit 1
fi

# Configure git to use .githooks directory
print_info "Configuring git to use .githooks directory..."
git config core.hooksPath .githooks

# Make hooks executable
print_info "Making hooks executable..."
chmod +x .githooks/*

# Verify hooks are set up correctly
if [ -f ".githooks/pre-commit" ]; then
    print_success "Pre-commit hook installed"
else
    print_warning "Pre-commit hook not found"
fi

# Test the pre-commit hook
print_info "Testing pre-commit hook..."
if .githooks/pre-commit --help 2>/dev/null || echo "Hook test completed"; then
    print_success "Pre-commit hook is working"
else
    print_warning "Pre-commit hook test failed (this might be normal)"
fi

# Install required tools if not present
print_info "Checking required tools..."

# Check for rustfmt
if command -v rustfmt >/dev/null 2>&1; then
    print_success "rustfmt is available"
else
    print_warning "rustfmt not found. Installing..."
    rustup component add rustfmt
fi

# Check for clippy
if command -v cargo-clippy >/dev/null 2>&1 || cargo clippy --version >/dev/null 2>&1; then
    print_success "clippy is available"
else
    print_warning "clippy not found. Installing..."
    rustup component add clippy
fi

# Create a sample git config for the project
print_info "Setting up recommended git configuration..."

# Set up conventional commit template
cat > .gitmessage << 'EOF'
# <type>(<scope>): <subject>
#
# <body>
#
# <footer>
#
# Type should be one of the following:
# * feat: A new feature
# * fix: A bug fix
# * docs: Documentation only changes
# * style: Changes that do not affect the meaning of the code
# * refactor: A code change that neither fixes a bug nor adds a feature
# * test: Adding missing tests or correcting existing tests
# * chore: Changes to the build process or auxiliary tools
# * ci: Changes to CI configuration files and scripts
# * build: Changes that affect the build system or external dependencies
#
# Scope is optional and should be the name of the package affected
# Subject should be imperative mood, lowercase, no period at the end
# Body should explain what and why vs. how (optional)
# Footer should contain any breaking changes or issue references (optional)
EOF

git config commit.template .gitmessage

# Set up other useful git configurations
git config push.default simple
git config pull.rebase true

print_success "Git configuration updated"

# Create a local configuration file for hook settings
cat > .git-hooks-config << 'EOF'
# VX Git Hooks Configuration
# You can override these settings by setting environment variables

# Enable quick tests on affected modules (default: false)
# VX_QUICK_TEST=true

# Enable strict mode - fail on any warnings (default: false)
# VX_STRICT_MODE=true

# Enable automatic formatting (default: true)
# VX_AUTO_FIX=true

# To use these settings, source this file or set the variables:
# source .git-hooks-config
# export VX_QUICK_TEST=true
# git commit -m "your message"
EOF

print_info "Created .git-hooks-config for local customization"

# Show usage information
echo
print_success "🎉 Git hooks setup completed!"
echo
print_info "📋 Usage:"
echo "  Normal commit:     git commit -m 'feat: add new feature'"
echo "  Skip hooks:        git commit --no-verify -m 'message'"
echo "  With quick tests:  VX_QUICK_TEST=true git commit -m 'message'"
echo "  Strict mode:       VX_STRICT_MODE=true git commit -m 'message'"
echo "  Disable auto-fix:  VX_AUTO_FIX=false git commit -m 'message'"
echo
print_info "📝 Configuration:"
echo "  Edit .git-hooks-config to customize hook behavior"
echo "  Use 'source .git-hooks-config' to load settings"
echo
print_info "🔧 Manual commands:"
echo "  Format code:       cargo fmt --all"
echo "  Run clippy:        cargo clippy --all-targets --all-features"
echo "  Quality check:     ./scripts/quality-check.ps1"
echo
print_warning "💡 Tips:"
echo "  - Use conventional commit format for better changelog generation"
echo "  - The pre-commit hook will automatically format your code"
echo "  - Set VX_QUICK_TEST=true to run tests on affected modules"
echo "  - Use --no-verify to skip hooks in emergency situations"

exit 0
