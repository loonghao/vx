#!/usr/bin/env bash
#
# Scan the commits of a pull request for CI-skip markers.
#
# GitHub suppresses every workflow run of a push event when the head commit
# message contains a skip marker. A squash merge composes its message from the
# pull request title plus a generated bullet list of the branch commits, so a
# marker on a single branch commit silently disables CI on the base branch.
#
# Usage:
#   scripts/check_pr_ci_markers.sh <pr-number-or-url> [--json]
#
# Requires the GitHub CLI (`gh`) with read access to the repository.
# Exits 0 when the pull request is clean, 1 when a marker was found.

set -euo pipefail

if [[ $# -lt 1 || -z "${1:-}" ]]; then
    echo "usage: $(basename "$0") <pr-number-or-url> [--json]" >&2
    exit 2
fi

PR="$1"
shift

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VALIDATOR="$REPO_ROOT/scripts/validate_commit_markers.py"

if [[ ! -f "$VALIDATOR" ]]; then
    echo "missing validator: $VALIDATOR" >&2
    exit 2
fi

if ! command -v gh >/dev/null 2>&1; then
    echo "the GitHub CLI (gh) is required to list pull request commits" >&2
    exit 2
fi

# Resolve the repository from the current checkout so the script also works in
# a fork or a worktree. GITHUB_REPOSITORY wins when running inside Actions.
REPO="${GITHUB_REPOSITORY:-}"
if [[ -z "$REPO" ]]; then
    REPO="$(gh repo view --json nameWithOwner --jq .nameWithOwner)"
fi

# Accept both a number and a pull request URL.
PR_NUMBER="${PR##*/}"

# The squash commit subject is the pull request title, so the title is scanned
# as if it were a commit. Every branch commit follows, encoded as
# "<short sha><TAB><json message>" so multi-line messages stay one line each.
TITLE="$(gh api "repos/$REPO/pulls/$PR_NUMBER" --jq .title)"
TITLE_RECORD="pr-title	$(printf '%s' "$TITLE" | python3 -c 'import json,sys; print(json.dumps(sys.stdin.read()))')"

COMMITS="$(
    gh api --paginate "repos/$REPO/pulls/$PR_NUMBER/commits" \
        --jq '.[] | .sha[0:12] + "\t" + (.commit.message | tojson)'
)"

if [[ -z "$COMMITS" ]]; then
    echo "no commits found for pull request #$PR_NUMBER in $REPO" >&2
    exit 2
fi

printf '%s\n%s\n' "$TITLE_RECORD" "$COMMITS" | python3 "$VALIDATOR" "$@"
