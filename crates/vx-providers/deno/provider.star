# provider.star - Deno provider
#
# Inheritance level: 2 (fetch_versions inherited, download_url overridden)
#
# Why override download_url?
#   - All platforms use .zip (not tar.gz)
#   - Linux uses gnu (not musl) — Deno doesn't provide musl builds
#   - Asset naming: "deno-{triple}.zip" (no version in asset name)
#
# Equivalent Rust replaced:
#   - DenoUrlBuilder::download_url() → custom download_url() below

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "deno"

def description():
    return "Deno - A modern runtime for JavaScript and TypeScript"

def homepage():
    return "https://deno.land"

def repository():
    return "https://github.com/denoland/deno"

def license():
    return "MIT"

def ecosystem():
    return "javascript"

def aliases():
    return []

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "deno",
        "executable":  "deno",
        "description": "Deno - A secure runtime for JavaScript and TypeScript",
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
# fetch_versions — fully inherited from github.star
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("denoland", "deno", include_prereleases = False)

# ---------------------------------------------------------------------------
# download_url — custom override
#
# Why override?
#   - All platforms use .zip (unlike most tools that use tar.gz on Unix)
#   - Linux uses gnu (Deno doesn't provide musl builds)
#   - Asset: "deno-{triple}.zip" — no version in asset filename
#   - Tag: "v{version}"
# ---------------------------------------------------------------------------

def _deno_triple(ctx):
    """Map platform to Deno's Rust target triple."""
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]

    triples = {
        "windows/x64":  "x86_64-pc-windows-msvc",
        "macos/x64":    "x86_64-apple-darwin",
        "macos/arm64":  "aarch64-apple-darwin",
        "linux/x64":    "x86_64-unknown-linux-gnu",   # gnu, not musl
        "linux/arm64":  "aarch64-unknown-linux-gnu",
    }
    return triples.get("{}/{}".format(os, arch))

def download_url(ctx, version):
    """Build the Deno download URL.

    All platforms use .zip. Linux uses gnu (no musl builds available).
    Asset name does NOT include version: "deno-{triple}.zip"

    Args:
        ctx:     Provider context
        version: Version string WITHOUT 'v' prefix, e.g. "2.1.4"

    Returns:
        Download URL string, or None if platform is unsupported
    """
    triple = _deno_triple(ctx)
    if not triple:
        return None

    # All platforms use zip — this is the key difference from most providers
    asset = "deno-{}.zip".format(triple)
    tag   = "v{}".format(version)

    return github_asset_url("denoland", "deno", tag, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    """Deno archives contain just the binary at root level (no subdirectory)."""
    os  = ctx["platform"]["os"]
    exe = "deno.exe" if os == "windows" else "deno"

    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": [exe, "deno"],
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {
        "PATH":      install_dir,
        "DENO_HOME": install_dir,
    }


# ---------------------------------------------------------------------------
# Path queries (RFC 0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    return "{vx_home}/store/deno"

def get_execute_path(ctx, version):
    os = ctx["platform"]["os"]
    if os == "windows":
        return "{install_dir}/deno.exe"
    else:
        return "{install_dir}/deno"

def post_install(ctx, version, install_dir):
    return None
