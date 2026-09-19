# provider.star - yazi provider
#
# yazi: A blazing fast terminal file manager written in Rust
# Releases: https://github.com/sxyazi/yazi/releases
# Asset format: yazi-{triple}.zip (all platforms use .zip)
# Tag format:   v{version}
# Binary names: yazi (file manager), ya (CLI helper)
#
# Uses custom download_url because yazi uses .zip for all platforms
# and has no version in the asset filename.

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "path_fns",
     "fetch_versions_with_tag_prefix",
     "platform_map")
load("@vx//stdlib:env.star", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "yazi"
description = "yazi - A blazing fast terminal file manager"
homepage    = "https://yazi-rs.github.io"
repository  = "https://github.com/sxyazi/yazi"
license     = "MIT"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

# yazi changed the shape of `yazi --version` in 26.9.1:
#
#   <= 25.x (single line)        >= 26.9.1 (multi-line banner)
#   Yazi 25.5.31 (e7d1a2b ...)   Yazi
#                                    Version: 26.9.1 (8dd895c 2026-09-01)
#                                    Debug  : false
#                                    ...
#
# The pattern therefore allows an optional `Version:` label between the program
# name and the version number, and always requires the full X.Y[.Z] version.
runtimes = [runtime_def("yazi", aliases=["ya"],
                         version_pattern="Yazi\\s+(?:Version:\\s*)?\\d+\\.\\d+(?:\\.\\d+)?")]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# Platform mapping
# ---------------------------------------------------------------------------

_PLATFORMS = {
    "windows/x64":   "x86_64-pc-windows-msvc",
    "windows/arm64": "aarch64-pc-windows-msvc",
    "macos/x64":     "x86_64-apple-darwin",
    "macos/arm64":   "aarch64-apple-darwin",
    "linux/x64":     "x86_64-unknown-linux-musl",
    "linux/arm64":   "aarch64-unknown-linux-musl",
}

# ---------------------------------------------------------------------------
# Provider functions
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_with_tag_prefix("sxyazi", "yazi", tag_prefix = "v")

def download_url(ctx, version):
    triple = platform_map(ctx, _PLATFORMS)
    if not triple:
        return None
    return "https://github.com/sxyazi/yazi/releases/download/v{}/yazi-{}.zip".format(
        version, triple)

def install_layout(ctx, _version):
    triple = platform_map(ctx, _PLATFORMS)
    if not triple:
        return {"__type": "archive", "executable_paths": []}
    if ctx.platform.os == "windows":
        exe_paths = ["yazi.exe", "ya.exe"]
    else:
        exe_paths = ["yazi", "ya"]
    return {
        "__type": "archive",
        "strip_prefix": "yazi-{}".format(triple),
        "executable_paths": exe_paths,
    }

paths = path_fns("yazi")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []
