# provider.star - witr provider
#
# witr: "Why is this running?" - Process introspection tool
#
# Assets:
#   Windows: witr-windows-{arch}.zip (contains witr.exe)
#   macOS:   witr-darwin-{arch}    (direct binary)
#   Linux:   witr-linux-{arch}     (direct binary)
#
# NOTE: github_binary_provider template expects version in asset name.
#       witr assets DON'T have version, so we override BOTH:
#       - download_url (remove version)
#       - install_layout (template generated wrong one)
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
# Platform mapping (GitHub releases use: windows, darwin, linux)
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

# ---------------------------------------------------------------------------
# github_binary_provider template
#
# We load the template, but OVERRIDE both:
#   - download_url (witr assets DON'T have version)
#   - install_layout (template generates wrong one with version)
#
# Template asset naming: witr-{os}-{arch}-v0.3.1.{ext]  (WRONG!)
# We need:              witr-{os}-{arch}.{ext}        (CORRECT!)
# ---------------------------------------------------------------------------

_p = github_binary_provider(
    "pranshuparmar", "witr",
    asset = "witr-{os}-{arch}{ext}",
    tag_prefix = "v",
)

# Inherit MOST functions from template
fetch_versions   = _p["fetch_versions"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
post_install     = _p["post_install"]
environment      = _p["environment"]
deps             = _p["deps"]

# ---------------------------------------------------------------------------
# download_url — OVERRIDE (witr asset names DON'T have version)
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    os_str = _OS_MAP.get(ctx.platform.os)
    arch_str = _ARCH_MAP.get(ctx.platform.arch)
    if not os_str or not arch_str:
        return None
    ext = ".zip" if ctx.platform.os == "windows" else ""
    asset = "witr-{}-{}{}".format(os_str, arch_str, ext)
    return github_asset_url("pranshuparmar", "witr", "v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout — OVERRIDE (template expects version in asset name)
#
# Based on WORKING yq/provider.star:
#   - __type__ = "binary_install"
#   - target_dir = "bin"
#   - executable_paths = ["bin/" + target_name]
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    """Install layout for witr.
    Based on yq/provider.star (which WORKS on macOS/Linux).
    """
    os_str   = _OS_MAP.get(ctx.platform.os, ctx.platform.os)
    arch_str = _ARCH_MAP.get(ctx.platform.arch, ctx.platform.arch)

    source_name = "witr-{}-{}".format(os_str, arch_str)
    target_name = "witr" + (".exe" if ctx.platform.os == "windows" else "")

    return {
        "__type__":         "binary_install",
        "source_name":      source_name,
        "target_name":      target_name,
        "target_dir":       "bin",
        "executable_paths": ["bin/" + target_name],
    }
