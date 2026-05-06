load("@vx//stdlib:system_install.star", "cross_platform_install")
# provider.star - Helm provider
#
# Helm releases are hosted on get.helm.sh (not GitHub releases).
# URL: https://get.helm.sh/helm-v{version}-{os}-{arch}.{ext}
# Archive layout: {os}-{arch}/helm[.exe]
#
# Uses runtime_def + github_permissions from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "make_fetch_versions")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "helm"
description = "Helm - The Kubernetes Package Manager"
homepage    = "https://helm.sh"
repository  = "https://github.com/helm/helm"
license     = "Apache-2.0"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("helm",
        version_cmd     = "{executable} version",
        version_pattern = "version\\.BuildInfo",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions(extra_hosts = ["get.helm.sh"])

# ---------------------------------------------------------------------------
# fetch_versions — GitHub releases
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("helm", "helm")

# ---------------------------------------------------------------------------
# Platform helpers
# ---------------------------------------------------------------------------

def _helm_platform(ctx):
    os_map   = {"windows": "windows", "macos": "darwin", "linux": "linux"}
    arch_map = {"x64": "amd64", "arm64": "arm64", "x86": "386", "arm": "arm"}
    return os_map.get(ctx.platform.os, "linux"), arch_map.get(ctx.platform.arch, "amd64")

# ---------------------------------------------------------------------------
# download_url — get.helm.sh
# URL: https://get.helm.sh/helm-v{version}-{os}-{arch}.{ext}
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    os_str, arch_str = _helm_platform(ctx)
    ext = "zip" if ctx.platform.os == "windows" else "tar.gz"
    return "https://get.helm.sh/helm-v{}-{}-{}.{}".format(version, os_str, arch_str, ext)

# ---------------------------------------------------------------------------
# install_layout — archive contains {os}-{arch}/helm[.exe]
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    os_str, arch_str = _helm_platform(ctx)
    exe = "helm.exe" if ctx.platform.os == "windows" else "helm"
    return {
        "type":             "archive",
        "strip_prefix":     "{}-{}".format(os_str, arch_str),
        "executable_paths": [exe, "helm"],
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/helm"

def get_execute_path(ctx, _version):
    exe = "helm.exe" if ctx.platform.os == "windows" else "helm"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def deps(_ctx, _version):
    return []

system_install = cross_platform_install(
    windows = "helm",
    macos   = "helm",
    linux   = "helm",
)
