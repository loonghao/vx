# provider.star - nerdctl provider
#
# nerdctl is a Docker-compatible CLI for containerd.
# It is a Linux-only tool (containerd runs on Linux).
#
# Release assets (GitHub releases, containerd/nerdctl):
#   nerdctl-{version}-linux-{arch}.tar.gz  (minimal, nerdctl binary only)
#
# Arch: amd64, arm64
#
# Version source: containerd/nerdctl releases on GitHub (tag prefix "v")

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "path_fns",
     "fetch_versions_with_tag_prefix")
load("@vx//stdlib:env.star", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "nerdctl"
description = "nerdctl - Docker-compatible CLI for containerd"
homepage    = "https://github.com/containerd/nerdctl"
repository  = "https://github.com/containerd/nerdctl"
license     = "Apache-2.0"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("nerdctl",
        version_cmd     = "{executable} --version",
        version_pattern = "nerdctl version v?\\d+\\.\\d+\\.\\d+",
        platform_constraint = {"os": ["linux"]},
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions - from containerd/nerdctl releases
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_with_tag_prefix("containerd", "nerdctl", tag_prefix = "v")

# ---------------------------------------------------------------------------
# Platform helpers
# nerdctl is Linux-only; Windows/macOS return None
# ---------------------------------------------------------------------------

_PLATFORMS = {
    "linux/x64":   "amd64",
    "linux/arm64": "arm64",
}

def _nerdctl_arch(ctx):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return _PLATFORMS.get(key)

# ---------------------------------------------------------------------------
# download_url - GitHub releases
# URL: https://github.com/containerd/nerdctl/releases/download/v{version}/nerdctl-{version}-linux-{arch}.tar.gz
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    arch = _nerdctl_arch(ctx)
    if not arch:
        # nerdctl is Linux-only
        return None
    return "https://github.com/containerd/nerdctl/releases/download/v{}/nerdctl-{}-linux-{}.tar.gz".format(
        version, version, arch)

# ---------------------------------------------------------------------------
# install_layout - archive, binary at root
# ---------------------------------------------------------------------------

def install_layout(_ctx, _version):
    return {
        "__type":           "archive",
        "executable_paths": ["nerdctl"],
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

paths            = path_fns("nerdctl")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]


def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir + "/bin")]


def post_install(_ctx, _version):
    return None


def deps(_ctx, _version):
    return []
