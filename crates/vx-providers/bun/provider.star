# provider.star - bun provider
#
# Bun: Incredibly fast JavaScript runtime, bundler, test runner, and package manager
# Inheritance pattern: Level 2 (custom download_url for bun's naming convention)
#   - fetch_versions: inherited from github.star
#   - download_url:   custom (bun-{os}-{arch} naming, no version in asset name)
#
# bun releases: https://github.com/oven-sh/bun/releases
# Asset format: bun-{os}-{arch}.zip

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "bun"

def description():
    return "Incredibly fast JavaScript runtime, bundler, test runner, and package manager"

def homepage():
    return "https://bun.sh"

def repository():
    return "https://github.com/oven-sh/bun"

def license():
    return "MIT"

def ecosystem():
    return "nodejs"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "bun",
        "executable":  "bun",
        "description": "Bun JavaScript runtime",
        "aliases":     [],
        "priority":    100,
    },
    {
        "name":         "bunx",
        "executable":   "bun",
        "description":  "Bun package runner",
        "bundled_with": "bun",
        "command_prefix": ["x"],
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
# fetch_versions — inherited
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("oven-sh", "bun")

# ---------------------------------------------------------------------------
# download_url — custom
#
# bun asset naming: bun-{os}-{arch}.zip
#   bun-windows-x64.zip
#   bun-darwin-x64.zip / bun-darwin-aarch64.zip
#   bun-linux-x64.zip / bun-linux-aarch64.zip
# ---------------------------------------------------------------------------

def _bun_platform(ctx):
    """Map platform to bun's naming convention."""
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]

    platform_map = {
        "windows/x64":   ("windows", "x64"),
        "macos/x64":     ("darwin",  "x64"),
        "macos/arm64":   ("darwin",  "aarch64"),
        "linux/x64":     ("linux",   "x64"),
        "linux/arm64":   ("linux",   "aarch64"),
    }
    return platform_map.get("{}/{}".format(os, arch))

def download_url(ctx, version):
    """Build the bun download URL.

    Args:
        ctx:     Provider context
        version: Version string, e.g. "1.2.3"

    Returns:
        Download URL string, or None if platform is unsupported
    """
    platform = _bun_platform(ctx)
    if not platform:
        return None

    bun_os, bun_arch = platform
    asset = "bun-{}-{}.zip".format(bun_os, bun_arch)
    tag = "bun-v{}".format(version)
    return github_asset_url("oven-sh", "bun", tag, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    platform = _bun_platform(ctx)
    os = ctx["platform"]["os"]
    exe = "bun.exe" if os == "windows" else "bun"

    if platform:
        bun_os, bun_arch = platform
        strip_prefix = "bun-{}-{}".format(bun_os, bun_arch)
    else:
        strip_prefix = ""

    return {
        "type":             "archive",
        "strip_prefix":     strip_prefix,
        "executable_paths": [exe, "bun"],
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {
        "PATH": install_dir,
    }
