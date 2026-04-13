# provider.star - grpcurl provider
#
# grpcurl is a command-line tool that lets you interact with gRPC servers.
# It is basically curl for gRPC servers.
#
# Release assets (GitHub releases, goreleaser with custom naming):
#   grpcurl_{version}_{os}_{arch}.tar.gz  (Linux/macOS)
#   grpcurl_{version}_{os}_{arch}.zip     (Windows)
#
# OS:   linux, osx, windows  (NOTE: macOS uses "osx", not "darwin")
# Arch: x86_64, arm64        (NOTE: x64 uses "x86_64", not "amd64")
#
# Version source: fullstorydev/grpcurl releases on GitHub (tag prefix "v")

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "path_fns")
load("@vx//stdlib:github.star", "make_fetch_versions")
load("@vx//stdlib:env.star", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "grpcurl"
description = "grpcurl - Like curl, but for gRPC: command-line tool for gRPC servers"
homepage    = "https://github.com/fullstorydev/grpcurl"
repository  = "https://github.com/fullstorydev/grpcurl"
license     = "MIT"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("grpcurl",
        version_cmd     = "{executable} version",
        version_pattern = "grpcurl v\\d+\\.\\d+\\.\\d+",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions - from fullstorydev/grpcurl releases
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("fullstorydev", "grpcurl")

# ---------------------------------------------------------------------------
# Platform helpers
# grpcurl uses non-standard naming: "osx" (not "darwin"), "x86_64" (not "amd64")
# ---------------------------------------------------------------------------

_PLATFORMS = {
    "windows/x64":   ("windows", "x86_64"),
    "macos/x64":     ("osx",    "x86_64"),
    "macos/arm64":   ("osx",    "arm64"),
    "linux/x64":     ("linux",  "x86_64"),
    "linux/arm64":   ("linux",  "arm64"),
}

def _grpcurl_platform(ctx):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return _PLATFORMS.get(key)

# ---------------------------------------------------------------------------
# download_url - GitHub releases
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    platform = _grpcurl_platform(ctx)
    if not platform:
        return None
    os_str, arch_str = platform
    ext = "zip" if ctx.platform.os == "windows" else "tar.gz"
    return "https://github.com/fullstorydev/grpcurl/releases/download/v{}/grpcurl_{}_{}_{}.{}".format(
        version, version, os_str, arch_str, ext)

# ---------------------------------------------------------------------------
# install_layout - archive containing grpcurl binary
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    exe = "grpcurl.exe" if ctx.platform.os == "windows" else "grpcurl"
    return {
        "__type":           "archive",
        "executable_paths": [exe],
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

paths            = path_fns("grpcurl")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []
