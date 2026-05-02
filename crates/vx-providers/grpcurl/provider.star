load("@vx//stdlib:system_install.star", "cross_platform_install")
# provider.star - grpcurl provider
#
# grpcurl is a command-line tool that lets you interact with gRPC servers.
# It is basically curl for gRPC servers.
#
# Release assets (GitHub releases, goreleaser format):
#   grpcurl_{version}_{os}_{arch}.tar.gz  (Linux/macOS)
#   grpcurl_{version}_{os}_{arch}.zip     (Windows)
#
# NOTE: grpcurl uses x86_64 (not amd64) for the arch in asset names.
#       macOS assets use "osx" (not "darwin") in the asset name.
#
# OS:   linux, osx, windows
# Arch: x86_64, arm64  (NOT amd64 for x86_64)
#
# Version source: fullstorydev/grpcurl releases on GitHub (tag prefix "v")

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "path_fns")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

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
        version_cmd     = "{executable} -version",
        version_pattern = "grpcurl(?:\\.exe)? v(\\d+\\.\\d+\\.\\d+)",
        test_commands = [
            {"command": "{executable} -version", "name": "version_check",
             "expected_output": "grpcurl(?:\\.exe)? v\\d+\\.\\d+\\.\\d+"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("fullstorydev", "grpcurl")

# ---------------------------------------------------------------------------
# Platform helpers
#
# grpcurl uses x86_64 (not amd64) for arch names — goreleaser default.
# ---------------------------------------------------------------------------

_PLATFORMS = {
    "linux/x64":    ("linux",   "x86_64", "tar.gz"),
    "linux/arm64":  ("linux",   "arm64",  "tar.gz"),
    "macos/x64":    ("osx",     "x86_64", "tar.gz"),
    "macos/arm64":  ("osx",     "arm64",  "tar.gz"),
    "windows/x64":  ("windows", "x86_64", "zip"),
}

def _grpcurl_platform(ctx):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return _PLATFORMS.get(key)

# ---------------------------------------------------------------------------
# download_url
#
# macOS binaries exist in the GitHub releases (named with "osx"), but we
# prefer Homebrew on macOS for better integration. Returning None here causes
# vx to fall back to system_install (brew).
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    platform = _grpcurl_platform(ctx)
    if not platform:
        return None
    os_name, arch_name, ext = platform[0], platform[1], platform[2]
    asset = "grpcurl_{}_{}_{}.{}".format(version, os_name, arch_name, ext)
    return github_asset_url("fullstorydev", "grpcurl", "v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    exe = "grpcurl.exe" if ctx.platform.os == "windows" else "grpcurl"
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": [exe],
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

paths            = path_fns("grpcurl")
store_root       = paths["store_root"]

def get_execute_path(ctx, _version):
    exe = "grpcurl.exe" if ctx.platform.os == "windows" else "grpcurl"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def deps(_ctx, _version):
    return []

system_install = cross_platform_install(
    windows = "grpcurl",
    macos   = "grpcurl",
    linux   = "grpcurl",
)
