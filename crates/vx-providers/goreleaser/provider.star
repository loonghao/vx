# provider.star - goreleaser provider
#
# goreleaser: Release engineering, simplified.
# Releases: https://github.com/goreleaser/goreleaser/releases
#
# Asset format (no version in filename, uses capitalised OS + x86_64 naming):
#   goreleaser_Linux_x86_64.tar.gz
#   goreleaser_Linux_arm64.tar.gz
#   goreleaser_Darwin_x86_64.tar.gz
#   goreleaser_Darwin_arm64.tar.gz
#   goreleaser_Windows_x86_64.zip
#   goreleaser_Windows_arm64.zip
#
# Tag format: v{version}

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "goreleaser"
description = "goreleaser - Release engineering, simplified for Go projects"
homepage    = "https://goreleaser.com"
repository  = "https://github.com/goreleaser/goreleaser"
license     = "MIT"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("goreleaser",
        # goreleaser --version outputs:
        #   GitVersion:    2.15.2
        #   GitCommit:     ...
        # So we match the version number after "GitVersion:"
        version_pattern = "\\d+\.\\d+\.\\d+",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("goreleaser", "goreleaser")

# ---------------------------------------------------------------------------
# Platform helpers
#
# goreleaser uses capitalised OS names (Linux, Darwin, Windows)
# and x86_64 (not amd64) for 64-bit x86.  No version in the filename.
# ---------------------------------------------------------------------------

_ARCH_MAP = {
    "x64":   "x86_64",
    "arm64": "arm64",
}

_OS_MAP = {
    "linux":   "Linux",
    "macos":   "Darwin",
    "windows": "Windows",
}

def _goreleaser_platform(ctx):
    os_name   = _OS_MAP.get(ctx.platform.os)
    arch_name = _ARCH_MAP.get(ctx.platform.arch)
    if not os_name or not arch_name:
        return None
    ext = "zip" if ctx.platform.os == "windows" else "tar.gz"
    return (os_name, arch_name, ext)

# ---------------------------------------------------------------------------
# download_url
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    platform = _goreleaser_platform(ctx)
    if not platform:
        return None
    os_name, arch_name, ext = platform[0], platform[1], platform[2]
    asset = "goreleaser_{}_{}.{}".format(os_name, arch_name, ext)
    return github_asset_url("vx-org", "mirrors", "goreleaser-" + version, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    exe = "goreleaser.exe" if ctx.platform.os == "windows" else "goreleaser"
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": [exe],
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/goreleaser"

def get_execute_path(ctx, _version):
    exe = "goreleaser.exe" if ctx.platform.os == "windows" else "goreleaser"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def deps(_ctx, _version):
    return []
