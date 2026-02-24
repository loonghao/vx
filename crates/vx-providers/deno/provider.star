# provider.star - Deno provider
#
# Deno uses Rust triple naming but with gnu on Linux (no musl builds).
# Asset: deno-{triple}.zip (no version in asset name, all platforms use zip)
#
# Uses github_rust_provider template from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_set", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "deno"
description = "Deno - A modern runtime for JavaScript and TypeScript"
homepage    = "https://deno.land"
repository  = "https://github.com/denoland/deno"
license     = "MIT"
ecosystem   = "javascript"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("deno",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "deno \\d+\\.\\d+"},
            {"command": "{executable} eval \"console.log('ok')\"", "name": "eval_check",
             "expected_output": "ok"},
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
#
# Deno uses Rust triples but gnu on Linux (no musl builds).
# All platforms use .zip (not tar.gz).
# Asset name does NOT include version: "deno-{triple}.zip"
# ---------------------------------------------------------------------------

_DENO_TRIPLES = {
    "windows/x64":  "x86_64-pc-windows-msvc",
    "macos/x64":    "x86_64-apple-darwin",
    "macos/arm64":  "aarch64-apple-darwin",
    "linux/x64":    "x86_64-unknown-linux-gnu",
    "linux/arm64":  "aarch64-unknown-linux-gnu",
}

def _deno_triple(ctx):
    return _DENO_TRIPLES.get("{}/{}".format(ctx.platform.os, ctx.platform.arch))

# ---------------------------------------------------------------------------
# download_url — deno-{triple}.zip, tag = "v{version}"
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

def install_layout(ctx, _version):
    exe = "deno.exe" if ctx.platform.os == "windows" else "deno"
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": [exe, "deno"],
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/deno"

def get_execute_path(ctx, _version):
    exe = "deno.exe" if ctx.platform.os == "windows" else "deno"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [
        env_prepend("PATH", ctx.install_dir),
        env_set("DENO_HOME", ctx.install_dir),
    ]

def deps(_ctx, _version):
    return []
