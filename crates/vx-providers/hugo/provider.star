# provider.star - hugo provider
#
# Hugo is a fast and flexible static site generator (written in Go).
#
# Release assets (GitHub releases):
#   hugo_{version}_Linux-64bit.tar.gz
#   hugo_{version}_Linux-ARM64.tar.gz
#   hugo_{version}_darwin-universal.pkg
#   hugo_{version}_Windows-64bit.zip
#   hugo_{version}_Windows-ARM64.zip
#
# Version source: gohugoio/hugo releases on GitHub (tag prefix "v")

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "path_fns", "path_env_fns",
     "fetch_versions_with_tag_prefix",
     "system_install_strategies", "winget_install", "brew_install", "apt_install")
load("@vx//stdlib:layout.star", "archive_layout")

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
    "macos/arm64":   ("darwin",  "universal"),
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
    if ctx.platform.os == "macos":
        # Recent Hugo releases publish macOS as .pkg installers, not tarballs.
        return None
    if ctx.platform.os == "windows":
        ext = "zip"
    else:
        ext = "tar.gz"
    return "https://github.com/gohugoio/hugo/releases/download/v{}/hugo_{}_{}-{}.{}".format(
        version, version, os_str, arch_str, ext)

# ---------------------------------------------------------------------------
# install_layout — hugo archives have no top-level dir; binary sits at root
# ---------------------------------------------------------------------------

install_layout = archive_layout("hugo")

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

paths            = path_fns("hugo")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]
env_fns          = path_env_fns()
environment      = env_fns["environment"]
post_install     = env_fns["post_install"]

def deps(_ctx, _version):
    return []

# system_install fallback when GitHub download is unavailable
# (use static form — more reliable than callable system_install)
system_install = system_install_strategies([
    winget_install("Hugo.Hugo.Extended", priority = 90),
    brew_install("hugo",                 priority = 90),
    apt_install("hugo",                  priority = 70),
])
