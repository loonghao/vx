# provider.star - eza provider
#
# eza: A modern replacement for ls (community fork of exa)
# Releases: https://github.com/eza-community/eza/releases
# Asset format: eza_{triple}.{ext}  (NO version in asset name)
# Windows:      eza.exe_{triple}.{ext}
# Tag format:   v{version}
#
# Uses github_rust_provider — asset has no version, uses underscore separator.
# Windows uses eza.exe_ prefix instead of eza_.

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "env_prepend", "path_fns",
     "fetch_versions_from_github")
load("@vx//stdlib:layout.star", "archive_layout")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "eza"
description = "eza - A modern replacement for ls"
homepage    = "https://github.com/eza-community/eza"
repository  = "https://github.com/eza-community/eza"
license     = "MIT"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [runtime_def("eza", version_pattern="\\d+\\.\\d+")]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# Platform mapping
# ---------------------------------------------------------------------------

_PLATFORMS = {
    "windows/x64":   "x86_64-pc-windows-gnu",
    "macos/arm64":   "aarch64-apple-darwin",
    "linux/x64":     "x86_64-unknown-linux-gnu",
    "linux/arm64":   "aarch64-unknown-linux-gnu",
}

# ---------------------------------------------------------------------------
# Provider functions
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_from_github("eza-community", "eza")

def download_url(ctx, version):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    triple = _PLATFORMS.get(key)
    if not triple:
        return None
    if ctx.platform.os == "windows":
        ext = "zip"
        return "https://github.com/eza-community/eza/releases/download/v{}/eza.exe_{}.{}".format(
            version, triple, ext)
    else:
        ext = "tar.gz"
        return "https://github.com/eza-community/eza/releases/download/v{}/eza_{}.{}".format(
            version, triple, ext)

install_layout = archive_layout("eza")

paths = path_fns("eza")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []
