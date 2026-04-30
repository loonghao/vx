# provider.star - worktrunk provider
#
# worktrunk: Git worktree manager for parallel AI agent workflows
# Homepage: https://worktrunk.dev
# Releases: https://github.com/max-sixty/worktrunk/releases
#
# Asset naming: worktrunk-{triple}.tar.xz  (NO version in asset name!)
#   e.g. worktrunk-x86_64-unknown-linux-musl.tar.xz
# Tag format:   v{version}  (e.g. "v0.46.0")
#
# NOTE: Asset names do NOT include the version string.
#       The version is only present in the GitHub release tag.
#       Custom download_url is required.
#
# Archive structure (cargo-dist):
#   Top-level dir: worktrunk-v{VERSION}-{TRIPLE}/
#   Binary:         worktrunk-v{VERSION}-{TRIPLE}/wt  (or wt.exe)

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "fetch_versions_with_tag_prefix")
load("@vx//stdlib:layout.star", "archive_layout", "post_extract_permissions")
load("@vx//stdlib:provider.star", "path_fns")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "worktrunk"
description = "Git worktree manager for parallel AI agent workflows"
homepage    = "https://worktrunk.dev"
repository  = "https://github.com/max-sixty/worktrunk"
license     = "MIT OR Apache-2.0"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("worktrunk", executable = "wt", aliases = ["wt"]),
]

# ---------------------------------------------------------------------------
# Permissions — needs GitHub releases access
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# Version fetching
# ---------------------------------------------------------------------------
# Tags look like "v0.46.0"
# fetch_versions_with_tag_prefix(owner, repo, tag_prefix="v")
#   → list of version strings (without the "v" prefix)
fetch_versions = fetch_versions_with_tag_prefix(
    "max-sixty", "worktrunk", tag_prefix = "v",
)

# ---------------------------------------------------------------------------
# Platform helpers
# ---------------------------------------------------------------------------

def _triple(ctx):
    """Return the platform triple used in worktrunk asset filenames."""
    os   = ctx.platform.os
    arch = ctx.platform.arch
    if os == "macos":
        return "aarch64-apple-darwin" if arch == "aarch64" else "x86_64-apple-darwin"
    elif os == "linux":
        # worktrunk uses musl for Linux
        return "aarch64-unknown-linux-musl" if arch == "aarch64" else "x86_64-unknown-linux-musl"
    elif os == "windows":
        return "x86_64-pc-windows-msvc"
    return None

# ---------------------------------------------------------------------------
# download_url  (custom — asset name has NO version component)
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    """Build the GitHub release download URL.

    The asset filename does NOT include the version.
    The version is only in the release tag used in the URL path.

    URL pattern:
      https://github.com/max-sixty/worktrunk/releases/download/v{VERSION}/worktrunk-{TRIPLE}.{EXT}
    """
    triple = _triple(ctx)
    if triple == None:
        return None
    ext = "zip" if ctx.platform.os == "windows" else "tar.xz"
    asset = "worktrunk-{}.{}".format(triple, ext)
    return "https://github.com/max-sixty/worktrunk/releases/download/v{}/{}".format(version, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------
# cargo-dist archives contain a top-level directory named:
#   worktrunk-v{VERSION}-{TRIPLE}/
#
# archive_layout(executable, strip_prefix=...) returns the install_layout function.
# strip_prefix supports placeholders: {version}, {vversion}, {triple}
#
install_layout = archive_layout(
    "wt",
    strip_prefix = "worktrunk-v{version}-{triple}",
)

# ---------------------------------------------------------------------------
# store_root  — where vx stores installed versions
# ---------------------------------------------------------------------------
# path_fns("worktrunk") returns dict with "store_root" and "get_execute_path" functions
_paths = path_fns("worktrunk")
store_root       = _paths["store_root"]
get_execute_path = _paths["get_execute_path"]

# ---------------------------------------------------------------------------
# environment  — env vars needed to run worktrunk
# ---------------------------------------------------------------------------

def environment(_ctx, _version):
    """Worktrunk needs no special environment variables."""
    return []

# ---------------------------------------------------------------------------
# deps  — runtime dependencies
# ---------------------------------------------------------------------------

def deps(_ctx, _version):
    """Worktrunk has no runtime dependencies (statically linked Rust binary)."""
    return []

# ---------------------------------------------------------------------------
# post_extract  — make binaries executable on Unix
# ---------------------------------------------------------------------------
# cargo-dist sets permissions, but we ensure it here for safety.
# post_extract_permissions(["wt", ...]) generates the post_extract hook.
post_extract = post_extract_permissions(["wt"])
