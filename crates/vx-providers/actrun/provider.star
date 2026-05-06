# provider.star - actrun provider
#
# actrun: Actionforge workflow runner CLI
# Asset: actrun-v{version}.cli-{arch}-{os}.{ext}
# macOS uses .pkg (not supported) — Windows/Linux only
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "archive_layout", "path_fns", "path_env_fns")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "actrun"
description = "Actionforge workflow runner CLI for executing GitHub Actions-compatible workflows locally"
homepage    = "https://github.com/actionforge/actrun-cli"
repository  = "https://github.com/actionforge/actrun-cli"
license     = "Actionforge-EULA"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("actrun",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check"},
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

fetch_versions = make_fetch_versions("actionforge", "actrun-cli")

# ---------------------------------------------------------------------------
# Platform helpers
# Asset: actrun-v{version}.cli-{arch}-{os}.zip (Windows) or .pkg (macOS)
# Note: macOS uses .pkg installer, Linux uses tar.gz
# ---------------------------------------------------------------------------

_ACTRUN_PLATFORMS = {
    "windows/x64":   ("x64",   "windows", "zip"),
    "windows/arm64": ("arm64", "windows", "zip"),
    "linux/x64":     ("x64",   "linux",   "tar.gz"),
    "linux/arm64":   ("arm64", "linux",   "tar.gz"),
    "macos/x64":     ("x64",   "macos",   "pkg"),
    "macos/arm64":   ("arm64", "macos",   "pkg"),
}

def download_url(ctx, version):
    platform = _ACTRUN_PLATFORMS.get("{}/{}".format(ctx.platform.os, ctx.platform.arch))
    if not platform:
        return None
    act_arch, act_os, ext = platform[0], platform[1], platform[2]
    # macOS uses .pkg installer which requires special handling
    if ext == "pkg":
        # Use Python wheel instead for macOS (zip archive)
        asset = "actrun-v{}.py-{}-{}.zip".format(version, act_arch, act_os)
    else:
        asset = "actrun-v{}.cli-{}-{}.{}".format(version, act_arch, act_os, ext)
    return github_asset_url("vx-org", "mirrors", "actrun-" + version, asset)

# ---------------------------------------------------------------------------
# Layout + path/env functions (from stdlib)
# ---------------------------------------------------------------------------

install_layout   = archive_layout("actrun")
paths            = path_fns("actrun")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]

env_fns          = path_env_fns()
post_install     = env_fns["post_install"]
environment      = env_fns["environment"]

def deps(_ctx, _version):
    return []
