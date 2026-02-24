# provider.star - Zig programming language provider
#
# Inheritance level: Level 1 (full custom) — Zig does NOT use GitHub Releases.
# Downloads come from ziglang.org with a unique URL scheme:
#   https://ziglang.org/download/{version}/zig-{arch}-{os}-{version}.{ext}
#
# Notable differences from standard GitHub providers:
#   - Custom download host: ziglang.org (not github.com)
#   - Asset naming: zig-{arch}-{os}-{version} (arch BEFORE os, unusual)
#   - Windows uses .zip, everything else uses .tar.xz (not .tar.gz)
#   - Version fetching: uses ziglang.org/download/index.json
#   - strip_prefix in archive: zig-{arch}-{os}-{version}/

load("@vx//stdlib:http.star",     "github_releases", "releases_to_versions")
load("@vx//stdlib:platform.star", "is_windows")
load("@vx//stdlib:env.star", "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "zig"
description = "Zig - A general-purpose programming language and toolchain"
homepage    = "https://ziglang.org"
repository  = "https://github.com/ziglang/zig"
license     = "MIT"
ecosystem   = "zig"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "zig",
        "executable":  "zig",
        "description": "Zig compiler and build system",
        "aliases":     [],
        "priority":    100,
        "test_commands": [
            {"command": "{executable} version", "name": "version_check", "expected_output": "^\\d+\\.\\d+"},
            {"command": "{executable} zen", "name": "zen_check", "expected_output": "Communicate intent"},
        ],
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["ziglang.org"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# fetch_versions — custom: uses ziglang.org/download/index.json
#
# The index.json contains a dict of version -> platform assets.
# We fall back to GitHub releases for the version list since the
# stdlib github.star already handles that cleanly.
# ---------------------------------------------------------------------------

def make_fetch_versions(owner, repo, include_prereleases = False):
    """Bind fetch_versions to ziglang/zig GitHub releases."""
    def fetch_versions(ctx):
        releases = github_releases(ctx, owner, repo, include_prereleases)
        return releases_to_versions(releases)
    return fetch_versions

fetch_versions = make_fetch_versions("ziglang", "zig")

# ---------------------------------------------------------------------------
# download_url — fully custom
#
# URL format: https://ziglang.org/download/{version}/zig-{arch}-{os}-{version}.{ext}
# Examples:
#   https://ziglang.org/download/0.13.0/zig-x86_64-windows-0.13.0.zip
#   https://ziglang.org/download/0.13.0/zig-x86_64-linux-0.13.0.tar.xz
#   https://ziglang.org/download/0.13.0/zig-aarch64-macos-0.13.0.tar.xz
# ---------------------------------------------------------------------------

def _zig_arch(ctx):
    """Map vx arch to Zig's arch string."""
    arch = ctx.platform.arch
    return {
        "x64":   "x86_64",
        "arm64": "aarch64",
        "x86":   "x86",
        "arm":   "armv7a",
    }.get(arch, "x86_64")

def _zig_os(ctx):
    """Map vx OS to Zig's OS string."""
    os = ctx.platform.os
    return {
        "windows": "windows",
        "macos":   "macos",
        "linux":   "linux",
    }.get(os, "linux")

def download_url(ctx, version):
    """Build the Zig download URL.

    Args:
        ctx:     Provider context
        version: Version string WITHOUT 'v' prefix, e.g. "0.13.0"

    Returns:
        Download URL string, or None if platform is unsupported
    """
    arch = _zig_arch(ctx)
    os   = _zig_os(ctx)
    ext  = "zip" if ctx.platform.os == "windows" else "tar.xz"

    # Asset: "zig-x86_64-linux-0.13.0.tar.xz"
    # Note: arch comes BEFORE os in Zig's naming convention
    asset = "zig-{}-{}-{}.{}".format(arch, os, version, ext)

    return "https://ziglang.org/download/{}/{}".format(version, asset)

# ---------------------------------------------------------------------------
# install_layout — strip the top-level directory from the archive
#
# Zig archives contain a single top-level directory:
#   zig-x86_64-linux-0.13.0/
#     zig          ← executable
#     lib/
#     doc/
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    """Describe how to extract the Zig archive."""
    arch = _zig_arch(ctx)
    os   = _zig_os(ctx)
    exe  = "zig.exe" if ctx.platform.os == "windows" else "zig"

    # The archive contains a top-level directory to strip
    strip_prefix = "zig-{}-{}-{}".format(arch, os, version)

    return {
        "type":             "archive",
        "strip_prefix":     strip_prefix,
        "executable_paths": [exe, "zig"],
    }

# ---------------------------------------------------------------------------
# Path queries (RFC-0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    """Return the vx store root directory for zig."""
    return ctx.vx_home + "/store/zig"

def get_execute_path(ctx, version):
    """Return the executable path for the given version."""
    os = ctx.platform.os
    exe = "zig.exe" if os == "windows" else "zig"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    """No post-install actions needed for zig."""
    return None

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]
