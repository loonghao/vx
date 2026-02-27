# provider.star - Deno provider
#
# Deno - A modern runtime for JavaScript and TypeScript
# Downloads from GitHub releases (denoland/deno)
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "archive_layout", "path_fns")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend", "env_set")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "deno"
description = "Deno - A modern runtime for JavaScript and TypeScript"
homepage    = "https://deno.land"
repository  = "https://github.com/denoland/deno"
license     = "MIT"
ecosystem   = "nodejs"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("deno",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "deno \\d"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("denoland", "deno")

# ---------------------------------------------------------------------------
# Platform helpers
# Deno uses Rust target triples
# ---------------------------------------------------------------------------

_DENO_TRIPLES = {
    "windows/x64":   "x86_64-pc-windows-msvc",
    "windows/arm64": "aarch64-pc-windows-msvc",
    "macos/x64":     "x86_64-apple-darwin",
    "macos/arm64":   "aarch64-apple-darwin",
    "linux/x64":     "x86_64-unknown-linux-gnu",
    "linux/arm64":   "aarch64-unknown-linux-gnu",
}

def _deno_triple(ctx):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return _DENO_TRIPLES.get(key)

# ---------------------------------------------------------------------------
# download_url
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    triple = _deno_triple(ctx)
    if not triple:
        return None
    asset = "deno-{}.zip".format(triple)
    return github_asset_url("denoland", "deno", "v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout — zip contains single binary at root
# ---------------------------------------------------------------------------

install_layout   = archive_layout("deno")
_paths           = path_fns("deno")
store_root       = _paths["store_root"]
get_execute_path = _paths["get_execute_path"]

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [
        env_prepend("PATH", ctx.install_dir),
        env_set("DENO_HOME", ctx.install_dir),
    ]

def deps(_ctx, _version):
    return []
