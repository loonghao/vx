# provider.star - hugo provider
#
# Hugo is a fast and flexible static site generator (written in Go).
#
# Release assets (GitHub releases):
#   hugo_{version}_Linux-64bit.tar.gz
#   hugo_{version}_Linux-ARM64.tar.gz
#   hugo_{version}_macOS-64bit.tar.gz
#   hugo_{version}_macOS-ARM64.tar.gz
#   hugo_{version}_Windows-64bit.zip
#   hugo_{version}_Windows-ARM64.zip
#
# Version source: gohugoio/hugo releases on GitHub (tag prefix "v")

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "path_fns",
     "fetch_versions_with_tag_prefix")
load("@vx//stdlib:env.star", "env_prepend")
load("@vx//stdlib:system_install.star", "cross_platform_install")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "hugo"
description = "Hugo - Fast and flexible static site generator"
homepage    = "https://gohugo.io"
repository  = "https://github.com/gohugoio/hugo"
license     = "Apache-2.0"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("hugo",
        version_cmd     = "{executable} version",
        version_pattern = "hugo v\\d+\\.\\d+\\.\\d+",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions - from gohugoio/hugo releases
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_with_tag_prefix("gohugoio", "hugo", tag_prefix = "v")

# ---------------------------------------------------------------------------
# Platform helpers
# ---------------------------------------------------------------------------

_PLATFORMS = {
    "windows/x64":  ("windows", "amd64"),
    "windows/arm64": ("windows", "arm64"),
    "macos/x64":     ("darwin",  "universal"),
    "macos/arm64":   ("darwin", "universal"),
    "linux/x64":     ("linux",   "amd64"),
    "linux/arm64":   ("linux",   "arm64"),
}

def _hugo_platform(ctx):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return _PLATFORMS.get(key)

# ---------------------------------------------------------------------------
# download_url
#
# Hugo uses custom naming: hugo_{version}_{OS}-{Arch}.{ext}
# Archives: .tar.gz (Linux/macOS), .zip (Windows)
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    platform = _hugo_platform(ctx)
    if not platform:
        return None
    os_str, arch_str = platform
    if ctx.platform.os == "windows":
        ext = "zip"
    else:
        ext = "tar.gz"
    return "https://github.com/gohugoio/hugo/releases/download/v{}/hugo_{}_{}-{}.{}".format(
        version, version, os_str, arch_str, ext)

# ---------------------------------------------------------------------------
# install_layout — standard archive with top-level dir
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    platform = _hugo_platform(ctx)
    if not platform:
        return None
    os_str, arch_str = platform
    archive_prefix = "hugo_{}_{}-{}".format(version, os_str, arch_str)
    target_name = "hugo" + (".exe" if ctx.platform.os == "windows" else "")
    return {
        "type":          "archive",
        "source_name":   None,  # archive extraction, binary name determined by archive content
        "target_name":   target_name,
        "target_dir":    "bin",
        "archive_prefix": archive_prefix + "/",
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

paths            = path_fns("hugo", executable = "bin/hugo")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir + "/bin")]

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []

# system_install fallback when GitHub download is unavailable
system_install = cross_platform_install(
    windows = "hugo",
    macos   = "hugo",
    linux   = "hugo",
)
