# provider.star - skaffold provider
#
# skaffold is a command-line tool that facilitates continuous development
# for Kubernetes applications.
#
# Release assets are downloaded from Google Storage (not GitHub releases):
#   https://storage.googleapis.com/skaffold/releases/v{version}/skaffold-{os}-{arch}[.exe]
#
# OS:   linux, darwin, windows
# Arch: amd64, arm64
#
# Version source: GoogleContainerTools/skaffold releases on GitHub (tag prefix "v")

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "binary_layout",
     "fetch_versions_with_tag_prefix")
load("@vx//stdlib:env.star", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "skaffold"
description = "skaffold - Easy and Repeatable Kubernetes Development"
homepage    = "https://skaffold.dev"
repository  = "https://github.com/GoogleContainerTools/skaffold"
license     = "Apache-2.0"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("skaffold",
        version_cmd     = "{executable} version",
        version_pattern = "v?\\d+\\.\\d+\\.\\d+",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions(extra_hosts = ["storage.googleapis.com"])

# ---------------------------------------------------------------------------
# fetch_versions - from GoogleContainerTools/skaffold releases
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_with_tag_prefix(
    "GoogleContainerTools", "skaffold", tag_prefix = "v")

# ---------------------------------------------------------------------------
# Platform helpers
# ---------------------------------------------------------------------------

_PLATFORMS = {
    "windows/x64":   ("windows", "amd64"),
    "macos/x64":     ("darwin",  "amd64"),
    "macos/arm64":   ("darwin",  "arm64"),
    "linux/x64":     ("linux",   "amd64"),
    "linux/arm64":   ("linux",   "arm64"),
}

def _skaffold_platform(ctx):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return _PLATFORMS.get(key)

# ---------------------------------------------------------------------------
# download_url - from Google Storage
# URL: https://storage.googleapis.com/skaffold/releases/v{version}/skaffold-{os}-{arch}[.exe]
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    platform = _skaffold_platform(ctx)
    if not platform:
        return None
    os_str, arch_str = platform
    exe = ".exe" if ctx.platform.os == "windows" else ""
    return "https://storage.googleapis.com/skaffold/releases/v{}/skaffold-{}-{}{}".format(
        version, os_str, arch_str, exe)

# ---------------------------------------------------------------------------
# install_layout - single binary (no archive)
# ---------------------------------------------------------------------------

install_layout = binary_layout("skaffold")

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/skaffold"


def get_execute_path(ctx, _version):
    exe = "skaffold.exe" if ctx.platform.os == "windows" else "skaffold"
    return ctx.install_dir + "/bin/" + exe


def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir + "/bin")]


def post_install(_ctx, _version):
    return None


def deps(_ctx, _version):
    return []
