#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration from environment variables
PACKAGES=${PACKAGES:-"all"}
FORCE=${FORCE:-"false"}
DRY_RUN=${DRY_RUN:-"true"}
WAIT_TIME=${WAIT_TIME:-30}

echo -e "${BLUE}🚀 VX Force Publishing Script${NC}"
echo -e "${BLUE}==============================${NC}"

if [ "$DRY_RUN" = "true" ]; then
    echo -e "${YELLOW}⚠️  DRY RUN MODE - No actual publishing${NC}"
else
    echo -e "${RED}🔥 LIVE MODE - Will actually publish to crates.io${NC}"
fi

if [ "$FORCE" = "true" ]; then
    echo -e "${RED}⚡ FORCE MODE - Will attempt to publish even if already published${NC}"
fi

echo ""

# All packages in dependency order
# Level 1: No internal dependencies
declare -a all_packages=(
    "crates/vx-shim"                # No internal dependencies

    # Level 2: Depends only on vx-shim
    "crates/vx-core"                # Depends on vx-shim

    # Level 3: Depends on vx-core
    "crates/vx-tools/vx-tool-go"    # Depends on vx-core
    "crates/vx-tools/vx-tool-rust"  # Depends on vx-core
    "crates/vx-tools/vx-tool-uv"    # Depends on vx-core
    "crates/vx-package-managers/vx-pm-npm" # Depends on vx-core

    # Level 4: Depends on vx-core + vx-pm-npm
    "crates/vx-tools/vx-tool-node"  # Depends on vx-core + vx-pm-npm

    # Level 5: Depends on all tools
    "crates/vx-cli"                 # Depends on vx-core + all tools

    # Level 6: Main package depends on everything
    "."                             # Main package depends on vx-cli
)

# Function to get package name and version
get_package_info() {
    local package_dir=$1

    if [ "$package_dir" = "." ]; then
        package_dir=""
    fi

    local cargo_toml="$package_dir/Cargo.toml"
    if [ ! -f "$cargo_toml" ]; then
        cargo_toml="Cargo.toml"
    fi

    local metadata=$(cargo metadata --no-deps --format-version 1 --manifest-path "$cargo_toml")
    local name=$(echo "$metadata" | grep -o '"name":"[^"]*"' | head -1 | sed 's/"name":"\([^"]*\)"/\1/')
    local version=$(echo "$metadata" | grep -o '"version":"[^"]*"' | head -1 | sed 's/"version":"\([^"]*\)"/\1/')

    echo "$name:$version"
}

# Function to check if package is already published
check_published() {
    local package_name=$1
    local version=$2
    
    echo -e "${BLUE}🔍 Checking if $package_name@$version is already published...${NC}"
    
    if cargo search "$package_name" --limit 1 | grep -q "$package_name = \"$version\""; then
        echo -e "${YELLOW}⚠️  $package_name@$version is already published${NC}"
        return 0
    else
        echo -e "${GREEN}✅ $package_name@$version is not yet published${NC}"
        return 1
    fi
}

# Function to force publish a package
force_publish_package() {
    local package_dir=$1
    local package_info=$(get_package_info "$package_dir")
    local package_name=$(echo "$package_info" | cut -d: -f1)
    local package_version=$(echo "$package_info" | cut -d: -f2)
    
    echo -e "${BLUE}📦 Force processing $package_name@$package_version in $package_dir${NC}"
    
    # Check if already published (unless force mode)
    if [ "$FORCE" != "true" ] && check_published "$package_name" "$package_version"; then
        echo -e "${YELLOW}⏭️  Skipping $package_name (already published, use force=true to override)${NC}"
        return 0
    fi
    
    # Change to package directory
    if [ "$package_dir" != "." ]; then
        cd "$package_dir"
    fi
    
    echo -e "${BLUE}🔨 Building $package_name...${NC}"
    cargo build --release
    
    echo -e "${BLUE}🧪 Testing $package_name...${NC}"
    cargo test
    
    echo -e "${BLUE}🔍 Dry run for $package_name...${NC}"
    cargo publish --dry-run
    
    if [ "$DRY_RUN" = "false" ]; then
        echo -e "${GREEN}🚀 Force publishing $package_name to crates.io...${NC}"
        if [ "$FORCE" = "true" ]; then
            # Try to publish with --allow-dirty flag for force mode
            cargo publish --allow-dirty || cargo publish
        else
            cargo publish
        fi
        echo -e "${GREEN}✅ Successfully published $package_name@$package_version${NC}"
        
        echo -e "${YELLOW}⏳ Waiting ${WAIT_TIME} seconds for crates.io to update...${NC}"
        sleep "$WAIT_TIME"
    else
        echo -e "${YELLOW}🔍 Dry run completed for $package_name${NC}"
    fi
    
    # Return to root directory
    if [ "$package_dir" != "." ]; then
        cd - > /dev/null
    fi
    
    echo ""
}

# Determine which packages to publish
declare -a packages_to_publish=()

if [ "$PACKAGES" = "all" ]; then
    packages_to_publish=("${all_packages[@]}")
else
    # Parse comma-separated package names
    IFS=',' read -ra PACKAGE_NAMES <<< "$PACKAGES"
    for package_name in "${PACKAGE_NAMES[@]}"; do
        package_name=$(echo "$package_name" | xargs)  # trim whitespace
        
        # Find the package directory
        found=false
        for package_dir in "${all_packages[@]}"; do
            pkg_info=$(get_package_info "$package_dir")
            pkg_name=$(echo "$pkg_info" | cut -d: -f1)
            if [ "$pkg_name" = "$package_name" ]; then
                packages_to_publish+=("$package_dir")
                found=true
                break
            fi
        done
        
        if [ "$found" = false ]; then
            echo -e "${RED}❌ Package '$package_name' not found${NC}"
            exit 1
        fi
    done
fi

# Display publishing plan
echo -e "${BLUE}📋 Force publishing plan:${NC}"
for package in "${packages_to_publish[@]}"; do
    package_info=$(get_package_info "$package")
    package_name=$(echo "$package_info" | cut -d: -f1)
    package_version=$(echo "$package_info" | cut -d: -f2)
    echo -e "  ${GREEN}$package_name@$package_version${NC} ($package)"
done
echo ""

# Publish each package
for package in "${packages_to_publish[@]}"; do
    force_publish_package "$package"
done

if [ "$DRY_RUN" = "true" ]; then
    echo -e "${GREEN}🎉 Force publish dry run completed successfully!${NC}"
    echo -e "${YELLOW}💡 To actually publish, run with DRY_RUN=false${NC}"
else
    echo -e "${GREEN}🎉 Force publish completed successfully!${NC}"
    echo -e "${GREEN}🎯 Users can now install with: cargo install vx${NC}"
fi
