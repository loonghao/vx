# provider.star - kubectl provider
#
# kubectl is a single binary downloaded from dl.k8s.io (not GitHub releases).
# URL: https://dl.k8s.io/release/v{version}/bin/{os}/{arch}/kubectl[.exe]
#
# Version source: kubernetes/kubernetes releases (kubectl version matches Kubernetes version)
#
# Uses runtime_def + github_permissions from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "binary_layout", "path_fns", "path_env_fns")
load("@vx//stdlib:github.star", "github_releases", "releases_to_versions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "kubectl"
description = "kubectl - The Kubernetes command-line tool"
homepage    = "https://kubernetes.io/docs/reference/kubectl/"
repository  = "https://github.com/kubernetes/kubernetes"
license     = "Apache-2.0"
ecosystem   = "devtools"
aliases     = ["kube", "k"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("kubectl",
        aliases         = ["kube", "k"],
        version_cmd     = "{executable} version --client",
        version_pattern = "Client Version",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions(extra_hosts = ["dl.k8s.io"])

# ---------------------------------------------------------------------------
# fetch_versions — from kubernetes/kubernetes releases
# kubectl versions match Kubernetes versions (e.g., v1.31.0)
# ---------------------------------------------------------------------------

def fetch_versions(ctx):
    # Fetch releases from kubernetes/kubernetes (not kubernetes/kubectl)
    # kubectl version matches Kubernetes major.minor.patch version
    releases = github_releases(ctx, "kubernetes", "kubernetes", False)
    versions = releases_to_versions(releases)
    # Filter to only include standard Kubernetes versions (v1.x.x)
    return [v for v in versions if v["version"].startswith("1.")]

# ---------------------------------------------------------------------------
# Platform helpers
# ---------------------------------------------------------------------------

def _kubectl_platform(ctx):
    os_map   = {"windows": "windows", "macos": "darwin", "linux": "linux"}
    arch_map = {"x64": "amd64", "arm64": "arm64", "x86": "386", "arm": "arm"}
    os_str   = os_map.get(ctx.platform.os)
    arch_str = arch_map.get(ctx.platform.arch, "amd64")
    return os_str, arch_str

# ---------------------------------------------------------------------------
# download_url — dl.k8s.io single binary
# URL: https://dl.k8s.io/release/v{version}/bin/{os}/{arch}/kubectl[.exe]
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    os_str, arch_str = _kubectl_platform(ctx)
    if not os_str:
        return None
    exe = ".exe" if ctx.platform.os == "windows" else ""
    return "https://dl.k8s.io/release/v{}/bin/{}/{}/kubectl{}".format(
        version, os_str, arch_str, exe)

# ---------------------------------------------------------------------------
# Layout + path/env functions (from stdlib)
# ---------------------------------------------------------------------------

install_layout   = binary_layout("kubectl")
_paths           = path_fns("kubectl")
store_root       = _paths["store_root"]
get_execute_path = _paths["get_execute_path"]

_env             = path_env_fns()
post_install     = _env["post_install"]
environment      = _env["environment"]

def deps(_ctx, _version):
    return []
