# @vx//stdlib:http.star
# HTTP utilities for vx provider scripts
#
# Usage:
#   load("@vx//stdlib:http.star", "github_releases", "parse_github_tag", "github_download_url")

def github_releases(ctx, owner, repo, include_prereleases = False):
    """Fetch GitHub releases for a repository.

    Args:
        ctx: Provider context
        owner: GitHub repository owner
        repo: GitHub repository name
        include_prereleases: Whether to include pre-release versions

    Returns:
        List of release dicts with keys: tag_name, prerelease, draft, body
    """
    url = "https://api.github.com/repos/{}/{}/releases?per_page=50".format(owner, repo)
    releases = ctx.http_get_json(url)
    if not include_prereleases:
        releases = [r for r in releases if not r.get("prerelease", False) and not r.get("draft", False)]
    return releases

def parse_github_tag(tag):
    """Parse a GitHub tag name to extract the version string.

    Handles common patterns:
    - "v1.2.3" -> "1.2.3"
    - "1.2.3" -> "1.2.3"
    - "release-1.2.3" -> "1.2.3"

    Args:
        tag: GitHub tag name string

    Returns:
        Version string without prefix
    """
    # Strip common prefixes
    for prefix in ["v", "V", "release-", "version-"]:
        if tag.startswith(prefix):
            return tag[len(prefix):]
    return tag

def github_download_url(owner, repo, tag, asset_name):
    """Build a GitHub release download URL.

    Args:
        owner: GitHub repository owner
        repo: GitHub repository name
        tag: Release tag (e.g. "v1.2.3")
        asset_name: Asset filename (e.g. "tool-linux-x64.tar.gz")

    Returns:
        Full download URL string
    """
    return "https://github.com/{}/{}/releases/download/{}/{}".format(
        owner, repo, tag, asset_name
    )

def github_latest_release(ctx, owner, repo):
    """Get the latest stable release tag for a GitHub repository.

    Args:
        ctx: Provider context
        owner: GitHub repository owner
        repo: GitHub repository name

    Returns:
        Latest release tag string, or None if not found
    """
    url = "https://api.github.com/repos/{}/{}/releases/latest".format(owner, repo)
    release = ctx.http_get_json(url)
    return release.get("tag_name")

def releases_to_versions(releases, tag_key = "tag_name"):
    """Convert a list of GitHub release dicts to version info dicts.

    Args:
        releases: List of GitHub release dicts
        tag_key: Key to use for the tag name (default: "tag_name")

    Returns:
        List of version info dicts: {"version": str, "lts": bool, "prerelease": bool}
    """
    versions = []
    for release in releases:
        tag = release.get(tag_key, "")
        version = parse_github_tag(tag)
        if version:
            versions.append({
                "version": version,
                "lts": not release.get("prerelease", False),
                "prerelease": release.get("prerelease", False),
                "date": release.get("published_at", ""),
            })
    return versions
