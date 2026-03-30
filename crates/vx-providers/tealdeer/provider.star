# provider.star - tealdeer provider
#
# tealdeer: A very fast Rust implementation of tldr (simplified man pages)
# Releases: https://github.com/tealdeer-rs/tealdeer/releases
# Asset format: tealdeer-{os}-{arch}{suffix} (single binary, no archive)
# Tag format:   v{version}
#
# Uses custom download_url because tealdeer distributes plain binaries
# with non-standard platform naming (e.g., tealdeer-linux-x86_64-musl).

load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "env_prepend", "path_fns",
     "fetch_versions_from_github")
load("@vx//stdlib:layout.star", "binary_layout")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "tealdeer"
description = "tealdeer - A very fast Rust implementation of tldr"
homepage    = "https://github.com/tealdeer-rs/tealdeer"
repository  = "https://github.com/tealdeer-rs/tealdeer"
license     = "MIT/Apache-2.0"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [runtime_def("tldr", aliases=["tealdeer"],
                         version_pattern="tealdeer v\\d+")]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# Platform mapping
# ---------------------------------------------------------------------------

_PLATFORMS = {
    "windows/x64":   "windows-x86_64-msvc",
    "macos/x64":     "macos-x86_64",
    "macos/arm64":   "macos-aarch64",
    "linux/x64":     "linux-x86_64-musl",
    "linux/arm64":   "linux-aarch64-musl",
}

# ---------------------------------------------------------------------------
# Provider functions
# ---------------------------------------------------------------------------

fetch_versions = fetch_versions_from_github("tealdeer-rs", "tealdeer")

def download_url(ctx, version):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    platform = _PLATFORMS.get(key)
    if not platform:
        return None
    exe = ".exe" if ctx.platform.os == "windows" else ""
    return "https://github.com/tealdeer-rs/tealdeer/releases/download/v{}/tealdeer-{}{}".format(
        version, platform, exe)

install_layout = binary_layout("tldr")

paths = path_fns("tealdeer")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []
