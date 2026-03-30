# provider.star - duf (disk usage utility)
#
# duf: A better df alternative
# Releases: https://github.com/muesli/duf/releases
# Asset format: duf_{version}_{os}_{arch}.{ext}  (goreleaser style)
# Tag format:   v{version}

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "env_prepend", "path_fns",
     "fetch_versions_from_github")
load("@vx//stdlib:layout.star", "archive_layout")

name        = "duf"
description = "duf - Disk usage utility with a user-friendly interface"
homepage    = "https://github.com/muesli/duf"
repository  = "https://github.com/muesli/duf"
license     = "MIT"
ecosystem   = "devtools"

runtimes = [runtime_def("duf", version_pattern="duf \\d+")]

permissions = github_permissions()

_PLATFORMS = {
    "windows/x64":   ("windows", "x86_64"),
    "windows/arm64": ("windows", "arm64"),
    "macos/x64":     ("darwin", "x86_64"),
    "macos/arm64":   ("darwin", "arm64"),
    "linux/x64":     ("linux", "x86_64"),
    "linux/arm64":   ("linux", "arm64"),
}

fetch_versions = fetch_versions_from_github("muesli", "duf")

def download_url(ctx, version):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    platform = _PLATFORMS.get(key)
    if not platform:
        return None
    os_str, arch_str = platform
    ext = "zip" if ctx.platform.os == "windows" else "tar.gz"
    return "https://github.com/muesli/duf/releases/download/v{}/duf_{}_{}_{}.{}".format(
        version, version, os_str, arch_str, ext)

install_layout = archive_layout("duf")

paths = path_fns("duf")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []
