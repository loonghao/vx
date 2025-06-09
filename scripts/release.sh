#!/bin/bash
# Release script for vx
# This script helps create and publish releases

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're on main branch
check_branch() {
    local current_branch
    current_branch=$(git branch --show-current)
    
    if [[ "$current_branch" != "main" ]]; then
        log_error "Must be on main branch to create a release. Current branch: $current_branch"
        exit 1
    fi
}

# Check if working directory is clean
check_clean() {
    if [[ -n $(git status --porcelain) ]]; then
        log_error "Working directory is not clean. Please commit or stash changes."
        git status --short
        exit 1
    fi
}

# Get current version from Cargo.toml
get_current_version() {
    grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/'
}

# Update version in Cargo.toml
update_version() {
    local new_version="$1"
    
    log_info "Updating version to $new_version in Cargo.toml"
    sed -i.bak "s/^version = \".*\"/version = \"$new_version\"/" Cargo.toml
    rm Cargo.toml.bak
}

# Create and push tag
create_tag() {
    local version="$1"
    local tag="v$version"
    
    log_info "Creating tag $tag"
    git add Cargo.toml
    git commit -m "chore: bump version to $version"
    git tag -a "$tag" -m "Release $tag"
    
    log_info "Pushing tag $tag"
    git push origin main
    git push origin "$tag"
}

# Main release function
release() {
    local version_type="$1"
    
    if [[ -z "$version_type" ]]; then
        echo "Usage: $0 <patch|minor|major|VERSION>"
        echo "Examples:"
        echo "  $0 patch    # 0.1.0 -> 0.1.1"
        echo "  $0 minor    # 0.1.0 -> 0.2.0"
        echo "  $0 major    # 0.1.0 -> 1.0.0"
        echo "  $0 1.2.3    # Set specific version"
        exit 1
    fi
    
    check_branch
    check_clean
    
    local current_version
    current_version=$(get_current_version)
    log_info "Current version: $current_version"
    
    local new_version
    case "$version_type" in
        patch)
            new_version=$(echo "$current_version" | awk -F. '{$NF = $NF + 1;} 1' | sed 's/ /./g')
            ;;
        minor)
            new_version=$(echo "$current_version" | awk -F. '{$(NF-1) = $(NF-1) + 1; $NF = 0;} 1' | sed 's/ /./g')
            ;;
        major)
            new_version=$(echo "$current_version" | awk -F. '{$1 = $1 + 1; $2 = 0; $3 = 0;} 1' | sed 's/ /./g')
            ;;
        *)
            # Assume it's a specific version
            new_version="$version_type"
            ;;
    esac
    
    log_info "New version: $new_version"
    
    # Confirm with user
    read -p "Create release $new_version? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "Release cancelled"
        exit 0
    fi
    
    # Update version and create tag
    update_version "$new_version"
    create_tag "$new_version"
    
    log_info "Release $new_version created successfully!"
    log_info "GitHub Actions will automatically build and publish the release."
    log_info "Check the progress at: https://github.com/loonghao/vx/actions"
}

# Run the release
release "$@"
