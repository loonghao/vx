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
     "env_prepend", "path_fns",
     "fetch_versions_from_github",
     "platform_map")

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

runtimes = [runtime_def("yazi", aliases=["ya"],
                         version_pattern="Yazi \\d+")]

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

fetch_versions = fetch_versions_from_github("sxyazi", "yazi")

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
