# provider.star - just (command runner)
#
# Inheritance level: 2 — fetch_versions inherited, download_url overridden
#
# Why override download_url?
#   - just tags have NO 'v' prefix: "1.45.0" (not "v1.45.0")
#   - Asset naming: "just-{version}-{triple}.{ext}"
#   - Linux uses musl for portability
#
# GitHub: https://github.com/casey/just

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "just"

def description():
    return "just - A handy way to save and run project-specific commands"

def homepage():
    return "https://github.com/casey/just"

def repository():
    return "https://github.com/casey/just"

def license():
    return "CC0-1.0"

def ecosystem():
    return "devtools"

def aliases():
    return []

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "just",
        "executable":  "just",
        "description": "just - A handy way to save and run project-specific commands",
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
#
# just tags have NO 'v' prefix (e.g. "1.45.0"), so strip_v_prefix is a no-op.
# releases_to_versions() inside github.star handles this correctly.
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("casey", "just", include_prereleases = False)

# ---------------------------------------------------------------------------
# download_url — custom override
#
# Why override?
#   - Tag has NO 'v' prefix: "1.45.0" (not "v1.45.0")
#   - Asset: "just-{version}-{triple}.{ext}"
#   - Linux uses musl for portability
# ---------------------------------------------------------------------------

def _just_triple(ctx):
    """Map platform to just's Rust target triple."""
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]

    triples = {
        "windows/x64":   "x86_64-pc-windows-msvc",
        "windows/arm64": "aarch64-pc-windows-msvc",
        "macos/x64":     "x86_64-apple-darwin",
        "macos/arm64":   "aarch64-apple-darwin",
        "linux/x64":     "x86_64-unknown-linux-musl",
        "linux/arm64":   "aarch64-unknown-linux-musl",
        "linux/arm":     "arm-unknown-linux-musleabihf",
    }
    return triples.get("{}/{}".format(os, arch))

def download_url(ctx, version):
    """Build the just download URL.

    Args:
        ctx:     Provider context
        version: Version string WITHOUT 'v' prefix, e.g. "1.45.0"

    Returns:
        Download URL string, or None if platform is unsupported
    """
    triple = _just_triple(ctx)
    if not triple:
        return None

    os  = ctx["platform"]["os"]
    ext = "zip" if os == "windows" else "tar.gz"

    # Asset: "just-1.45.0-x86_64-unknown-linux-musl.tar.gz"
    asset = "just-{}-{}.{}".format(version, triple, ext)

    # Tag has NO 'v' prefix: "1.45.0"
    tag = version

    return github_asset_url("casey", "just", tag, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    os  = ctx["platform"]["os"]
    exe = "just.exe" if os == "windows" else "just"
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": [exe, "just"],
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {
        "PATH": install_dir,
    }
