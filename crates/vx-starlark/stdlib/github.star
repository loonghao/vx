# @vx//stdlib:github.star
# GitHub provider base utilities for vx provider scripts
#
# This module provides factory functions that implement the "inheritance via load()"
# pattern: a provider can load a base implementation and only override what it needs.
#
# Usage (simple - reuse everything):
#   load("@vx//stdlib:github.star", "make_github_provider")
#   provider = make_github_provider("jj-vcs", "jj")
#   fetch_versions = provider.fetch_versions
#   download_url   = provider.download_url
#
# Usage (override download_url only):
#   load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
#   fetch_versions = make_fetch_versions("jj-vcs", "jj")
#
#   def download_url(ctx, version):
#       triple = _jj_triple(ctx)
#       ext    = "zip" if ctx.platform.os == "windows" else "tar.gz"
#       asset  = "jj-v{}-{}.{}".format(version, triple, ext)
#       return github_asset_url("jj-vcs", "jj", "v" + version, asset)

load("@vx//stdlib:http.star",     "github_releases", "parse_github_tag",
                                   "github_download_url", "releases_to_versions")
load("@vx//stdlib:platform.star", "platform_triple", "platform_ext", "exe_ext")

# ---------------------------------------------------------------------------
# Low-level helpers
# ---------------------------------------------------------------------------

def github_asset_url(owner, repo, tag, asset_name):
    """Build a GitHub release asset download URL.

    Thin alias over github_download_url for naming consistency.

    Args:
        owner:      GitHub owner (e.g. "jj-vcs")
        repo:       GitHub repo  (e.g. "jj")
        tag:        Release tag  (e.g. "v0.38.0")
        asset_name: Asset file   (e.g. "jj-v0.38.0-x86_64-pc-windows-msvc.zip")

    Returns:
        Full download URL string
    """
    return github_download_url(owner, repo, tag, asset_name)

# ---------------------------------------------------------------------------
# Factory: fetch_versions
# ---------------------------------------------------------------------------

def make_fetch_versions(owner, repo, include_prereleases = False):
    """Return a fetch_versions(ctx) function bound to a specific GitHub repo.

    This is the primary "inheritance" mechanism: a provider that only needs
    standard GitHub release fetching can do:

        fetch_versions = make_fetch_versions("owner", "repo")

    and get a fully working implementation without writing any logic.

    Args:
        owner:               GitHub owner
        repo:                GitHub repo name
        include_prereleases: Whether to include pre-release versions

    Returns:
        A function with signature: fetch_versions(ctx) -> list[VersionInfo]
    """
    def fetch_versions(ctx):
        releases = github_releases(ctx, owner, repo, include_prereleases)
        return releases_to_versions(releases)

    return fetch_versions

# ---------------------------------------------------------------------------
# Factory: download_url  (standard Rust-target-triple layout)
# ---------------------------------------------------------------------------

def make_download_url(owner, repo, asset_template):
    """Return a download_url(ctx, version) function for a standard GitHub release.

    The asset_template supports the following placeholders:
        {version}  - version string without 'v' prefix  (e.g. "0.38.0")
        {vversion} - version string with    'v' prefix  (e.g. "v0.38.0")
        {triple}   - Rust target triple                 (e.g. "x86_64-pc-windows-msvc")
        {ext}      - archive extension without dot      (e.g. "zip" or "tar.gz")
        {exe}      - executable extension               (e.g. ".exe" or "")
        {os}       - OS name                            (e.g. "windows")
        {arch}     - arch name                          (e.g. "x64")

    Example:
        make_download_url(
            "jj-vcs", "jj",
            "jj-{vversion}-{triple}.{ext}"
        )

    Args:
        owner:          GitHub owner
        repo:           GitHub repo name
        asset_template: Asset filename template with placeholders

    Returns:
        A function with signature: download_url(ctx, version) -> str | None
    """
    def download_url(ctx, version):
        triple = platform_triple(ctx)
        if not triple:
            return None
        ext  = platform_ext(ctx).lstrip(".")   # "zip" or "tar.gz"
        exe  = exe_ext(ctx)                     # ".exe" or ""
        os   = ctx.platform.os
        arch = ctx.platform.arch

        asset = asset_template
        asset = asset.replace("{version}",  version)
        asset = asset.replace("{vversion}", "v" + version)
        asset = asset.replace("{triple}",   triple)
        asset = asset.replace("{ext}",      ext)
        asset = asset.replace("{exe}",      exe)
        asset = asset.replace("{os}",       os)
        asset = asset.replace("{arch}",     arch)

        tag = "v" + version
        return github_asset_url(owner, repo, tag, asset)

    return download_url

# ---------------------------------------------------------------------------
# Composite factory: full provider namespace
# ---------------------------------------------------------------------------

def make_github_provider(owner, repo, asset_template = None,
                         include_prereleases = False):
    """Create a complete GitHub provider namespace with fetch_versions + download_url.

    This is the highest-level factory: a provider that follows the standard
    GitHub release pattern can be implemented in just two lines:

        load("@vx//stdlib:github.star", "make_github_provider")
        _p = make_github_provider("owner", "repo", "{name}-{vversion}-{triple}.{ext}")
        fetch_versions = _p.fetch_versions
        download_url   = _p.download_url

    If asset_template is None, download_url will return None (provider must
    override it manually).

    Args:
        owner:               GitHub owner
        repo:                GitHub repo name
        asset_template:      Asset filename template (see make_download_url)
        include_prereleases: Whether to include pre-release versions

    Returns:
        A struct-like dict with keys: fetch_versions, download_url
    """
    fv = make_fetch_versions(owner, repo, include_prereleases)

    if asset_template != None:
        du = make_download_url(owner, repo, asset_template)
    else:
        def du(ctx, version):
            return None

    return {
        "fetch_versions": fv,
        "download_url":   du,
    }
