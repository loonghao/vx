# provider.star - witr provider
#
# witr: "Why is this running?" - Process introspection tool
#
# Assets:
#   Windows: witr-windows-{arch}.zip (contains witr.exe)
#   macOS:   witr-darwin-{arch}    (direct binary)
#   Linux:   witr-linux-{arch}     (direct binary)
#
# Uses github_binary_provider template
# Override download_url (asset name has no version)
# Override install_layout (template expects version in asset name)
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
# github_binary_provider template
#
# We use the template but OVERRIDE both download_url and install_layout.
# Template expects asset name: witr-{os}-{arch}-v0.3.1.{ext]
# Actual asset name:   witr-{os}-{arch}.{ext}
# ---------------------------------------------------------------------------

_p = github_binary_provider(
    "pranshuparmar", "witr",
    asset = "witr-{os}-{arch}{ext}",
    tag_prefix = "v",
)

# Inherit unmodified functions from template
fetch_versions   = _p["fetch_versions"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
post_install     = _p["post_install"]
environment      = _p["environment"]
deps             = _p["deps"]

# ---------------------------------------------------------------------------
# download_url — OVERRIDE (witr asset names don't have version)
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
# For DIRECT BINARIES (macOS/Linux):
#   __type__ = "binary_install"
#   source_name = "witr-{os}-{arch}"  (downloaded filename)
#   target_name = "witr"                   (installed filename)
#
# For ZIP ARCHIVES (Windows):
#   __type__ = "archive"
#   source_name = "witr.exe"             (inside .zip)
#   target_name = "witr.exe"             (installed filename)
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    if ctx.platform.os == "windows":
        # .zip archive: binary inside is witr.exe
        return {
            "__type__":           "archive",
            "source_name":      "witr.exe",
            "target_name":      "witr.exe",
            "target_dir":       "",
            "executable_paths": ["witr.exe"],
        }
    else:
        # Direct binary (macOS/Linux)
        os_str = _OS_MAP[ctx.platform.os]
        arch_str = _ARCH_MAP[ctx.platform.arch]
        source_name = "witr-{}-{}".format(os_str, arch_str)
        return {
            "__type__":           "binary_install",
            "source_name":      source_name,
            "target_name":      "witr",
            "target_dir":       "",
            "executable_paths": ["witr"],
        }
