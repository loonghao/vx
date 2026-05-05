# provider.star - witr provider
#
# witr: "Why is this running?" - Process introspection tool
#
# All platforms served from vx-org/mirrors GitHub Releases:
#   Windows: witr-windows-{arch}.zip  (contains witr.exe)
#   macOS:   witr-darwin-{arch}       (direct binary)
#   Linux:   witr-linux-{arch}        (direct binary)
#
# Mirror source: https://github.com/vx-org/mirrors

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "path_fns")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")

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
# Platform mapping
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
# fetch_versions — from vx-org/mirrors tags (format: witr-{version})
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("vx-org", "mirrors",
    tag_prefix = "witr-")

# ---------------------------------------------------------------------------
# download_url — all platforms from vx-org/mirrors
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    os_str   = _OS_MAP.get(ctx.platform.os)
    arch_str = _ARCH_MAP.get(ctx.platform.arch)
    if not os_str or not arch_str:
        return None
    ext = ".zip" if ctx.platform.os == "windows" else ""
    asset = "witr-{}-{}{}".format(os_str, arch_str, ext)
    return github_asset_url("vx-org", "mirrors", "witr-" + version, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    os_str   = _OS_MAP.get(ctx.platform.os, ctx.platform.os)
    arch_str = _ARCH_MAP.get(ctx.platform.arch, ctx.platform.arch)

    if ctx.platform.os == "windows":
        return {
            "__type__":         "archive",
            "source_name":      "witr.exe",
            "target_name":      "witr.exe",
            "target_dir":       "bin",
            "executable_paths": ["bin/witr.exe"],
        }
    else:
        source_name = "witr-{}-{}".format(os_str, arch_str)
        return {
            "__type__":         "binary_install",
            "source_name":      source_name,
            "target_name":      "witr",
            "target_dir":       "bin",
            "executable_paths": ["bin/witr"],
        }

# ---------------------------------------------------------------------------
# Path + env functions
# ---------------------------------------------------------------------------

paths            = path_fns("witr")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]

def environment(ctx, _version):
    return [{"op": "prepend", "key": "PATH", "value": ctx.install_dir + "/bin"}]

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []
