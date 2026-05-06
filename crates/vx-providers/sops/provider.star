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
     "runtime_def", "github_permissions", "path_fns")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star", "env_prepend")
load("@vx//stdlib:system_install.star", "cross_platform_install")

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
        version_pattern = "\\d+\\.\\d+\\.\\d+",
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
    # sops uses dots as separators. Windows assets omit the os segment:
    # sops-v3.12.2.linux.amd64, sops-v3.12.2.amd64.exe
    if ctx.platform.os == "windows":
        asset = "sops-v{}.{}.exe".format(version, arch_str)
    else:
        asset = "sops-v{}.{}.{}".format(version, os_str, arch_str)
    return github_asset_url("vx-org", "mirrors", "sops-" + version, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    # sops releases are direct binaries (no archive)
    exe = "sops.exe" if ctx.platform.os == "windows" else "sops"
    return {
        "type":             "binary",
        "target_name":      exe,
        "target_dir":       "bin",
        "executable_paths": ["bin/" + exe, exe, "sops"],
    }

# ---------------------------------------------------------------------------
# Path + env functions
# ---------------------------------------------------------------------------

paths            = path_fns("sops", executable = "bin/sops")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir + "/bin")]

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []

# system_install fallback when GitHub download is unavailable
system_install = cross_platform_install(
    windows = "sops",
    macos   = "sops",
    linux   = "sops",
)
