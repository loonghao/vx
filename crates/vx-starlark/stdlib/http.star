# @vx//stdlib:http.star
# HTTP utilities for vx provider scripts
#
# Design: Starlark scripts are pure computation — they do NOT make real HTTP
# requests. Instead, functions like github_releases() return a descriptor dict
# that the Rust runtime interprets to perform the actual network I/O.
#
# This keeps Starlark sandboxed and testable, while the Rust layer handles
# all real I/O (HTTP, filesystem, etc.).
#
# Usage:
#   load("@vx//stdlib:http.star", "github_releases", "parse_github_tag",
#        "github_download_url", "releases_to_versions")

def github_releases(ctx, owner, repo, include_prereleases = False):
    """Return a GitHub releases descriptor for the Rust runtime to execute.

    This function does NOT make a real HTTP request. It returns a descriptor
    dict that the Rust runtime interprets to fetch releases from GitHub API.

    Args:
        ctx:                 Provider context dict (injected by vx runtime)
        owner:               GitHub repository owner (e.g. "jj-vcs")
        repo:                GitHub repository name (e.g. "jj")
        include_prereleases: Whether to include pre-release versions

    Returns:
        A releases descriptor dict consumed by releases_to_versions()
    """
    return {
        "__type":             "github_releases",
        "owner":              owner,
        "repo":               repo,
        "include_prereleases": include_prereleases,
        "url":                "https://api.github.com/repos/{}/{}/releases?per_page=100".format(owner, repo),
    }

def parse_github_tag(tag):
    """Parse a GitHub tag name to extract the version string.

    Handles common patterns:
    - "v1.2.3"       -> "1.2.3"
    - "1.2.3"        -> "1.2.3"
    - "release-1.2.3" -> "1.2.3"
    - "version-1.2.3" -> "1.2.3"

    Args:
        tag: GitHub tag name string

    Returns:
        Version string without prefix
    """
    for prefix in ["v", "V", "release-", "version-"]:
        if tag.startswith(prefix):
            return tag[len(prefix):]
    return tag

def github_download_url(owner, repo, tag, asset_name):
    """Build a GitHub release asset download URL.

    Args:
        owner:      GitHub repository owner
        repo:       GitHub repository name
        tag:        Release tag (e.g. "v1.2.3")
        asset_name: Asset filename (e.g. "tool-linux-x64.tar.gz")

    Returns:
        Full download URL string
    """
    return "https://github.com/{}/{}/releases/download/{}/{}".format(
        owner, repo, tag, asset_name
    )

def github_latest_release(ctx, owner, repo):
    """Return a descriptor for the latest stable release tag.

    Like github_releases(), this returns a descriptor dict rather than
    making a real HTTP request.

    Args:
        ctx:   Provider context dict
        owner: GitHub repository owner
        repo:  GitHub repository name

    Returns:
        A latest-release descriptor dict
    """
    return {
        "__type": "github_latest_release",
        "owner":  owner,
        "repo":   repo,
        "url":    "https://api.github.com/repos/{}/{}/releases/latest".format(owner, repo),
    }

def fetch_json(ctx, url):
    """Return a generic HTTP JSON fetch descriptor for the Rust runtime to execute.

    This function does NOT make a real HTTP request. It returns a descriptor
    dict that the Rust runtime interprets to fetch JSON from any URL.

    Use this for non-GitHub APIs (e.g. go.dev, nodejs.org, etc.) that return
    JSON data directly.

    Args:
        ctx: Provider context dict (injected by vx runtime)
        url: The URL to fetch JSON from

    Returns:
        A fetch_json descriptor dict consumed by the Rust runtime.
        The Rust runtime will replace this descriptor with the actual JSON
        response when executing fetch_versions().
    """
    return {
        "__type": "fetch_json",
        "url":    url,
    }

def fetch_json_versions(ctx, url, transform, headers = {}):
    """Return a fetch_json_versions descriptor for the Rust runtime to execute.

    This is the unified descriptor for fetching version lists from any JSON API.
    The Rust runtime fetches the URL and applies the named transform strategy
    to convert the raw JSON response into a list of VersionInfo objects.

    This function does NOT make a real HTTP request. It returns a descriptor
    dict that the Rust runtime resolves.

    Supported transform strategies:
        "go_versions"       - go.dev API: [{version: "go1.21.0", stable: true, ...}]
        "nodejs_org"        - nodejs.org API: [{version: "v20.0.0", lts: "Iron", ...}]
        "pypi"              - PyPI JSON API: {info: {version: "..."}, releases: {...}}
        "npm_registry"      - npm registry: {versions: {"1.0.0": {...}, ...}}
        "hashicorp_releases"- HashiCorp releases API: {versions: {"1.0.0": {...}}}
        "adoptium"          - Eclipse Adoptium API for Java
        "github_tags"       - GitHub tags API (alternative to github_releases)

    Args:
        ctx:       Provider context dict (injected by vx runtime)
        url:       The URL to fetch JSON from
        transform: Named transform strategy (see above)
        headers:   Optional HTTP headers dict (e.g. {"Authorization": "token ..."})

    Returns:
        A fetch_json_versions descriptor dict consumed by the Rust runtime.

    Example (node/provider.star):
        load("@vx//stdlib:http.star", "fetch_json_versions")

        def fetch_versions(ctx):
            return fetch_json_versions(ctx,
                "https://nodejs.org/dist/index.json",
                "nodejs_org",
            )

    Example (go/provider.star):
        load("@vx//stdlib:http.star", "fetch_json_versions")

        def fetch_versions(ctx):
            return fetch_json_versions(ctx,
                "https://go.dev/dl/?mode=json&include=all",
                "go_versions",
            )
    """
    return {
        "__type":    "fetch_json_versions",
        "url":       url,
        "transform": transform,
        "headers":   headers,
    }

def releases_to_versions(releases, tag_key = "tag_name"):
    """Convert a GitHub releases descriptor (or list) to version info dicts.

    When given a descriptor dict (from github_releases()), this function
    passes it through so the Rust runtime can resolve it. When given a
    plain list of release dicts (e.g. in tests), it converts them directly.

    Args:
        releases: Either a github_releases() descriptor dict, or a list of
                  GitHub release dicts with keys: tag_name, prerelease, etc.
        tag_key:  Key to use for the tag name (default: "tag_name")

    Returns:
        Either a versions descriptor dict (for Rust to resolve), or a list
        of version info dicts: {"version": str, "lts": bool, "prerelease": bool}
    """
    # If it's a descriptor, wrap it for the Rust runtime
    if type(releases) == type({}):
        if releases.get("__type") == "github_releases":
            return {
                "__type":             "github_versions",
                "source":             releases,
                "tag_key":            tag_key,
                "strip_v_prefix":     True,
                "skip_prereleases":   not releases.get("include_prereleases", False),
            }

    # Plain list: convert directly (useful for testing / custom sources)
    versions = []
    for release in releases:
        tag = release.get(tag_key, "")
        version = parse_github_tag(tag)
        if version:
            versions.append({
                "version":    version,
                "lts":        not release.get("prerelease", False),
                "prerelease": release.get("prerelease", False),
                "date":       release.get("published_at", ""),
            })
    return versions
