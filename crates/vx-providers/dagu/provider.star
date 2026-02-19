# provider.star - Dagu provider
#
# Dagu is a powerful DAG (Directed Acyclic Graph) workflow engine.
# Asset naming: dagu_{version}_{os}_{arch}.tar.gz
# All platforms use tar.gz.
#
# Inheritance pattern (Level 2):
#   - fetch_versions: fully inherited from github.star
#   - download_url:   overridden (go-style os/arch naming, not Rust triple)

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "dagu"

def description():
    return "Dagu - A powerful DAG workflow engine with a Web UI"

def homepage():
    return "https://dagu.run"

def repository():
    return "https://github.com/dagu-org/dagu"

def license():
    return "GPL-3.0"

def ecosystem():
    return "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "dagu",
        "executable":  "dagu",
        "description": "Dagu - DAG workflow engine",
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

fetch_versions = make_fetch_versions("dagu-org", "dagu", include_prereleases = False)

# ---------------------------------------------------------------------------
# download_url — custom override
#
# Asset pattern: dagu_{version}_{os}_{arch}.tar.gz
# e.g. dagu_1.14.5_linux_amd64.tar.gz
# ---------------------------------------------------------------------------

def _dagu_platform(ctx):
    """Map platform to dagu's os/arch naming."""
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]

    platforms = {
        "windows/x64":  ("windows", "amd64"),
        "windows/arm64": ("windows", "arm64"),
        "macos/x64":    ("darwin",  "amd64"),
        "macos/arm64":  ("darwin",  "arm64"),
        "linux/x64":    ("linux",   "amd64"),
        "linux/arm64":  ("linux",   "arm64"),
    }
    return platforms.get("{}/{}".format(os, arch))

def download_url(ctx, version):
    """Build the dagu download URL.

    Args:
        ctx:     Provider context
        version: Version string WITHOUT 'v' prefix, e.g. "1.14.5"

    Returns:
        Download URL string, or None if platform is unsupported
    """
    platform = _dagu_platform(ctx)
    if not platform:
        return None

    os_name, arch_name = platform[0], platform[1]

    # Asset: "dagu_1.14.5_linux_amd64.tar.gz"
    asset = "dagu_{}_{}_{}.tar.gz".format(version, os_name, arch_name)
    tag   = "v{}".format(version)

    return github_asset_url("dagu-org", "dagu", tag, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    os  = ctx["platform"]["os"]
    exe = "dagu.exe" if os == "windows" else "dagu"
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": [exe, "dagu"],
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {
        "PATH": install_dir,
    }
