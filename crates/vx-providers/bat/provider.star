# provider.star - bat provider
#
# bat: A cat clone with syntax highlighting and Git integration
# Releases: https://github.com/sharkdp/bat/releases
# Asset format: bat-v{version}-{triple}.{ext}
# Tag format:   v{version}
#
# Inheritance pattern (Level 2):
#   fetch_versions → fully inherited from github.star
#   download_url   → overridden (asset name includes v-prefix: bat-v{version}-{triple}.{ext})

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")

load("@vx//stdlib:env.star", "env_prepend")
# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "bat"
description = "A cat clone with syntax highlighting and Git integration"
homepage    = "https://github.com/sharkdp/bat"
repository  = "https://github.com/sharkdp/bat"
license     = "MIT OR Apache-2.0"
ecosystem   = "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        name,
        "executable":  name,
        "description": "A cat clone with syntax highlighting and Git integration",
        "aliases":     [],
        "priority":    100,
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["api.github.com", "github.com"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# fetch_versions — fully inherited
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("sharkdp", "bat")

# ---------------------------------------------------------------------------
# download_url — custom override
#
# Asset naming: "bat-v{version}-{triple}.{ext}"
# Tag:          "v{version}"
# Linux uses musl for portability (same as fd/jj pattern from sharkdp)
# ---------------------------------------------------------------------------

def _bat_triple(ctx):
    """Map platform to bat's Rust target triple."""
    os   = ctx.platform.os
    arch = ctx.platform.arch

    triples = {
        "windows/x64":   "x86_64-pc-windows-msvc",
        "macos/x64":     "x86_64-apple-darwin",
        "macos/arm64":   "aarch64-apple-darwin",
        "linux/x64":     "x86_64-unknown-linux-musl",
        "linux/arm64":   "aarch64-unknown-linux-gnu",
    }
    return triples.get("{}/{}".format(os, arch))

def download_url(ctx, version):
    """Build the bat download URL.

    Args:
        ctx:     Provider context
        version: Version string WITHOUT 'v' prefix, e.g. "0.24.0"

    Returns:
        Download URL string, or None if platform is unsupported
    """
    triple = _bat_triple(ctx)
    if not triple:
        return None

    os  = ctx.platform.os
    ext = "zip" if os == "windows" else "tar.gz"

    # Asset: "bat-v0.24.0-x86_64-unknown-linux-musl.tar.gz"
    asset = "bat-v{}-{}.{}".format(version, triple, ext)
    tag   = "v{}".format(version)

    return github_asset_url("sharkdp", "bat", tag, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    """bat archives contain a subdirectory: bat-v{version}-{triple}/"""
    triple = _bat_triple(ctx)
    if not triple:
        return {"type": "archive", "strip_prefix": "", "executable_paths": ["bat"]}

    os = ctx.platform.os
    ext = "zip" if os == "windows" else "tar.gz"
    exe = "bat.exe" if os == "windows" else "bat"

    # Strip the top-level directory: bat-v0.24.0-x86_64-unknown-linux-musl/
    strip_prefix = "bat-v{}-{}".format(version, triple)

    return {
        "type":             "archive",
        "strip_prefix":     strip_prefix,
        "executable_paths": [exe, "bat"],
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version):
    return [env_prepend("PATH", ctx.install_dir)]


# ---------------------------------------------------------------------------
# Path queries (RFC 0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    """Return the vx store root directory for bat."""
    return ctx.vx_home + "/store/bat"

def get_execute_path(ctx, _version):
    """Return the executable path for the given version."""
    os = ctx.platform.os
    exe = "bat.exe" if os == "windows" else "bat"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version, _install_dir):
    """Post-install hook (no-op for bat)."""
    return None
