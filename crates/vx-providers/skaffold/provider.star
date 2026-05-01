# provider.star - skaffold provider
#
# skaffold is a command-line tool that facilitates continuous development
# for Kubernetes applications.
#
# Release assets are downloaded from Google Storage (not GitHub releases):
#   https://storage.googleapis.com/skaffold/releases/v{version}/skaffold-{os}-{arch}[.exe]
#
# The binary is downloaded with a platform-suffixed name (e.g. skaffold-linux-amd64).
# We rename it to plain "skaffold" using source_name/target_name in install_layout.
#
# OS:   linux, darwin, windows
# Arch: amd64, arm64
#
# Version source: GoogleContainerTools/skaffold releases on GitHub (tag prefix "v")

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "path_fns",
     "fetch_versions_with_tag_prefix")
load("@vx//stdlib:env.star", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "skaffold"
description = "skaffold - Easy and Repeatable Kubernetes Development"
homepage    = "https://skaffold.dev"
repository  = "https://github.com/GoogleContainerTools/skaffold"
license     = "Apache-2.0"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("skaffold",
        version_cmd     = "{executable} version",
        version_pattern = "v?\\d+\\.\\d+\\.\\d+",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions(extra_hosts = ["storage.googleapis.com"])

# ---------------------------------------------------------------------------
# fetch_versions - from GoogleContainerTools/skaffold releases
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_with_tag_prefix(
    "GoogleContainerTools", "skaffold", tag_prefix = "v")

# ---------------------------------------------------------------------------
# Platform helpers
# ---------------------------------------------------------------------------

_PLATFORMS = {
    "windows/x64":   ("windows", "amd64"),
    "macos/x64":     ("darwin",  "amd64"),
    "macos/arm64":   ("darwin",  "arm64"),
    "linux/x64":     ("linux",   "amd64"),
    "linux/arm64":   ("linux",   "arm64"),
}

def _skaffold_platform(ctx):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return _PLATFORMS.get(key)

# ---------------------------------------------------------------------------
# download_url - from Google Storage
# URL: https://storage.googleapis.com/skaffold/releases/v{version}/skaffold-{os}-{arch}[.exe]
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    platform = _skaffold_platform(ctx)
    if not platform:
        return None
    os_str, arch_str = platform
    ext = ".exe" if ctx.platform.os == "windows" else ""
    return "https://storage.googleapis.com/skaffold/releases/v{}/skaffold-{}-{}{}".format(
        version, os_str, arch_str, ext)

# ---------------------------------------------------------------------------
# install_layout
#
# skaffold distributes as a single binary named "skaffold-{os}-{arch}[.exe]".
# Use source_name/target_name to rename it to plain "skaffold[.exe]" in bin/.
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    platform = _skaffold_platform(ctx)
    if not platform:
        return None
    os_str, arch_str = platform
    ext = ".exe" if ctx.platform.os == "windows" else ""
    source_name = "skaffold-{}-{}{}".format(os_str, arch_str, ext)
    target_name = "skaffold" + ext
    return {
        "type":        "binary",
        "source_name": source_name,
        "target_name": target_name,
        "target_dir":  "bin",
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

paths            = path_fns("skaffold", executable = "bin/skaffold")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir + "/bin")]

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []

system_install = cross_platform_install(
    windows = "skaffold",
    macos   = "skaffold",
    linux   = "skaffold",
)
