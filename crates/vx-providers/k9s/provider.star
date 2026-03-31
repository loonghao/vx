# provider.star - k9s (Kubernetes TUI)
#
# k9s: Terminal UI to interact with Kubernetes clusters
# Releases: https://github.com/derailed/k9s/releases
# Asset format: k9s_{OS}_{arch}.{ext}  (no version in filename, OS=Darwin/Linux/Windows)
# Tag format:   v{version}

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "path_fns",
     "fetch_versions_from_github")
load("@vx//stdlib:env.star", "env_prepend")
load("@vx//stdlib:layout.star", "archive_layout")

name        = "k9s"
description = "k9s - Terminal UI to interact with Kubernetes clusters"
homepage    = "https://k9scli.io"
repository  = "https://github.com/derailed/k9s"
license     = "Apache-2.0"
ecosystem   = "devtools"

runtimes = [runtime_def("k9s", version_pattern="Version:")]

permissions = github_permissions()

_PLATFORMS = {
    "windows/x64":   ("Windows", "amd64"),
    "windows/arm64": ("Windows", "arm64"),
    "macos/x64":     ("Darwin", "amd64"),
    "macos/arm64":   ("Darwin", "arm64"),
    "linux/x64":     ("Linux", "amd64"),
    "linux/arm64":   ("Linux", "arm64"),
}

fetch_versions = fetch_versions_from_github("derailed", "k9s")

def download_url(ctx, version):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    platform = _PLATFORMS.get(key)
    if not platform:
        return None
    os_str, arch_str = platform
    ext = "zip" if ctx.platform.os == "windows" else "tar.gz"
    return "https://github.com/derailed/k9s/releases/download/v{}/k9s_{}_{}.{}".format(
        version, os_str, arch_str, ext)

install_layout = archive_layout("k9s")

paths = path_fns("k9s")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []
