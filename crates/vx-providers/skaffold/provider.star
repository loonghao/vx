# provider.star - Skaffold provider
#
# Skaffold is a command-line tool for continuous development of Kubernetes applications.
# Releases are hosted on Google Cloud Storage (not GitHub releases directly):
#   https://storage.googleapis.com/skaffold/releases/v{version}/skaffold-{os}-{arch}[.exe]
#
# Version info is still fetched from GitHub releases:
#   https://github.com/GoogleContainerTools/skaffold/releases
#
# Asset naming: skaffold-{os}-{arch}[.exe]
# e.g. skaffold-linux-amd64, skaffold-darwin-arm64, skaffold-windows-amd64.exe
#
# Uses binary_layout (single binary, no archive).

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "binary_layout")
load("@vx//stdlib:github.star", "make_fetch_versions")
load("@vx//stdlib:env.star", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "skaffold"
description = "Skaffold - Easy and Repeatable Kubernetes Development"
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
        version_pattern = "skaffold/",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# Skaffold binaries are served from storage.googleapis.com
# ---------------------------------------------------------------------------

permissions = github_permissions(extra_hosts = ["storage.googleapis.com"])

# ---------------------------------------------------------------------------
# fetch_versions — from GoogleContainerTools/skaffold releases
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("GoogleContainerTools", "skaffold")

# ---------------------------------------------------------------------------
# Platform helpers
# skaffold uses os/arch format: linux/amd64, darwin/arm64, windows/amd64
# ---------------------------------------------------------------------------

def _skaffold_platform(ctx):
    os_map   = {"windows": "windows", "macos": "darwin", "linux": "linux"}
    arch_map = {"x64": "amd64", "arm64": "arm64"}
    os_str   = os_map.get(ctx.platform.os)
    arch_str = arch_map.get(ctx.platform.arch, "amd64")
    return os_str, arch_str

# ---------------------------------------------------------------------------
# download_url — from Google Cloud Storage (not GitHub releases)
# URL: https://storage.googleapis.com/skaffold/releases/v{version}/skaffold-{os}-{arch}[.exe]
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    os_str, arch_str = _skaffold_platform(ctx)
    if not os_str:
        return None
    exe = ".exe" if ctx.platform.os == "windows" else ""
    return "https://storage.googleapis.com/skaffold/releases/v{}/skaffold-{}-{}{}".format(
        version, os_str, arch_str, exe)

# ---------------------------------------------------------------------------
# Layout + path/env functions
# skaffold is a single binary, no archive
# ---------------------------------------------------------------------------

install_layout = binary_layout("skaffold")


def store_root(ctx):
    return ctx.vx_home + "/store/skaffold"


def get_execute_path(ctx, _version):
    exe = "skaffold.exe" if ctx.platform.os == "windows" else "skaffold"
    return ctx.install_dir + "/" + exe


def post_install(_ctx, _version):
    return None


def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]


def deps(_ctx, _version):
    return []
