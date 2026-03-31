# provider.star - lazydocker (Docker TUI)
#
# lazydocker: A simple TUI for Docker and docker-compose
# Releases: https://github.com/jesseduffield/lazydocker/releases
# Asset format: lazydocker_{version}_{OS}_{arch}.{ext}  (OS=Darwin/Linux/Windows, arch=x86_64/arm64)
# Tag format:   v{version}

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "path_fns",
     "fetch_versions_from_github")
load("@vx//stdlib:env.star", "env_prepend")
load("@vx//stdlib:layout.star", "archive_layout")

name        = "lazydocker"
description = "lazydocker - A simple TUI for Docker and docker-compose"
homepage    = "https://github.com/jesseduffield/lazydocker"
repository  = "https://github.com/jesseduffield/lazydocker"
license     = "MIT"
ecosystem   = "devtools"

runtimes = [runtime_def("lazydocker", version_pattern="Version:")]

permissions = github_permissions()

_PLATFORMS = {
    "windows/x64":   ("Windows", "x86_64"),
    "windows/arm64": ("Windows", "arm64"),
    "macos/x64":     ("Darwin", "x86_64"),
    "macos/arm64":   ("Darwin", "arm64"),
    "linux/x64":     ("Linux", "x86_64"),
    "linux/arm64":   ("Linux", "arm64"),
}

fetch_versions = fetch_versions_from_github("jesseduffield", "lazydocker")

def download_url(ctx, version):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    platform = _PLATFORMS.get(key)
    if not platform:
        return None
    os_str, arch_str = platform
    ext = "zip" if ctx.platform.os == "windows" else "tar.gz"
    return "https://github.com/jesseduffield/lazydocker/releases/download/v{}/lazydocker_{}_{}_{}.{}".format(
        version, version, os_str, arch_str, ext)

install_layout = archive_layout("lazydocker")

paths = path_fns("lazydocker")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []
