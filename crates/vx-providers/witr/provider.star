# provider.star - witr provider
#
# witr: "Why is this running?" - Process introspection tool
#
# Assets:
#   Windows: witr-windows-{arch}.zip (contains witr.exe)
#   macOS:   witr-darwin-{arch}    (direct binary)
#   Linux:   witr-linux-{arch}     (direct binary)
#
# Use github_binary_provider template (handles direct binaries correctly)
# Only override download_url (asset name has no version)
#

load("@vx//stdlib:provider.star",
     "github_binary_provider", "runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "github_asset_url")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "witr"
description = "witr - Why is this running? Process introspection tool"
homepage    = "https://github.com/pranshuparmar/witr"
repository  = "https://github.com/pranshuparmar/witr"
license     = "MIT"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("witr",
        version_pattern = "witr v\\d+\\.\\d+\\.\\d+",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# github_binary_provider template
#
# Asset naming: witr-{os}-{arch}[.zip]
# - os: windows, darwin, linux (matches GitHub releases)
# - arch: amd64, arm64
# ---------------------------------------------------------------------------

_OS_MAP = {
    "windows": "windows",
    "macos":   "darwin",
    "linux":   "linux",
}

_ARCH_MAP = {
    "x64":   "amd64",
    "arm64": "arm64",
}

_p = github_binary_provider(
    "pranshuparmar", "witr",
    asset = "witr-{os}-{arch}{ext}",
    tag_prefix = "v",
)

# Inherit ALL functions from template (including install_layout)
fetch_versions   = _p["fetch_versions"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
post_install     = _p["post_install"]
environment      = _p["environment"]
deps             = _p["deps"]

# ---------------------------------------------------------------------------
# download_url — ONLY override (witr asset names don't have version)
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    os_str = _OS_MAP.get(ctx.platform.os)
    arch_str = _ARCH_MAP.get(ctx.platform.arch)
    if not os_str or not arch_str:
        return None
    ext = ".zip" if ctx.platform.os == "windows" else ""
    asset = "witr-{}-{}{}".format(os_str, arch_str, ext)
    return github_asset_url("pranshuparmar", "witr", "v" + version, asset)
