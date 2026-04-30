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
load("@vx//stdlib:env.star", "env_prepend")

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

# ---------------------------------------------------------------------------
# Path + env functions
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/age"

def get_execute_path(ctx, _version):
    exe = "age.exe" if ctx.platform.os == "windows" else "age"
    return ctx.install_dir + "/" + exe

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []
