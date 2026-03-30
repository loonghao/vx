# provider.star - actionlint (GitHub Actions workflow linter)
#
# actionlint: Static checker for GitHub Actions workflow files
# Releases: https://github.com/rhysd/actionlint/releases
# Asset format: actionlint_{version}_{os}_{arch}.{ext}  (Go-style naming)
# Tag format:   v{version}

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "env_prepend", "path_fns",
     "fetch_versions_from_github")
load("@vx//stdlib:layout.star", "archive_layout")

name        = "actionlint"
description = "actionlint - Static checker for GitHub Actions workflow files"
homepage    = "https://github.com/rhysd/actionlint"
repository  = "https://github.com/rhysd/actionlint"
license     = "MIT"
ecosystem   = "devtools"

runtimes = [runtime_def("actionlint", version_pattern="\\d+\\.\\d+\\.\\d+")]

permissions = github_permissions()

_PLATFORMS = {
    "windows/x64":   ("windows", "amd64"),
    "windows/arm64": ("windows", "arm64"),
    "macos/x64":     ("darwin", "amd64"),
    "macos/arm64":   ("darwin", "arm64"),
    "linux/x64":     ("linux", "amd64"),
    "linux/arm64":   ("linux", "arm64"),
}

fetch_versions = fetch_versions_from_github("rhysd", "actionlint")

def download_url(ctx, version):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    platform = _PLATFORMS.get(key)
    if not platform:
        return None
    os_str, arch_str = platform
    ext = "zip" if ctx.platform.os == "windows" else "tar.gz"
    return "https://github.com/rhysd/actionlint/releases/download/v{}/actionlint_{}_{}_{}.{}".format(
        version, version, os_str, arch_str, ext)

install_layout = archive_layout("actionlint")

paths = path_fns("actionlint")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []
