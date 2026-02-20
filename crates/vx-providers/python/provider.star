# provider.star - Python provider
#
# Version source: GitHub releases of python-build-standalone
#   https://github.com/astral-sh/python-build-standalone/releases
#
# python-build-standalone provides pre-built, portable Python distributions
# that work without system dependencies. Used by uv, rye, etc.
#
# Bundled runtimes: pip (included in every Python release)
#
# Inheritance pattern: Level 2 (custom fetch_versions + download_url)

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "python"

def description():
    return "Python - A programming language that lets you work quickly and integrate systems more effectively"

def homepage():
    return "https://www.python.org"

def repository():
    return "https://github.com/python/cpython"

def license():
    return "PSF-2.0"

def ecosystem():
    return "python"

def aliases():
    return ["python3", "py"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "python",
        "executable":  "python",
        "description": "Python programming language runtime",
        "aliases":     ["python3", "py"],
        "priority":    100,
    },
    {
        "name":        "pip",
        "executable":  "pip",
        "description": "Python package installer (bundled with Python)",
        "aliases":     ["pip3"],
        "bundled_with": "python",
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["api.github.com", "github.com"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# fetch_versions — python-build-standalone GitHub releases
#
# Tag format: "20240107" (date-based), asset names encode Python version:
#   cpython-3.12.1+20240107-x86_64-pc-windows-msvc-install_only_stripped.tar.gz
# We extract the Python version from asset names.
# ---------------------------------------------------------------------------

def fetch_versions(ctx):
    """Fetch Python versions from python-build-standalone GitHub releases."""
    releases = ctx["http"]["get_json"](
        "https://api.github.com/repos/astral-sh/python-build-standalone/releases?per_page=50"
    )

    versions = {}
    for release in releases:
        if release.get("draft") or release.get("prerelease"):
            continue
        for asset in release.get("assets", []):
            asset_name = asset.get("name", "")
            # Parse: cpython-3.12.1+20240107-x86_64-...
            if asset_name.startswith("cpython-") and "install_only" in asset_name:
                # Extract version: "3.12.1" from "cpython-3.12.1+20240107-..."
                rest = asset_name[len("cpython-"):]
                plus_idx = rest.find("+")
                if plus_idx > 0:
                    py_version = rest[:plus_idx]
                    if py_version not in versions:
                        # Use release tag as the build date
                        tag = release.get("tag_name", "")
                        versions[py_version] = {
                            "version":    py_version,
                            "lts":        py_version.startswith("3."),
                            "prerelease": False,
                            "build_tag":  tag,
                        }

    return list(versions.values())

# ---------------------------------------------------------------------------
# download_url — python-build-standalone asset
# ---------------------------------------------------------------------------

def _pbs_triple(ctx):
    """Map vx platform to python-build-standalone triple."""
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]

    triples = {
        "windows/x64":   "x86_64-pc-windows-msvc",
        "macos/x64":     "x86_64-apple-darwin",
        "macos/arm64":   "aarch64-apple-darwin",
        "linux/x64":     "x86_64-unknown-linux-gnu",
        "linux/arm64":   "aarch64-unknown-linux-gnu",
    }
    return triples.get("{}/{}".format(os, arch))

def download_url(ctx, version):
    """Build the python-build-standalone download URL.

    Args:
        ctx:     Provider context
        version: Python version string, e.g. "3.12.1"

    Returns:
        Download URL string, or None if platform is unsupported
    """
    triple = _pbs_triple(ctx)
    if not triple:
        return None

    # We need to find the latest release tag for this Python version
    # Query the releases to find the matching asset
    releases = ctx["http"]["get_json"](
        "https://api.github.com/repos/astral-sh/python-build-standalone/releases?per_page=50"
    )

    for release in releases:
        if release.get("draft") or release.get("prerelease"):
            continue
        tag = release.get("tag_name", "")
        for asset in release.get("assets", []):
            asset_name = asset.get("name", "")
            # Match: cpython-{version}+{date}-{triple}-install_only_stripped.tar.gz
            expected_prefix = "cpython-{}+".format(version)
            expected_suffix = "-{}-install_only_stripped.tar.gz".format(triple)
            if asset_name.startswith(expected_prefix) and asset_name.endswith(expected_suffix):
                return asset.get("browser_download_url")

    return None

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    os = ctx["platform"]["os"]

    if os == "windows":
        exe_paths = ["python/python.exe", "python.exe"]
    else:
        exe_paths = ["python/bin/python3", "python/bin/python", "bin/python3"]

    return {
        "type":             "archive",
        "strip_prefix":     "",   # python-build-standalone has variable prefix
        "executable_paths": exe_paths,
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    os = ctx["platform"]["os"]
    if os == "windows":
        return {
            "PYTHONHOME": install_dir + "/python",
            "PATH":       install_dir + "/python",
        }
    else:
        return {
            "PYTHONHOME": install_dir + "/python",
            "PATH":       install_dir + "/python/bin",
        }

# ---------------------------------------------------------------------------
# Path queries (RFC-0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    """Return the vx store root directory for python."""
    return "{vx_home}/store/python"

def get_execute_path(ctx, version):
    """Return the executable path for the given version."""
    os = ctx["platform"]["os"]
    if os == "windows":
        return "{install_dir}/python/python.exe"
    else:
        return "{install_dir}/python/bin/python3"

def post_install(ctx, version, install_dir):
    """No post-install steps needed for python."""
    return None

# ---------------------------------------------------------------------------
# deps
# ---------------------------------------------------------------------------

def deps(ctx, version):
    """Python recommends uv for package management."""
    return [
        {"runtime": "uv", "version": "*", "optional": True,
         "reason": "uv provides faster package management for Python"},
    ]
