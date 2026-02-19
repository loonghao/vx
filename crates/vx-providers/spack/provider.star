# provider.star - Spack provider
#
# Version source: https://github.com/spack/spack/releases
#
# Spack is a flexible package manager for supercomputers, Linux, and macOS.
# It is Python-based and typically installed via git clone.
# Only supports Linux and macOS (no Windows support).
#
# Inheritance pattern: Level 2 (custom fetch_versions + download_url)

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "spack"

def description():
    return "Spack - Flexible package manager for supercomputers, Linux, and macOS"

def homepage():
    return "https://spack.io"

def repository():
    return "https://github.com/spack/spack"

def license():
    return "Apache-2.0 OR MIT"

def ecosystem():
    return "python"

# ---------------------------------------------------------------------------
# Platform constraint — Linux and macOS only
# ---------------------------------------------------------------------------

platforms = {
    "os": ["linux", "macos"],
}

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "spack",
        "executable":  "spack",
        "description": "Spack package manager",
        "priority":    100,
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["api.github.com", "github.com"],
    "fs":   [],
    "exec": ["python", "git"],
}

# ---------------------------------------------------------------------------
# fetch_versions — spack/spack GitHub releases
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("spack", "spack")

# ---------------------------------------------------------------------------
# download_url — tar.gz source archive
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    """Build Spack download URL from GitHub releases.

    Spack releases are source archives (tar.gz).
    """
    os = ctx["platform"]["os"]
    if os == "windows":
        return None

    # e.g. https://github.com/spack/spack/releases/download/v0.22.0/spack-0.22.0.tar.gz
    asset = "spack-{}.tar.gz".format(version)
    return github_asset_url("spack", "spack", version, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    return {
        "type":             "archive",
        "strip_prefix":     "spack-{}".format(version),
        "executable_paths": ["bin/spack"],
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {
        "SPACK_ROOT": install_dir,
        "PATH":       install_dir + "/bin",
    }

# ---------------------------------------------------------------------------
# deps — requires python 3.6+ and git
# ---------------------------------------------------------------------------

def deps(ctx, version):
    return [
        {"runtime": "python", "version": ">=3.6",
         "reason": "Spack requires Python 3.6+"},
        {"runtime": "git", "version": "*", "optional": True,
         "reason": "Git is required for fetching Spack packages"},
    ]
