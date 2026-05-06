# provider.star - lefthook provider
#
# lefthook: Fast and powerful Git hooks manager.
# Releases: https://github.com/evilmartians/lefthook/releases
#
# Asset format (single binary, underscore-separated, Pascal-case OS):
#   lefthook_{version}_Linux_x86_64
#   lefthook_{version}_Linux_aarch64
#   lefthook_{version}_MacOS_x86_64
#   lefthook_{version}_MacOS_arm64
#   lefthook_{version}_Windows_x86_64.exe
#   lefthook_{version}_Windows_arm64.exe
#
# Tag format: v{version}

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "lefthook"
description = "lefthook - Fast and powerful Git hooks manager for any type of projects"
homepage    = "https://lefthook.dev"
repository  = "https://github.com/evilmartians/lefthook"
license     = "MIT"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("lefthook",
        version_pattern = "lefthook version \\d+\\.\\d+",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("evilmartians", "lefthook")

# ---------------------------------------------------------------------------
# Platform helpers
#
# lefthook uses Pascal-case OS names (Linux/MacOS/Windows) and full arch names.
# Single binary: Windows has .exe; Linux/macOS have no extension.
# ---------------------------------------------------------------------------

_PLATFORMS = {
    "linux/x64":    ("Linux", "x86_64"),
    "linux/arm64":  ("Linux", "aarch64"),
    "macos/x64":    ("MacOS", "x86_64"),
    "macos/arm64":  ("MacOS", "arm64"),
    "windows/x64":  ("Windows", "x86_64"),
    "windows/arm64":("Windows", "arm64"),
}

def _lefthook_platform(ctx):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return _PLATFORMS.get(key)

# ---------------------------------------------------------------------------
# download_url
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    platform = _lefthook_platform(ctx)
    if not platform:
        return None
    os_name, arch_name = platform[0], platform[1]
    if ctx.platform.os == "windows":
        asset = "lefthook_{}_{}_{}.exe".format(version, os_name, arch_name)
    else:
        asset = "lefthook_{}_{}_{}".format(version, os_name, arch_name)
    return github_asset_url("evilmartians", "lefthook", "v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    if ctx.platform.os == "windows":
        return {
            "type":        "binary",
            "target_name": "lefthook.exe",
            "target_dir":  "bin",
        }
    return {
        "type":        "binary",
        "target_name": "lefthook",
        "target_dir":  "bin",
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/lefthook"

def get_execute_path(ctx, _version):
    exe = "lefthook.exe" if ctx.platform.os == "windows" else "lefthook"
    return ctx.install_dir + "/bin/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir + "/bin")]

def deps(_ctx, _version):
    return []
