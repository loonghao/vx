#!/bin/bash
set -euo pipefail

# Create release archives and checksums
# Usage: ./scripts/create-archives.sh

echo "📦 Creating release archives and checksums..."

# Function to find and package binary
package_binary() {
    local platform="$1"
    local artifact_dir="$2"
    local binary_name="$3"
    local archive_name="$4"
    local archive_type="$5"
    
    echo "🔍 Looking for $platform binary in $artifact_dir..."
    
    if [ -d "$artifact_dir" ]; then
        echo "📁 Contents of $artifact_dir:"
        ls -la "$artifact_dir"
        
        # Find the binary file
        local binary_file=$(find "$artifact_dir" -name "$binary_name*" -type f | head -1)
        if [ -n "$binary_file" ]; then
            echo "✅ Found $platform binary: $binary_file"
            
            # Create archive based on type
            if [ "$archive_type" = "tar.gz" ]; then
                tar -czf "$archive_name" -C "$artifact_dir" "$(basename "$binary_file")"
                echo "📦 Created tar.gz: $archive_name"
            elif [ "$archive_type" = "zip" ]; then
                # Create zip in current directory, not in artifact_dir
                zip "$archive_name" -j "$binary_file"
                echo "📦 Created zip: $archive_name"
            fi
            
            return 0
        fi
    fi
    
    echo "❌ $platform binary not found in $artifact_dir"
    return 1
}

# Ensure we're in the right directory
if [ ! -d "artifacts" ]; then
    echo "❌ artifacts directory not found. Make sure to run this from the project root after downloading artifacts."
    exit 1
fi

# Create archives for each platform
echo "🏗️ Creating platform archives..."
CREATED_COUNT=0

if package_binary "Linux AMD64" "artifacts/vx-linux-amd64" "vx" "vx-linux-amd64.tar.gz" "tar.gz"; then
    CREATED_COUNT=$((CREATED_COUNT + 1))
fi

if package_binary "Windows AMD64" "artifacts/vx-windows-amd64.exe" "vx" "vx-windows-amd64.zip" "zip"; then
    CREATED_COUNT=$((CREATED_COUNT + 1))
fi

if package_binary "macOS Intel" "artifacts/vx-macos-amd64" "vx" "vx-macos-amd64.tar.gz" "tar.gz"; then
    CREATED_COUNT=$((CREATED_COUNT + 1))
fi

if package_binary "macOS ARM64" "artifacts/vx-macos-arm64" "vx" "vx-macos-arm64.tar.gz" "tar.gz"; then
    CREATED_COUNT=$((CREATED_COUNT + 1))
fi

# Check if we created any archives
if [ $CREATED_COUNT -eq 0 ]; then
    echo "💥 FATAL: No archives were created! Cannot proceed with release."
    exit 1
fi

echo "📊 Created $CREATED_COUNT archive(s)"

# Debug: List current directory contents
echo "📁 Current directory contents:"
ls -la

# Generate checksums
echo "🔐 Generating checksums..."
if ls *.tar.gz *.zip 1> /dev/null 2>&1; then
    sha256sum *.tar.gz *.zip > checksums.txt
    echo "✅ Generated checksums:"
    cat checksums.txt
else
    echo "❌ No archive files found for checksum generation"
    echo "📁 Files in current directory:"
    ls -la
    exit 1
fi

# List all release files
echo "📋 Release files created:"
ls -la *.tar.gz *.zip checksums.txt 2>/dev/null || echo "No release files found"

echo "✅ Archives and checksums created successfully!"
