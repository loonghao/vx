# provider.star - gping (graphical ping utility)
#
# gping: Ping with a graph
# Releases: https://github.com/orf/gping/releases
# Asset format: gping-{OS}-{libc_arch}.{ext}  (custom naming per platform)
# Tag format:   gping-v{version}

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "path_fns",
     "fetch_versions_with_tag_prefix")
load("@vx//stdlib:env.star", "env_prepend")
load("@vx//stdlib:layout.star", "archive_layout")

name        = "gping"
description = "gping - Ping with a graph"
homepage    = "https://github.com/orf/gping"
repository  = "https://github.com/orf/gping"
license     = "MIT"
ecosystem   = "devtools"

runtimes = [runtime_def("gping", version_pattern="gping \\d+")]

permissions = github_permissions()

_PLATFORMS = {
    "windows/x64":   ("Windows", "msvc-x86_64"),
    "macos/x64":     ("macOS", "x86_64"),
    "macos/arm64":   ("macOS", "arm64"),
    "linux/x64":     ("Linux", "gnu-x86_64"),
    "linux/arm64":   ("Linux", "gnu-arm64"),
}

fetch_versions = fetch_versions_with_tag_prefix("orf", "gping", tag_prefix="gping-v")

def download_url(ctx, version):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    platform = _PLATFORMS.get(key)
    if not platform:
        return None
    os_str, arch_str = platform
    ext = "zip" if ctx.platform.os == "windows" else "tar.gz"
    return "https://github.com/orf/gping/releases/download/gping-v{}/gping-{}-{}.{}".format(
        version, os_str, arch_str, ext)

install_layout = archive_layout("gping")

paths = path_fns("gping")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []
