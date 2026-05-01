# provider.star - age provider
#
# age is a simple, modern and secure encryption tool (Go).
#
# Release assets (GitHub releases):
#   age-v{version}-{os}-{arch}.tar.gz
#
# OS:   linux, darwin, windows, freebsd
# Arch: amd64, arm64, arm
#
# Version source: FiloSottile/age releases on GitHub (tag prefix "v")

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:layout.star", "path_fns", "path_env_fns")
load("@vx//stdlib:system_install.star", "cross_platform_install")

# Use stdlib path_fns for correct cross-platform path handling
paths = path_fns("age")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]

# path_fns does NOT export "environment" – use path_env_fns for PATH prepend
_env_fns   = path_env_fns()
environment = _env_fns["environment"]

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "age"
description = "age - A simple, modern and secure encryption tool"
homepage    = "https://age-encryption.org/"
repository  = "https://github.com/FiloSottile/age"
license     = "BSD-3-Clause"
ecosystem   = "crypto"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("age",
        version_pattern = "\\d+\\.\\d+\\.\\d+",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("FiloSottile", "age")

# ---------------------------------------------------------------------------
# Platform helpers
# ---------------------------------------------------------------------------

_PLATFORMS = {
    "linux/x64":   ("linux",   "amd64"),
    "linux/arm64": ("linux",   "arm64"),
    "linux/arm":   ("linux",   "arm"),
    "macos/x64":   ("darwin",  "amd64"),
    "macos/arm64": ("darwin", "arm64"),
    "windows/x64": ("windows", "amd64"),
    "windows/arm64": ("windows", "arm64"),
    "freebsd/x64": ("freebsd", "amd64"),
}

def _age_platform(ctx):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return _PLATFORMS.get(key)

# ---------------------------------------------------------------------------
# download_url
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    platform = _age_platform(ctx)
    if not platform:
        return None
    os_str, arch_str = platform
    ext = "zip" if ctx.platform.os == "windows" else "tar.gz"
    asset = "age-v{}-{}-{}.{}".format(version, os_str, arch_str, ext)
    return github_asset_url("FiloSottile", "age", "v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(_ctx, _version):
    return {
        "__type":           "archive",
        "strip_prefix":     "age",
        "executable_paths": ["age"],
    }

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []

# system_install fallback when GitHub download is unavailable
system_install = cross_platform_install(
    windows = "age",
    macos   = "age",
    linux   = "age",
)
