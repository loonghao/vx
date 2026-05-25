# provider.star - wasmer provider
#
# wasmer is a WebAssembly runtime and package runner. Release assets are
# archives with bin/wasmer(.exe) and supporting libraries. The separate
# wasmer-windows.exe asset is an interactive installer, not the CLI binary.

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions", "fetch_versions_with_tag_prefix")
load("@vx//stdlib:github.star", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "wasmer"
description = "Wasmer - Universal WebAssembly runtime"
homepage    = "https://wasmer.io/"
repository  = "https://github.com/wasmerio/wasmer"
license     = "MIT"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("wasmer",
        version_cmd     = "{executable} --version",
        version_pattern = "wasmer \\d+\\.\\d+\\.\\d+",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "wasmer \\d+\\.\\d+\\.\\d+"},
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

fetch_versions = fetch_versions_with_tag_prefix("wasmerio", "wasmer",
    tag_prefix = "v")

# ---------------------------------------------------------------------------
# Platform helpers
# ---------------------------------------------------------------------------

_PLATFORMS = {
    "windows/x64":  ("windows", "amd64", ".tar.gz"),
    "linux/x64":    ("linux",   "amd64", ".tar.gz"),
    "linux/arm64":  ("linux",   "aarch64", ".tar.gz"),
    "macos/x64":    ("darwin",  "amd64", ".tar.gz"),
    "macos/arm64":  ("darwin",  "arm64", ".tar.gz"),
}

def _wasmer_platform(ctx):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return _PLATFORMS.get(key)

def _asset_name(ctx):
    platform = _wasmer_platform(ctx)
    if not platform:
        return None
    os_name, arch_name, suffix = platform[0], platform[1], platform[2]
    return "wasmer-{}-{}{}".format(os_name, arch_name, suffix)

# ---------------------------------------------------------------------------
# download_url
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    asset = _asset_name(ctx)
    if not asset:
        return None
    return github_asset_url("wasmerio", "wasmer", "v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    asset = _asset_name(ctx)
    if not asset:
        return None
    executable = "wasmer.exe" if ctx.platform.os == "windows" else "wasmer"
    secondary = "wasmer" if ctx.platform.os == "windows" else "wasmer.exe"
    return {
        "__type":           "archive",
        "strip_prefix":     "",
        "executable_paths": ["bin/" + executable, executable, "bin/" + secondary, secondary],
    }

# ---------------------------------------------------------------------------
# Path + env functions
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/wasmer"

def get_execute_path(ctx, _version):
    executable = "wasmer.exe" if ctx.platform.os == "windows" else "wasmer"
    return ctx.install_dir + "/bin/" + executable

def environment(ctx, _version):
    if ctx.platform.os == "windows":
        return [
            env_prepend("PATH", ctx.install_dir + "/bin"),
            env_prepend("PATH", ctx.install_dir + "/lib"),
        ]
    return [env_prepend("PATH", ctx.install_dir + "/bin")]

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []
