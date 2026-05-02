load("@vx//stdlib:system_install.star", "cross_platform_install")
# provider.star - tilt provider
#
# tilt: A toolkit for fixing the pains of microservice development.
# Releases: https://github.com/tilt-dev/tilt/releases
#
# Asset format (dot-separated, custom OS names):
#   tilt.{version}.linux.x86_64.tar.gz
#   tilt.{version}.mac.x86_64.tar.gz
#   tilt.{version}.mac.arm64_BETA.tar.gz
#   tilt.{version}.windows.x86_64.zip
#
# Tag format: v{version}

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "tilt"
description = "tilt - A toolkit for fixing the pains of microservice development"
homepage    = "https://tilt.dev"
repository  = "https://github.com/tilt-dev/tilt"
license     = "Apache-2.0"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("tilt",
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

fetch_versions = make_fetch_versions("tilt-dev", "tilt")

# ---------------------------------------------------------------------------
# Platform helpers
#
# tilt uses dot-separated format with custom OS names:
#   linux, mac, windows
# and standard arch names:
#   x86_64
# Note: arm64 on macOS uses "arm64_BETA" suffix in older releases,
# and "arm64" in newer releases (v0.33+). We use "arm64" for modern releases.
# ---------------------------------------------------------------------------

_PLATFORMS = {
    "linux/x64":    ("linux",   "x86_64", "tar.gz"),
    "linux/arm64":  ("linux",   "arm64",  "tar.gz"),
    "macos/x64":    ("mac",     "x86_64", "tar.gz"),
    "macos/arm64":  ("mac",     "arm64",  "tar.gz"),
    "windows/x64":  ("windows", "x86_64", "zip"),
}

def _tilt_platform(ctx):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return _PLATFORMS.get(key)

# ---------------------------------------------------------------------------
# download_url
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    platform = _tilt_platform(ctx)
    if not platform:
        return None
    os_name, arch_name, ext = platform[0], platform[1], platform[2]
    asset = "tilt.{}.{}.{}.{}".format(version, os_name, arch_name, ext)
    return github_asset_url("tilt-dev", "tilt", "v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    exe = "tilt.exe" if ctx.platform.os == "windows" else "tilt"
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": [exe],
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/tilt"

def get_execute_path(ctx, _version):
    exe = "tilt.exe" if ctx.platform.os == "windows" else "tilt"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def deps(_ctx, _version):
    return []

system_install = cross_platform_install(
    windows = "tilt",
    macos   = "tilt",
    linux   = "tilt",
)
