# provider.star - yq provider
#
# yq: portable command-line YAML, JSON, XML, CSV, TOML processor
# Asset: yq_{os}_{arch}[.exe]  (direct binary, no archive)
#
# Inheritance level: 2 (partial override)
#   - fetch_versions: fully inherited from github_binary_provider
#   - download_url:   overridden — yq uses Go-style os naming (darwin, not macos)
#   - install_layout: overridden — rename downloaded binary to yq[.exe]
#
# Release URL format:
#   https://github.com/mikefarah/yq/releases/download/v{version}/yq_{os}_{arch}[.exe]

load("@vx//stdlib:provider.star",
     "github_binary_provider", "runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "github_asset_url")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "yq"
description = "yq - a portable command-line YAML, JSON, XML, CSV, TOML and properties processor"
homepage    = "https://github.com/mikefarah/yq"
repository  = "https://github.com/mikefarah/yq"
license     = "MIT"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("yq",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "yq \\(https://github.com/mikefarah/yq\\) version"},
        ],
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
    "mikefarah", "yq",
    asset = "yq_{os}_{arch}{exe}",
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
# yq uses Go-style os naming (darwin, not macos) and Go-style arch naming
# (amd64, arm64, 386, arm).
# Asset: yq_{os}_{arch}[.exe]
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    os_str = _OS_MAP.get(ctx.platform.os)
    arch_str = _ARCH_MAP.get(ctx.platform.arch)
    if not os_str or not arch_str:
        return None
    ext = ".exe" if ctx.platform.os == "windows" else ""
    asset = "yq_{}_{}{}".format(os_str, arch_str, ext)
    return github_asset_url("mikefarah", "yq", "v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout — binary, rename yq_{os}_{arch}[.exe] → bin/yq[.exe]
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    os_str   = _OS_MAP.get(ctx.platform.os, ctx.platform.os)
    arch_str = _ARCH_MAP.get(ctx.platform.arch, ctx.platform.arch)
    ext      = ".exe" if ctx.platform.os == "windows" else ""

    source_name = "yq_{}_{}{}".format(os_str, arch_str, ext)
    target_name = "yq" + ext

    return {
        "source_name":      source_name,
        "target_name":      target_name,
        "target_dir":       "bin",
        "executable_paths": ["bin/" + target_name],
    }
