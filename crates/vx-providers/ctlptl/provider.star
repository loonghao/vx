# provider.star - ctlptl provider
#
# ctlptl: Making local Kubernetes clusters fun and easy to set up.
# Releases: https://github.com/tilt-dev/ctlptl/releases
#
# Asset format (dot-separated, custom OS names):
#   ctlptl.{version}.linux.x86_64.tar.gz
#   ctlptl.{version}.linux.arm64.tar.gz
#   ctlptl.{version}.mac.x86_64.tar.gz
#   ctlptl.{version}.mac.arm64.tar.gz
#   ctlptl.{version}.windows.x86_64.zip
#
# Tag format: v{version}

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "ctlptl"
description = "ctlptl - Making local Kubernetes clusters fun and easy to set up"
homepage    = "https://github.com/tilt-dev/ctlptl"
repository  = "https://github.com/tilt-dev/ctlptl"
license     = "Apache-2.0"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("ctlptl",
        # ctlptl uses `ctlptl version` (not --version)
        version_cmd     = "{executable} version",
        version_pattern = "v?\\d+\\.\\d+\\.\\d+",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("tilt-dev", "ctlptl")

# ---------------------------------------------------------------------------
# Platform helpers
#
# ctlptl uses dot-separated format with custom OS names:
#   linux, mac, windows
# and standard arch names:
#   x86_64, arm64
# ---------------------------------------------------------------------------

_PLATFORMS = {
    "linux/x64":    ("linux",   "x86_64", "tar.gz"),
    "linux/arm64":  ("linux",   "arm64",  "tar.gz"),
    "macos/x64":    ("mac",     "x86_64", "tar.gz"),
    "macos/arm64":  ("mac",     "arm64",  "tar.gz"),
    "windows/x64":  ("windows", "x86_64", "zip"),
}

def _ctlptl_platform(ctx):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return _PLATFORMS.get(key)

# ---------------------------------------------------------------------------
# download_url
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    platform = _ctlptl_platform(ctx)
    if not platform:
        return None
    os_name, arch_name, ext = platform[0], platform[1], platform[2]
    asset = "ctlptl.{}.{}.{}.{}".format(version, os_name, arch_name, ext)
    return github_asset_url("tilt-dev", "ctlptl", "v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    exe = "ctlptl.exe" if ctx.platform.os == "windows" else "ctlptl"
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": [exe],
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/ctlptl"

def get_execute_path(ctx, _version):
    exe = "ctlptl.exe" if ctx.platform.os == "windows" else "ctlptl"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def deps(_ctx, _version):
    return []
