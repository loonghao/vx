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
     "path_fns",
     "fetch_versions_with_tag_prefix")
load("@vx//stdlib:env.star", "env_prepend")

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

runtimes = [runtime_def("tealdeer", aliases=["tldr"],
                         version_pattern="tealdeer \\d+")]

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

fetch_versions = fetch_versions_with_tag_prefix("tealdeer-rs", "tealdeer", tag_prefix = "v")

def download_url(ctx, version):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    platform = _PLATFORMS.get(key)
    if not platform:
        return None
    exe = ".exe" if ctx.platform.os == "windows" else ""
    return "https://github.com/tealdeer-rs/tealdeer/releases/download/v{}/tealdeer-{}{}".format(
        version, platform, exe)

def install_layout(ctx, _version):
    """Custom binary install_layout that renames the platform-specific binary
    to the canonical 'tealdeer' (or 'tealdeer.exe') name."""
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    platform = _PLATFORMS.get(key)
    if not platform:
        return None
    exe = ".exe" if ctx.platform.os == "windows" else ""
    source = "tealdeer-" + platform + exe
    target = "tealdeer" + exe
    return {
        "source_name":      source,
        "target_name":      target,
        "target_dir":       "bin",
        "executable_paths": ["bin/" + target],
    }

paths = path_fns("tealdeer")
store_root = paths["store_root"]

def get_execute_path(ctx, _version):
    exe = "/bin/tealdeer.exe" if ctx.platform.os == "windows" else "/bin/tealdeer"
    return ctx.install_dir + exe

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir + "/bin")]

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []
