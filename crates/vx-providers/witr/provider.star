# provider.star - witr provider
#
# witr: "Why is this running?" - Process introspection tool
#
# Release assets:
#   - Windows: witr-windows-{arch}.zip (contains witr.exe)
#   - macOS:   witr-darwin-{arch} (direct binary)
#   - Linux:   witr-linux-{arch}   (direct binary)
#
# Inheritance level: 1 (fully manual, no template)
# - Reason: assets are direct binaries on macOS/Linux, .zip on Windows
#

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "github_asset_url", "make_fetch_versions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "witr"
description = "witr - Why is this running? Process introspection tool"
homepage    = "https://github.com/pranshuparmar/witr"
repository  = "https://github.com/pranshuparmar/witr"
license     = "MIT"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("witr",
        version_pattern = "witr v\\d+\\.\\d+\\.\\d+",
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# Helper: map vx platform to witr asset naming
# ---------------------------------------------------------------------------

def _asset_name(os, arch):
    """Return the asset filename for the given platform."""
    os_map = {"windows": "windows", "macos": "darwin", "linux": "linux"}
    arch_map = {"x64": "amd64", "arm64": "arm64"}
    os_str = os_map.get(os)
    arch_str = arch_map.get(arch)
    if not os_str or not arch_str:
        return None
    ext = ".zip" if os == "windows" else ""
    return "witr-{}-{}{}".format(os_str, arch_str, ext)

# ---------------------------------------------------------------------------
# fetch_versions — from GitHub Releases tags
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("pranshuparmar", "witr")

# ---------------------------------------------------------------------------
# download_url — custom (asset name has no version)
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    asset = _asset_name(ctx.platform.os, ctx.platform.arch)
    if not asset:
        return None
    return github_asset_url("pranshuparmar", "witr", "v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout — handle both direct binaries and .zip
# ---------------------------------------------------------------------------

def install_layout(ctx, _version):
    """Install layout for witr."""
    os = ctx.platform.os
    target_name = "witr" + (".exe" if os == "windows" else "")

    if os == "windows":
        # .zip archive: binary inside is witr.exe
        return {
            "__type":           "archive",
            "source_name":      "witr.exe",
            "target_name":      target_name,
            "target_dir":       "",
            "executable_paths": [target_name],
        }
    else:
        # Direct binary (macOS/Linux): no extraction needed
        source_name = _asset_name(os, ctx.platform.arch)
        return {
            "__type":           "binary",
            "source_name":      source_name,
            "target_name":      target_name,
            "target_dir":       "",
            "executable_paths": [target_name],
        }

# ---------------------------------------------------------------------------
# Paths
# ---------------------------------------------------------------------------

def store_root(ctx, version):
    return "~/.vx/store/{}/{}".format(ctx.runtime.name, version)

def get_execute_path(ctx, version):
    root = store_root(ctx, version)
    exe = "witr" + (".exe" if ctx.platform.os == "windows" else "")
    return "{}/{}".format(root, exe)

# ---------------------------------------------------------------------------
# Environment
# ---------------------------------------------------------------------------

def environment(ctx, _version):
    root = store_root(ctx, ctx.version_resolved)
    return [{"op": "prepend", "key": "PATH", "value": root}]

# ---------------------------------------------------------------------------
# Post-install: vx will set executable bit automatically
# ---------------------------------------------------------------------------

def post_install(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# Dependencies (none)
# ---------------------------------------------------------------------------

def deps(_ctx, _version):
    return []
