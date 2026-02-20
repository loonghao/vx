# provider.star - rcedit provider (Windows-only)
#
# Reuse pattern: Level 2 (partial override)
#   - fetch_versions: fully inherited from github.star
#   - download_url:   overridden — rcedit is Windows-only, direct binary download
#
# Key characteristics:
#   - Windows-only tool (returns None on non-Windows platforms)
#   - Direct binary download (NOT an archive) — type = "binary"
#   - Asset naming: "rcedit-{arch}.exe"  (x64 / arm64 / x86)
#   - URL: https://github.com/electron/rcedit/releases/download/v{version}/rcedit-{arch}.exe
#   - post_extract renames "rcedit-x64.exe" → "bin/rcedit.exe"
#
# Equivalent Rust replaced:
#   - RceditUrlBuilder::download_url()  → download_url() below
#   - RceditRuntime::post_extract()     → install_layout() rename hint

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "rcedit"

def description():
    return "rcedit - Command-line tool to edit resources of Windows executables"

def homepage():
    return "https://github.com/electron/rcedit"

def repository():
    return "https://github.com/electron/rcedit"

def license():
    return "MIT"

def ecosystem():
    return "system"

def aliases():
    return []

# ---------------------------------------------------------------------------
# Platform constraint — Windows only
# ---------------------------------------------------------------------------

platforms = {
    "os": ["windows"],
}

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":                "rcedit",
        "executable":          "rcedit",
        "description":         "Command-line tool to edit resources of Windows executables",
        "aliases":             [],
        "priority":            100,
        "platform_constraint": {"os": ["windows"]},
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["api.github.com", "github.com", "objects.githubusercontent.com"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# fetch_versions — fully inherited from github.star
#
# rcedit tags are "v2.0.0"; strip_v_prefix handled by releases_to_versions()
# so versions are stored as "2.0.0".
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("electron", "rcedit", include_prereleases = False)

# ---------------------------------------------------------------------------
# download_url — custom override
#
# Why override?
#   - Windows-only: return None for non-Windows platforms
#   - Direct binary (no archive): asset is "rcedit-{arch}.exe"
#   - Arch mapping: x64 → "x64", arm64 → "arm64", x86 → "x86"
# ---------------------------------------------------------------------------

def _rcedit_arch(ctx):
    """Map platform arch to rcedit's arch suffix.

    Returns None if the platform is not Windows or arch is unsupported.
    """
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]

    # rcedit is Windows-only
    if os != "windows":
        return None

    arch_map = {
        "x64":   "x64",
        "arm64": "arm64",
        "x86":   "x86",
    }
    return arch_map.get(arch)

def download_url(ctx, version):
    """Build the rcedit download URL for the given version and platform.

    Args:
        ctx:     Provider context
        version: Version string WITHOUT 'v' prefix, e.g. "2.0.0"

    Returns:
        Download URL string, or None if platform is unsupported (non-Windows)
    """
    arch = _rcedit_arch(ctx)
    if not arch:
        return None

    # Asset: "rcedit-x64.exe"  (direct binary, no archive)
    asset = "rcedit-{}.exe".format(arch)
    tag   = "v{}".format(version)

    return github_asset_url("electron", "rcedit", tag, asset)

# ---------------------------------------------------------------------------
# install_layout — binary download, rename to standard name
#
# The downloaded file is "rcedit-x64.exe" (or arm64/x86).
# After install, it should be accessible as "rcedit.exe" in bin/.
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    """Describe how to handle the downloaded binary.

    Args:
        ctx:     Provider context
        version: Installed version string

    Returns:
        Layout dict consumed by the vx installer
    """
    arch = _rcedit_arch(ctx)
    if not arch:
        return {"type": "binary", "executable_name": "rcedit.exe"}

    # Downloaded as "rcedit-x64.exe", rename to "rcedit.exe"
    return {
        "type":            "binary",
        "source_name":     "rcedit-{}.exe".format(arch),
        "executable_name": "rcedit.exe",
    }

# ---------------------------------------------------------------------------
# Path queries (RFC-0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    """Return the vx store root directory for rcedit."""
    return "{vx_home}/store/rcedit"

def get_execute_path(ctx, version):
    """Return the executable path for the given version (Windows only)."""
    return "{install_dir}/rcedit.exe"

def post_install(ctx, version, install_dir):
    """No post-install steps needed for rcedit."""
    return None

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {
        "PATH": install_dir,
    }
