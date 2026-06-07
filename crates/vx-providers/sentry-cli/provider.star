# provider.star - sentry-cli provider
#
# sentry-cli is the official Sentry command-line tool for release management,
# source map uploads, symbol management, and event exploration.
#
# Assets are standalone binaries (no archive), named:
#   sentry-cli-{OS}-{arch}[.exe]
# where OS is PascalCase (Darwin/Linux/Windows).

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "sentry-cli"
description = "Sentry CLI - official command-line tool for Sentry"
homepage    = "https://docs.sentry.io/cli/"
repository  = "https://github.com/getsentry/sentry-cli"
license     = "BSD-3-Clause"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("sentry-cli",
        version_cmd     = "{executable} --version",
        version_pattern = "\\d+\\.\\d+\\.\\d+",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions - standard GitHub tags (v prefix)
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("getsentry", "sentry-cli")

# ---------------------------------------------------------------------------
# Platform helpers
#
# sentry-cli uses PascalCase OS names and custom arch naming.
# Standalone binary: sentry-cli-{OS}-{arch}[.exe]
# ---------------------------------------------------------------------------

_PLATFORMS = {
    "macos/x64":     "Darwin-x86_64",
    "macos/arm64":   "Darwin-arm64",
    "linux/x64":     "Linux-x86_64",
    "linux/arm64":   "Linux-aarch64",
    "windows/x64":   "Windows-x86_64.exe",
    "windows/arm64": "Windows-aarch64.exe",
}

def _sentry_platform_suffix(ctx):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return _PLATFORMS.get(key)

# ---------------------------------------------------------------------------
# download_url
# Asset: sentry-cli-{platform_suffix}
#   e.g. sentry-cli-Linux-x86_64, sentry-cli-Windows-x86_64.exe
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    suffix = _sentry_platform_suffix(ctx)
    if not suffix:
        return None
    asset = "sentry-cli-{}".format(suffix)
    return github_asset_url("getsentry", "sentry-cli", version, asset)

# ---------------------------------------------------------------------------
# install_layout - standalone binary (no archive)
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    exe = "sentry-cli.exe" if ctx.platform.os == "windows" else "sentry-cli"
    suffix = _sentry_platform_suffix(ctx)
    return {
        "__type":           "binary",
        "source_name":      "sentry-cli-{}".format(suffix) if suffix else None,
        "target_name":      exe,
        "target_dir":       "bin",
        "executable_paths": ["bin/" + exe, exe, "sentry-cli"],
    }

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/sentry-cli"

def get_execute_path(ctx, _version):
    exe = "sentry-cli.exe" if ctx.platform.os == "windows" else "sentry-cli"
    return ctx.install_dir + "/bin/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir + "/bin")]

def deps(_ctx, _version):
    return []
