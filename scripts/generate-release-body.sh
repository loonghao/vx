#!/bin/bash
set -euo pipefail

# Generate release body from template
# Usage: ./scripts/generate-release-body.sh <version> [changelog]

VERSION="${1:-}"
CHANGELOG="${2:-"See commit history for detailed changes."}"

if [ -z "$VERSION" ]; then
    echo "❌ Usage: $0 <version> [changelog]"
    echo "   Example: $0 v1.0.0 'Added new features'"
    exit 1
fi

# Remove 'v' prefix if present for clean version display
CLEAN_VERSION="${VERSION#v}"

echo "📝 Generating release body for version: $VERSION"

# Read template
TEMPLATE_FILE="scripts/release-template.md"
if [ ! -f "$TEMPLATE_FILE" ]; then
    echo "❌ Template file not found: $TEMPLATE_FILE"
    exit 1
fi

# Generate release body by replacing placeholders
RELEASE_BODY=$(cat "$TEMPLATE_FILE")
RELEASE_BODY="${RELEASE_BODY//\{\{VERSION\}\}/$VERSION}"
RELEASE_BODY="${RELEASE_BODY//\{\{CHANGELOG\}\}/$CHANGELOG}"

# Write to output file
OUTPUT_FILE="release-body.md"
echo "$RELEASE_BODY" > "$OUTPUT_FILE"

echo "✅ Release body generated: $OUTPUT_FILE"
echo ""
echo "📋 Preview:"
echo "----------------------------------------"
cat "$OUTPUT_FILE"
echo "----------------------------------------"
