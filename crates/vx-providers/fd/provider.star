# provider.star - fd (fd-find) provider
#
# Inheritance pattern (Level 2):
#   - fetch_versions: fully inherited from github.star
#   - download_url:   overridden — asset uses "fd-v{version}-{triple}.{ext}" naming
#
# Rust: sharkdp/fd config.rs -> ~30 lines Starlark

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")

load("@vx//stdlib:env.star", "env_prepend")
# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "fd"
description = "fd - A simple, fast and user-friendly alternative to 'find'"
homepage    = "https://github.com/sharkdp/fd"
repository  = "https://github.com/sharkdp/fd"
license     = "MIT OR Apache-2.0"
ecosystem   = "devtools"
aliases     = ["fd-find"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "fd",
        "executable":  "fd",
        "description": "A simple, fast and user-friendly alternative to 'find'",
        "aliases":     ["fd-find"],
        "priority":    100,
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check", "expected_output": "fd \\d+"},
        ],
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["api.github.com", "github.com"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# fetch_versions — fully inherited
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("sharkdp", "fd")

# ---------------------------------------------------------------------------
# download_url — override
#
# Asset naming: "fd-v{version}-{triple}.{ext}"
# Tag:          "v{version}"
# Linux uses musl for portability.
# ---------------------------------------------------------------------------

def _fd_triple(ctx):
    os   = ctx.platform.os
    arch = ctx.platform.arch
    triples = {
        "windows/x64":   "x86_64-pc-windows-msvc",
        "windows/arm64": "aarch64-pc-windows-msvc",
        "macos/x64":     "x86_64-apple-darwin",
        "macos/arm64":   "aarch64-apple-darwin",
        "linux/x64":     "x86_64-unknown-linux-musl",
        "linux/arm64":   "aarch64-unknown-linux-gnu",
    }
    return triples.get("{}/{}".format(os, arch))

def download_url(ctx, version):
    triple = _fd_triple(ctx)
    if not triple:
        return None
    os  = ctx.platform.os
    ext = "zip" if os == "windows" else "tar.gz"
    # Asset: "fd-v0.10.2-x86_64-unknown-linux-musl.tar.gz"
    asset = "fd-v{}-{}.{}".format(version, triple, ext)
    return github_asset_url("sharkdp", "fd", "v{}".format(version), asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    os  = ctx.platform.os
    triple = _fd_triple(ctx)
    exe = "fd.exe" if os == "windows" else "fd"
    # fd archives contain a subdirectory: "fd-v{version}-{triple}/"
    strip = "fd-v{}-{}".format(version, triple) if triple else ""
    return {
        "type":             "archive",
        "strip_prefix":     strip,
        "executable_paths": [exe, "fd"],
    }

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]


# ---------------------------------------------------------------------------
# Path queries (RFC 0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/fd"

def get_execute_path(ctx, _version):
    os = ctx.platform.os
    if os == "windows":
        return ctx.install_dir + "/fd.exe"
    else:
        return ctx.install_dir + "/fd"

def post_install(_ctx, _version):
    return None
