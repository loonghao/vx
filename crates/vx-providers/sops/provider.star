# provider.star - sops provider
#
# sops is a secret management tool (Go).
#
# Release assets (GitHub releases):
#   sops-v{version}.{os}.{arch}[.exe]  (using dots as separators)
#
# OS:   linux, darwin, windows
# Arch: amd64, arm64
#
# Version source: getsops/sops releases on GitHub (tag prefix "v")

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "sops"
description = "sops - Secret operations (secrets management)"
homepage    = "https://getsops.io/"
repository  = "https://github.com/getsops/sops"
license     = "MPL-2.0"
ecosystem   = "security"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("sops",
        version_pattern = "v\\d+\\.\\d+\\.\\d+",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("getsops", "sops")

# ---------------------------------------------------------------------------
# Platform helpers
# ---------------------------------------------------------------------------

_PLATFORMS = {
    "linux/x64":   ("linux",   "amd64"),
    "linux/arm64": ("linux",   "arm64"),
    "macos/x64":   ("darwin",  "amd64"),
    "macos/arm64": ("darwin", "arm64"),
    "windows/x64": ("windows", "amd64"),
    "windows/arm64": ("windows", "arm64"),
}

def _sops_platform(ctx):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    return _PLATFORMS.get(key)

# ---------------------------------------------------------------------------
# download_url
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    platform = _sops_platform(ctx)
    if not platform:
        return None
    os_str, arch_str = platform
    # sops uses dots as separators: sops-v3.12.2.linux.amd64
    if ctx.platform.os == "windows":
        asset = "sops-v{}.{}.{}.exe".format(version, os_str, arch_str)
    else:
        asset = "sops-v{}.{}.{}".format(version, os_str, arch_str)
    return github_asset_url("getsops", "sops", "v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(_ctx, _version):
    # sops releases are direct binaries (no archive)
    return {
        "__type":        "binary",
        "target_name":   "sops",
        "target_dir":    "bin",
    }

# ---------------------------------------------------------------------------
# Path + env functions
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/sops"

def get_execute_path(ctx, _version):
    exe = "sops.exe" if ctx.platform.os == "windows" else "sops"
    return ctx.install_dir + "/" + exe

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []
