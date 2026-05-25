# provider.star - wasm-pack provider
#
# wasm-pack builds and packages Rust-generated WebAssembly for JavaScript
# consumers. Release archives contain a top-level wasm-pack-v{version}-{triple}
# directory with the wasm-pack binary inside.

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "fetch_versions_with_tag_prefix")
load("@vx//stdlib:github.star", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "wasm-pack"
description = "wasm-pack - Build and package Rust-generated WebAssembly"
homepage    = "https://rustwasm.github.io/wasm-pack/"
repository  = "https://github.com/wasm-bindgen/wasm-pack"
license     = "MIT OR Apache-2.0"
ecosystem   = "rust"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("wasm-pack",
        version_cmd     = "{executable} --version",
        version_pattern = "wasm-pack \\d+\\.\\d+\\.\\d+",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "wasm-pack \\d+\\.\\d+\\.\\d+"},
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

fetch_versions = fetch_versions_with_tag_prefix("wasm-bindgen", "wasm-pack",
    tag_prefix = "v")

# ---------------------------------------------------------------------------
# Platform helpers
# ---------------------------------------------------------------------------

_PLATFORMS = {
    "windows/x64": ("x86_64-pc-windows-msvc", ".tar.gz"),
    "linux/x64":   ("x86_64-unknown-linux-musl", ".tar.gz"),
    "linux/arm64": ("aarch64-unknown-linux-musl", ".tar.gz"),
    "macos/x64":   ("x86_64-apple-darwin", ".tar.gz"),
    "macos/arm64": ("aarch64-apple-darwin", ".tar.gz"),
}

def _wasm_pack_platform(ctx):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return _PLATFORMS.get(key)

def _triple(ctx):
    platform = _wasm_pack_platform(ctx)
    return platform[0] if platform else None

def _asset_name(ctx, version):
    platform = _wasm_pack_platform(ctx)
    if not platform:
        return None
    triple, ext = platform[0], platform[1]
    return "wasm-pack-v{}-{}{}".format(version, triple, ext)

# ---------------------------------------------------------------------------
# download_url
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    asset = _asset_name(ctx, version)
    if not asset:
        return None
    return github_asset_url("wasm-bindgen", "wasm-pack", "v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout - strip top-level archive directory
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    triple = _triple(ctx)
    if not triple:
        return None
    executable = "wasm-pack.exe" if ctx.platform.os == "windows" else "wasm-pack"
    return {
        "__type":           "archive",
        "strip_prefix":     "wasm-pack-v{}-{}".format(version, triple),
        "executable_paths": [executable, "wasm-pack"],
    }

# ---------------------------------------------------------------------------
# Path + env functions
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/wasm-pack"

def get_execute_path(ctx, _version):
    executable = "wasm-pack.exe" if ctx.platform.os == "windows" else "wasm-pack"
    return ctx.install_dir + "/" + executable

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []
