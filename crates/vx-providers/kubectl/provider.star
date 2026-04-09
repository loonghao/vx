# provider.star - kubectl provider
#
# kubectl is a single binary downloaded from dl.k8s.io (not GitHub releases).
# URL: https://dl.k8s.io/release/v{version}/bin/{os}/{arch}/kubectl[.exe]
#
# Version source: kubernetes/kubernetes releases (kubectl version matches Kubernetes version)
#
# Uses runtime_def + github_permissions from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "binary_layout")
load("@vx//stdlib:env.star", "env_prepend")
load("@vx//stdlib:github.star", "make_fetch_versions")

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

fetch_versions = make_fetch_versions("kubernetes", "kubernetes")

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
# Layout + path/env functions
# ---------------------------------------------------------------------------

install_layout = binary_layout("kubectl")


def store_root(ctx):
    return ctx.vx_home + "/store/kubectl"


def get_execute_path(ctx, _version):
    exe = "kubectl.exe" if ctx.platform.os == "windows" else "kubectl"
    return ctx.install_dir + "/bin/" + exe


def post_install(_ctx, _version):
    return None


def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir + "/bin")]


def deps(_ctx, _version):
    return []
