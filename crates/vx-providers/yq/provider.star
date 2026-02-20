# provider.star - yq provider
#
# yq: a portable command-line YAML, JSON, XML, CSV, TOML and properties processor
# Releases: https://github.com/mikefarah/yq/releases
#
# Inheritance level: Level 2
#   - fetch_versions: fully inherited from github.star
#   - download_url:   overridden — yq is a direct binary download (no archive),
#                     uses {os}_{arch} naming (not Rust triple)
#
# URL format: https://github.com/mikefarah/yq/releases/download/v{version}/yq_{os}_{arch}[.exe]

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "yq"

def description():
    return "yq - a portable command-line YAML, JSON, XML, CSV, TOML and properties processor"

def homepage():
    return "https://github.com/mikefarah/yq"

def repository():
    return "https://github.com/mikefarah/yq"

def license():
    return "MIT"

def ecosystem():
    return "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "yq",
        "executable":  "yq",
        "description": "Portable command-line YAML/JSON/XML processor",
        "aliases":     [],
        "priority":    100,
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
# fetch_versions — fully inherited from github.star
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("mikefarah", "yq")

# ---------------------------------------------------------------------------
# download_url — custom override
#
# Why override?
#   - yq releases are direct binary downloads (no .tar.gz / .zip archive)
#   - Naming: yq_{os}_{arch}[.exe]  (NOT Rust triple)
#   - Examples:
#       yq_linux_amd64
#       yq_linux_arm64
#       yq_darwin_amd64
#       yq_darwin_arm64
#       yq_windows_amd64.exe
#       yq_windows_arm64.exe
# ---------------------------------------------------------------------------

def _yq_asset(ctx):
    """Return the yq asset filename for the current platform."""
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]

    os_map = {
        "windows": "windows",
        "macos":   "darwin",
        "linux":   "linux",
    }
    arch_map = {
        "x64":   "amd64",
        "arm64": "arm64",
        "arm":   "arm",
        "x86":   "386",
    }

    os_str   = os_map.get(os)
    arch_str = arch_map.get(arch)
    if not os_str or not arch_str:
        return None

    ext = ".exe" if os == "windows" else ""
    return "yq_{}_{}{}".format(os_str, arch_str, ext)

def download_url(ctx, version):
    """Build the yq download URL.

    yq provides direct binary downloads — no archive extraction needed.

    Args:
        ctx:     Provider context
        version: Version string WITHOUT 'v' prefix, e.g. "4.44.1"

    Returns:
        Download URL string, or None if platform is unsupported
    """
    asset = _yq_asset(ctx)
    if not asset:
        return None

    tag = "v{}".format(version)
    return github_asset_url("mikefarah", "yq", tag, asset)

# ---------------------------------------------------------------------------
# install_layout — binary download, no extraction
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    """yq is a direct binary — no archive to extract."""
    os  = ctx["platform"]["os"]
    exe = "yq.exe" if os == "windows" else "yq"

    return {
        "type":             "binary",
        "rename_to":        exe,
        "executable_paths": [exe],
    }

# ---------------------------------------------------------------------------
# Path queries (RFC-0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    """Return the vx store root directory for yq."""
    return "{vx_home}/store/yq"

def get_execute_path(ctx, version):
    """Return the executable path for the given version."""
    os = ctx["platform"]["os"]
    exe = "yq.exe" if os == "windows" else "yq"
    return "{install_dir}/" + exe

def post_install(ctx, version, install_dir):
    """No post-install actions needed for yq."""
    return None

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {
        "PATH": install_dir,
    }
