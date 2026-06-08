#!/usr/bin/env bash
# headroom MCP smoke test (PIP-584 Phase 1)
#
# Validates that the headroom MCP server is reachable and the three
# core tools (headroom_compress, headroom_retrieve, headroom_stats)
# work correctly.
#
# Requires: vx (with mcpcall installed) or mcpcall directly.
#
# Usage:
#   ./smoke_test.sh
#   ./smoke_test.sh --url http://localhost:8765/mcp
#   ./smoke_test.sh --json
#   ./smoke_test.sh --sample-file ./test_data.txt

set -euo pipefail

MCP_URL="${MCP_URL:-http://127.0.0.1:8765/mcp}"
SAMPLE_FILE=""
JSON_OUTPUT=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --url) MCP_URL="$2"; shift 2 ;;
        --sample-file) SAMPLE_FILE="$2"; shift 2 ;;
        --json) JSON_OUTPUT=true; shift ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

MC_MCP_CALL=""
if command -v vx &>/dev/null; then
    MC_MCP_CALL="vx mcpcall"
elif command -v mcpcall &>/dev/null; then
    MC_MCP_CALL="mcpcall"
else
    echo "ERROR: neither vx nor mcpcall found on PATH"
    exit 1
fi

run_mcpcall() {
    # shellcheck disable=SC2086
    $MC_MCP_CALL "$@"
}

results_json() {
    cat <<EOF
{
  "url": "$MCP_URL",
  "tools_found": $(printf '%s\n' "${TOOLS_FOUND[@]}" | jq -R . | jq -s . 2>/dev/null || echo '[]'),
  "compress_ok": $COMPRESS_OK,
  "retrieve_ok": $RETRIEVE_OK,
  "stats_ok": $STATS_OK,
  "roundtrip_ok": $ROUNDTRIP_OK
}
EOF
}

TOOLS_FOUND=()
COMPRESS_OK=false
RETRIEVE_OK=false
STATS_OK=false
ROUNDTRIP_OK=false

# Step 1: List tools
if ! $JSON_OUTPUT; then
    echo "--- Listing MCP tools ---"
fi

LIST_OUTPUT=$(run_mcpcall --url "$MCP_URL" list 2>&1 || true)

if echo "$LIST_OUTPUT" | grep -qi "compress"; then
    TOOLS_FOUND+=("headroom_compress")
fi
if echo "$LIST_OUTPUT" | grep -qi "retrieve"; then
    TOOLS_FOUND+=("headroom_retrieve")
fi
if echo "$LIST_OUTPUT" | grep -qi "stats"; then
    TOOLS_FOUND+=("headroom_stats")
fi

if ! $JSON_OUTPUT; then
    if [ ${#TOOLS_FOUND[@]} -gt 0 ]; then
        echo "  Found MCP tools: ${TOOLS_FOUND[*]}"
    else
        echo "  No expected MCP tools found"
        echo "  Raw output: $LIST_OUTPUT"
    fi
fi

# Step 2: Read sample content
if [ -n "$SAMPLE_FILE" ] && [ -f "$SAMPLE_FILE" ]; then
    SAMPLE_CONTENT=$(cat "$SAMPLE_FILE")
else
    SAMPLE_CONTENT="Hello, headroom MCP smoke test! This is sample content for round-trip testing."
fi

# Step 3: Test compress
if ! $JSON_OUTPUT; then
    echo ""
    echo "--- Testing headroom_compress ---"
fi

COMPRESS_OUTPUT=$(run_mcpcall --url "$MCP_URL" call headroom_compress --args "{\"content\": \"$SAMPLE_CONTENT\"}" 2>&1 || true)
HASH=$(echo "$COMPRESS_OUTPUT" | tail -1 | tr -d '[:space:]')

if [ -n "$HASH" ]; then
    COMPRESS_OK=true
    if ! $JSON_OUTPUT; then
        echo "  compress result: $HASH"
    fi
else
    if ! $JSON_OUTPUT; then
        echo "  compress failed or returned empty"
    fi
fi

# Step 4: Test retrieve
if ! $JSON_OUTPUT; then
    echo ""
    echo "--- Testing headroom_retrieve ---"
fi

if $COMPRESS_OK && [ -n "$HASH" ]; then
    RETRIEVE_OUTPUT=$(run_mcpcall --url "$MCP_URL" call headroom_retrieve --args "{\"hash\": \"$HASH\"}" 2>&1 || true)
    if echo "$RETRIEVE_OUTPUT" | grep -qF "$SAMPLE_CONTENT"; then
        RETRIEVE_OK=true
        ROUNDTRIP_OK=true
        if ! $JSON_OUTPUT; then
            echo "  retrieve: content matches original"
        fi
    else
        if ! $JSON_OUTPUT; then
            echo "  retrieve: content mismatch"
            echo "  Got: $RETRIEVE_OUTPUT"
        fi
    fi
else
    if ! $JSON_OUTPUT; then
        echo "  Skipping retrieve: compress failed or hash empty"
    fi
fi

# Step 5: Test stats
if ! $JSON_OUTPUT; then
    echo ""
    echo "--- Testing headroom_stats ---"
fi

STATS_OUTPUT=$(run_mcpcall --url "$MCP_URL" call headroom_stats --args "{}" 2>&1 || true)
if [ -n "$STATS_OUTPUT" ]; then
    STATS_OK=true
    if ! $JSON_OUTPUT; then
        echo "  stats: $STATS_OUTPUT"
    fi
fi

# Output
if $JSON_OUTPUT; then
    results_json
else
    echo ""
    echo "=== Summary ==="
    echo ""
    echo "  Tools found:     ${TOOLS_FOUND[*]}"
    echo "  compress:        $(if $COMPRESS_OK; then echo 'PASS'; else echo 'FAIL'; fi)"
    echo "  retrieve:        $(if $RETRIEVE_OK; then echo 'PASS'; else echo 'FAIL'; fi)"
    echo "  stats:           $(if $STATS_OK; then echo 'PASS'; else echo 'FAIL'; fi)"
    echo "  round-trip:      $(if $ROUNDTRIP_OK; then echo 'PASS'; else echo 'FAIL'; fi)"
fi

# Exit code
if $COMPRESS_OK && $RETRIEVE_OK && $STATS_OK; then
    exit 0
else
    exit 1
fi
