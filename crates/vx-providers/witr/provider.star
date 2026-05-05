# provider.star - witr provider
#
# witr: "Why is this running?" - Process introspection tool
# Asset: witr-{os}-{arch}[.zip]  (direct binary, no version in filename)
#
# Inheritance level: 2 (partial override)
#   - fetch_versions: fully inherited from github_binary_provider
#   - download_url:   overridden — witr asset names don't have version
#   - install_layout: overridden — rename downloaded binary to witr[.exe]
#
# Release URL format:
#   https://github.com/pranshuparmar/witr/releases/download/v{version}/witr-{os}-{arch}[.zip]

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
        version_pattern = "witr version \\d+\\.\\d+\\.\\d+",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# Provider template — github_binary_provider
#
# yq asset naming: yq_{os}_{arch}[.exe]
# - os: windows, darwin, linux
# - arch: amd64, arm64, arm, 386
# Note: uses Go-style os naming (darwin, not macos)
# ---------------------------------------------------------------------------

_OS_MAP = {
    "windows": "windows",
    "macos":   "darwin",
    "linux":   "linux",
}

_ARCH_MAP = {
    "x64":   "amd64",
    "arm64": "arm64",
    "x86":   "386",
    "arm":   "arm",
}

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
# download_url — custom override
#
# witr asset names don't have version in filename.
# Asset: witr-{os}-{arch}[.zip]
# - os: windows, darwin, linux, freebsd
# - arch: amd64, arm64
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
# install_layout — binary, rename witr-{os}-{arch}[.zip] → witr[.exe]
# Note: binary goes to root dir (not bin/), matching witr's .zip layout
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    source_name = "witr" + (".exe" if ctx.platform.os == "windows" else "")
    target_name = source_name

    return {
        "__type":           "binary_install",
        "source_name":      source_name,
        "target_name":      target_name,
        "target_dir":       "",
        "executable_paths": [target_name],
    }
