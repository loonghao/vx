# provider.star - Flux provider
#
# Flux is a tool for keeping Kubernetes clusters in sync with sources of
# configuration (like Git repositories), and automating updates to that
# configuration when there is new code to deploy.
#
# Release assets use goreleaser format:
#   flux_{version}_{os}_{arch}.{ext}
# e.g. flux_2.8.5_linux_amd64.tar.gz, flux_2.8.5_windows_amd64.zip
#
# Archive contains: flux[.exe] at the root

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "make_fetch_versions")
load("@vx//stdlib:env.star", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "flux"
description = "Flux - GitOps toolkit for keeping Kubernetes clusters in sync"
homepage    = "https://fluxcd.io"
repository  = "https://github.com/fluxcd/flux2"
license     = "Apache-2.0"
ecosystem   = "devtools"
aliases     = ["flux2"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("flux",
        aliases         = ["flux2"],
        version_cmd     = "{executable} version --client",
        version_pattern = "flux: v",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions — from fluxcd/flux2 releases
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("fluxcd", "flux2")

# ---------------------------------------------------------------------------
# Platform helpers
# flux uses goreleaser os/arch naming: linux/amd64, darwin/arm64, windows/amd64
# ---------------------------------------------------------------------------

def _flux_platform(ctx):
    os_map   = {"windows": "windows", "macos": "darwin", "linux": "linux"}
    arch_map = {"x64": "amd64", "arm64": "arm64", "arm": "arm"}
    os_str   = os_map.get(ctx.platform.os)
    arch_str = arch_map.get(ctx.platform.arch, "amd64")
    return os_str, arch_str

# ---------------------------------------------------------------------------
# download_url — GitHub releases
# URL: https://github.com/fluxcd/flux2/releases/download/v{version}/flux_{version}_{os}_{arch}.{ext}
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    os_str, arch_str = _flux_platform(ctx)
    if not os_str:
        return None
    ext = "zip" if ctx.platform.os == "windows" else "tar.gz"
    filename = "flux_{}_{}_{}{}".format(version, os_str, arch_str, "." + ext)
    return "https://github.com/fluxcd/flux2/releases/download/v{}/{}".format(version, filename)

# ---------------------------------------------------------------------------
# install_layout — archive contains flux[.exe] at root (no strip_prefix)
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    exe = "flux.exe" if ctx.platform.os == "windows" else "flux"
    return {
        "type":             "archive",
        "executable_paths": [exe, "flux"],
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/flux"


def get_execute_path(ctx, _version):
    exe = "flux.exe" if ctx.platform.os == "windows" else "flux"
    return ctx.install_dir + "/" + exe


def post_install(_ctx, _version):
    return None


def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]


def deps(_ctx, _version):
    return []
