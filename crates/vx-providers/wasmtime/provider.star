# provider.star - wasmtime provider
#
# wasmtime is the Bytecode Alliance WebAssembly runtime. Release archives use
# wasmtime-v{version}-{target}.{ext} and contain a top-level directory with the
# wasmtime binary at its root.

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "fetch_versions_with_tag_prefix")
load("@vx//stdlib:github.star", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "wasmtime"
description = "Wasmtime - Fast and secure WebAssembly runtime from the Bytecode Alliance"
homepage    = "https://wasmtime.dev/"
repository  = "https://github.com/bytecodealliance/wasmtime"
license     = "Apache-2.0 WITH LLVM-exception"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("wasmtime",
        version_cmd     = "{executable} --version",
        version_pattern = "wasmtime \\d+\\.\\d+\\.\\d+",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "wasmtime \\d+\\.\\d+\\.\\d+"},
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

fetch_versions = fetch_versions_with_tag_prefix("bytecodealliance", "wasmtime",
    tag_prefix = "v")

# ---------------------------------------------------------------------------
# Platform helpers
# ---------------------------------------------------------------------------

_PLATFORMS = {
    "windows/x64":  ("x86_64-windows", ".zip"),
    "windows/arm64": ("aarch64-windows", ".zip"),
    "linux/x64":    ("x86_64-linux", ".tar.xz"),
    "linux/arm64":  ("aarch64-linux", ".tar.xz"),
    "macos/x64":    ("x86_64-macos", ".tar.xz"),
    "macos/arm64":  ("aarch64-macos", ".tar.xz"),
}

def _wasmtime_platform(ctx):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return _PLATFORMS.get(key)

def _target(ctx):
    platform = _wasmtime_platform(ctx)
    return platform[0] if platform else None

def _asset_name(ctx, version):
    platform = _wasmtime_platform(ctx)
    if not platform:
        return None
    target, ext = platform[0], platform[1]
    return "wasmtime-v{}-{}{}".format(version, target, ext)

# ---------------------------------------------------------------------------
# download_url
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    asset = _asset_name(ctx, version)
    if not asset:
        return None
    return github_asset_url("bytecodealliance", "wasmtime", "v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    target = _target(ctx)
    if not target:
        return None
    executable = "wasmtime.exe" if ctx.platform.os == "windows" else "wasmtime"
    return {
        "__type":           "archive",
        "strip_prefix":     "wasmtime-v{}-{}".format(version, target),
        "executable_paths": [executable, "wasmtime"],
    }

# ---------------------------------------------------------------------------
# Path + env functions
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/wasmtime"

def get_execute_path(ctx, _version):
    executable = "wasmtime.exe" if ctx.platform.os == "windows" else "wasmtime"
    return ctx.install_dir + "/" + executable

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []
