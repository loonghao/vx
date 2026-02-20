# provider.star - actrun provider
#
# actrun: Actionforge workflow runner CLI
# Inheritance pattern: Level 2 (custom download_url for actrun's naming)
#   - fetch_versions: inherited from github.star
#   - download_url:   custom (actrun-v{version}.cli-{arch}-{os}.{ext})
#
# actrun releases: https://github.com/actionforge/actrun-cli/releases
# Asset format: actrun-v{version}.cli-{arch}-{os}.{ext}
# Note: macOS uses .pkg which is not supported, skip macOS

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "actrun"

def description():
    return "Actionforge workflow runner CLI for executing GitHub Actions-compatible workflows locally"

def homepage():
    return "https://github.com/actionforge/actrun-cli"

def repository():
    return "https://github.com/actionforge/actrun-cli"

def license():
    return "Actionforge-EULA"

def ecosystem():
    return "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "actrun",
        "executable":  "actrun",
        "description": "Actionforge workflow runner CLI",
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
# fetch_versions — inherited
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("actionforge", "actrun-cli")

# ---------------------------------------------------------------------------
# download_url — custom
#
# actrun asset naming: actrun-v{version}.cli-{arch}-{os}.{ext}
#   actrun-v0.1.0.cli-x64-windows.zip
#   actrun-v0.1.0.cli-arm64-windows.zip
#   actrun-v0.1.0.cli-x64-linux.tar.gz
#   actrun-v0.1.0.cli-arm64-linux.tar.gz
#   actrun-v0.1.0.cli-x64-macos.pkg  (NOT supported)
#   actrun-v0.1.0.cli-arm64-macos.pkg (NOT supported)
# ---------------------------------------------------------------------------

def _actrun_platform(ctx):
    """Map platform to actrun's naming convention.

    Returns (arch, os, ext) tuple, or None if unsupported.
    Note: macOS uses .pkg which is not supported by vx-installer.
    """
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]

    platform_map = {
        "windows/x64":   ("x64",   "windows", "zip"),
        "windows/arm64": ("arm64", "windows", "zip"),
        "linux/x64":     ("x64",   "linux",   "tar.gz"),
        "linux/arm64":   ("arm64", "linux",   "tar.gz"),
        # macOS uses .pkg - not supported
    }
    return platform_map.get("{}/{}".format(os, arch))

def download_url(ctx, version):
    """Build the actrun download URL.

    Args:
        ctx:     Provider context
        version: Version string, e.g. "0.1.0"

    Returns:
        Download URL string, or None if platform is unsupported
    """
    platform = _actrun_platform(ctx)
    if not platform:
        return None

    act_arch, act_os, ext = platform
    asset = "actrun-v{}.cli-{}-{}.{}".format(version, act_arch, act_os, ext)
    tag = "v{}".format(version)
    return github_asset_url("actionforge", "actrun-cli", tag, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    os = ctx["platform"]["os"]
    exe = "actrun.exe" if os == "windows" else "actrun"
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": [exe, "actrun"],
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {
        "PATH": install_dir,
    }


# ---------------------------------------------------------------------------
# Path queries (RFC 0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    """Return the vx store root directory for actrun."""
    return "{vx_home}/store/actrun"

def get_execute_path(ctx, version):
    """Return the executable path for the given version."""
    os = ctx["platform"]["os"]
    if os == "windows":
        return "{install_dir}/actrun.exe"
    else:
        return "{install_dir}/actrun"

def post_install(ctx, version, install_dir):
    """Post-install hook (no-op for actrun)."""
    return None

# ---------------------------------------------------------------------------
# Path queries (RFC 0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    return "{vx_home}/store/actrun"

def get_execute_path(ctx, version):
    os = ctx["platform"]["os"]
    if os == "windows":
        return "{install_dir}/actrun.exe"
    else:
        return "{install_dir}/actrun"

def post_install(ctx, version, install_dir):
    return None
