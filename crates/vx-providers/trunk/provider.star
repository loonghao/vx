# provider.star - trunk provider
#
# trunk: Build, bundle, and ship Rust WASM applications to the web.
# Releases: https://github.com/trunk-rs/trunk/releases
# Asset format:
#   linux:   trunk-{triple}.tar.gz
#   macOS:   trunk-{triple}.tar.gz
#   Windows: trunk-x86_64-pc-windows-msvc.zip
# Archives contain a single trunk binary at the archive root.

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "fetch_versions_with_tag_prefix")
load("@vx//stdlib:github.star", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "trunk"
description = "Trunk - Build, bundle, and ship Rust WASM applications to the web"
homepage    = "https://trunk-rs.github.io/trunk/"
repository  = "https://github.com/trunk-rs/trunk"
license     = "Apache-2.0"
ecosystem   = "rust"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("trunk",
        version_cmd     = "{executable} --version",
        version_pattern = "trunk \\d+\\.\\d+\\.\\d+",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "trunk \\d+\\.\\d+\\.\\d+"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions - GitHub tags use v{version}
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_with_tag_prefix("trunk-rs", "trunk",
    tag_prefix = "v")

# ---------------------------------------------------------------------------
# Platform helpers
# ---------------------------------------------------------------------------

_PLATFORMS = {
    "windows/x64": ("x86_64-pc-windows-msvc", ".zip"),
    "linux/x64":   ("x86_64-unknown-linux-gnu", ".tar.gz"),
    "linux/arm64": ("aarch64-unknown-linux-gnu", ".tar.gz"),
    "macos/x64":   ("x86_64-apple-darwin", ".tar.gz"),
    "macos/arm64": ("aarch64-apple-darwin", ".tar.gz"),
}

def _trunk_platform(ctx):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return _PLATFORMS.get(key)

def _asset_name(ctx):
    platform = _trunk_platform(ctx)
    if not platform:
        return None
    triple, ext = platform[0], platform[1]
    return "trunk-{}{}".format(triple, ext)

# ---------------------------------------------------------------------------
# download_url
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    asset = _asset_name(ctx)
    if not asset:
        return None
    return github_asset_url("trunk-rs", "trunk", "v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout - archives contain trunk(.exe) at the archive root
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    if not _asset_name(ctx):
        return None
    executable = "trunk.exe" if ctx.platform.os == "windows" else "trunk"
    return {
        "__type":           "archive",
        "strip_prefix":     "",
        "executable_paths": [executable, "trunk"],
    }

# ---------------------------------------------------------------------------
# Path + env functions
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/trunk"

def get_execute_path(ctx, _version):
    executable = "trunk.exe" if ctx.platform.os == "windows" else "trunk"
    return ctx.install_dir + "/" + executable

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []
