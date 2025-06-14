#!/bin/bash
set -euo pipefail

# Verify release files
# Usage: ./scripts/verify-release.sh

echo "🔍 Verifying release files..."

VERIFICATION_FAILED=0

# Check archives
echo "📦 Checking archives..."
EXPECTED_ARCHIVES=("vx-linux-amd64.tar.gz" "vx-windows-amd64.zip" "vx-macos-amd64.tar.gz" "vx-macos-arm64.tar.gz")
FOUND_ARCHIVES=0

for archive in "${EXPECTED_ARCHIVES[@]}"; do
    if [ -f "$archive" ]; then
        size=$(stat -c%s "$archive" 2>/dev/null || stat -f%z "$archive" 2>/dev/null || echo "unknown")
        echo "✅ Found archive: $archive ($size bytes)"
        FOUND_ARCHIVES=$((FOUND_ARCHIVES + 1))
        
        # Basic archive integrity check
        if [[ "$archive" == *.tar.gz ]]; then
            if tar -tzf "$archive" >/dev/null 2>&1; then
                echo "  ✅ Archive integrity check passed"
            else
                echo "  ❌ Archive integrity check failed"
                VERIFICATION_FAILED=1
            fi
        elif [[ "$archive" == *.zip ]]; then
            if unzip -t "$archive" >/dev/null 2>&1; then
                echo "  ✅ Archive integrity check passed"
            else
                echo "  ❌ Archive integrity check failed"
                VERIFICATION_FAILED=1
            fi
        fi
    else
        echo "❌ Missing archive: $archive"
        VERIFICATION_FAILED=1
    fi
done

echo "📊 Found $FOUND_ARCHIVES out of ${#EXPECTED_ARCHIVES[@]} expected archives"

# Check checksums file
echo "🔐 Checking checksums..."
if [ -f "checksums.txt" ]; then
    echo "✅ Found checksums.txt"
    echo "📋 Checksums content:"
    cat checksums.txt
    
    # Verify checksums
    if command -v sha256sum >/dev/null 2>&1; then
        echo "🔐 Verifying checksums..."
        if sha256sum -c checksums.txt; then
            echo "✅ All checksums verified successfully"
        else
            echo "❌ Checksum verification failed"
            VERIFICATION_FAILED=1
        fi
    else
        echo "⚠️ sha256sum not available, skipping verification"
    fi
else
    echo "❌ Missing checksums.txt"
    VERIFICATION_FAILED=1
fi

# Check if we have minimum required files
if [ $FOUND_ARCHIVES -eq 0 ]; then
    echo "💥 FATAL: No archive files found!"
    VERIFICATION_FAILED=1
fi

# Summary
echo ""
echo "📋 Verification Summary:"
echo "  Archives found: $FOUND_ARCHIVES/${#EXPECTED_ARCHIVES[@]}"
echo "  Checksums file: $([ -f "checksums.txt" ] && echo "✅" || echo "❌")"

if [ $VERIFICATION_FAILED -eq 0 ]; then
    echo "✅ All release files verified successfully!"
    exit 0
else
    echo "❌ Release file verification failed!"
    exit 1
fi
