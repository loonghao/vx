# provider.star - pnpm provider
#
# pnpm: Fast, disk space efficient package manager
# Inheritance pattern: Level 2 (custom download_url for pnpm's naming)
#   - fetch_versions: inherited from github.star
#   - download_url:   custom (pnpm-{os}-{arch} naming)
#   - deps:           version-based Node.js dependency
#
# pnpm releases: https://github.com/pnpm/pnpm/releases
# Asset format: pnpm-{os}-{arch}[.exe]  (direct binary)

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "pnpm"

def description():
    return "Fast, disk space efficient package manager"

def homepage():
    return "https://pnpm.io"

def repository():
    return "https://github.com/pnpm/pnpm"

def license():
    return "MIT"

def ecosystem():
    return "nodejs"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "pnpm",
        "executable":  "pnpm",
        "description": "pnpm package manager",
        "aliases":     ["pnpx"],
        "priority":    85,
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

fetch_versions = make_fetch_versions("pnpm", "pnpm")

# ---------------------------------------------------------------------------
# download_url — custom
#
# pnpm asset naming: pnpm-{os}-{arch}[.exe]
#   pnpm-win-x64.exe
#   pnpm-macos-x64 / pnpm-macos-arm64
#   pnpm-linux-x64 / pnpm-linux-arm64
# ---------------------------------------------------------------------------

def _pnpm_platform(ctx):
    """Map platform to pnpm's naming convention."""
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]

    platform_map = {
        "windows/x64":   ("win",   "x64"),
        "macos/x64":     ("macos", "x64"),
        "macos/arm64":   ("macos", "arm64"),
        "linux/x64":     ("linux", "x64"),
        "linux/arm64":   ("linux", "arm64"),
    }
    return platform_map.get("{}/{}".format(os, arch))

def download_url(ctx, version):
    """Build the pnpm download URL.

    Args:
        ctx:     Provider context
        version: Version string, e.g. "9.15.0"

    Returns:
        Download URL string, or None if platform is unsupported
    """
    platform = _pnpm_platform(ctx)
    if not platform:
        return None

    pnpm_os, pnpm_arch = platform
    os = ctx["platform"]["os"]

    if os == "windows":
        asset = "pnpm-{}-{}.exe".format(pnpm_os, pnpm_arch)
    else:
        asset = "pnpm-{}-{}".format(pnpm_os, pnpm_arch)

    tag = "v{}".format(version)
    return github_asset_url("pnpm", "pnpm", tag, asset)

# ---------------------------------------------------------------------------
# deps — version-based Node.js dependency
# ---------------------------------------------------------------------------

def deps(ctx, version):
    """Declare Node.js dependency based on pnpm version.

    pnpm 7.x requires Node.js 14+
    pnpm 8.x requires Node.js 16+
    pnpm 9.x requires Node.js 18+
    """
    parts = version.split(".")
    major = int(parts[0]) if parts else 0

    if major >= 9:
        return [{"runtime": "node", "version": ">=18"}]
    elif major >= 8:
        return [{"runtime": "node", "version": ">=16"}]
    else:
        return [{"runtime": "node", "version": ">=14"}]

# ---------------------------------------------------------------------------
# install_layout — binary (single file)
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    os = ctx["platform"]["os"]
    exe = "pnpm.exe" if os == "windows" else "pnpm"
    return {
        "type":             "binary",
        "target_name":      exe,
        "target_dir":       "bin",
        "target_permissions": "755",
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {
        "PATH": install_dir + "/bin",
    }
