# provider.star - ripgrep (rg) provider
#
# Inheritance pattern: Level 2 (partial override)
#   - fetch_versions: fully inherited from github.star
#   - download_url:   overridden because:
#       * release tag has NO 'v' prefix (e.g. "14.1.1", not "v14.1.1")
#       * Linux x64 uses musl, Linux arm64 uses gnu
#
# ripgrep releases: https://github.com/BurntSushi/ripgrep/releases
# Asset format: ripgrep-{version}-{triple}.{ext}

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "ripgrep"

def description():
    return "ripgrep (rg) - recursively searches directories for a regex pattern"

def homepage():
    return "https://github.com/BurntSushi/ripgrep"

def repository():
    return "https://github.com/BurntSushi/ripgrep"

def license():
    return "MIT OR Unlicense"

def ecosystem():
    return "devtools"

def aliases():
    return ["rg"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "ripgrep",
        "executable":  "rg",
        "description": "Fast regex search tool",
        "aliases":     ["rg"],
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
# ripgrep tags have NO 'v' prefix: "14.1.1", "14.0.0", ...
# parse_github_tag() inside releases_to_versions() handles both cases,
# so versions are stored as "14.1.1".
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("BurntSushi", "ripgrep")

# ---------------------------------------------------------------------------
# download_url — custom override
#
# Why override:
#   1. Tag has NO 'v' prefix → tag = version (e.g. "14.1.1")
#   2. Linux x64 uses musl, Linux arm64 uses gnu
# ---------------------------------------------------------------------------

def _rg_triple(ctx):
    """Map platform to ripgrep's Rust target triple."""
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]

    triples = {
        "windows/x64":   "x86_64-pc-windows-msvc",
        "macos/x64":     "x86_64-apple-darwin",
        "macos/arm64":   "aarch64-apple-darwin",
        "linux/x64":     "x86_64-unknown-linux-musl",   # musl for portability
        "linux/arm64":   "aarch64-unknown-linux-gnu",
    }
    return triples.get("{}/{}".format(os, arch))

def download_url(ctx, version):
    """Build the ripgrep download URL.

    Args:
        ctx:     Provider context
        version: Version string WITHOUT 'v' prefix, e.g. "14.1.1"

    Returns:
        Download URL string, or None if platform is unsupported
    """
    triple = _rg_triple(ctx)
    if not triple:
        return None

    os  = ctx["platform"]["os"]
    ext = "zip" if os == "windows" else "tar.gz"

    # Asset: "ripgrep-14.1.1-x86_64-unknown-linux-musl.tar.gz"
    asset = "ripgrep-{}-{}.{}".format(version, triple, ext)

    # Tag has NO 'v' prefix: "14.1.1"
    tag = version

    return github_asset_url("BurntSushi", "ripgrep", tag, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    triple = _rg_triple(ctx)
    os = ctx["platform"]["os"]
    exe = "rg.exe" if os == "windows" else "rg"

    # ripgrep archives contain a subdirectory: ripgrep-{version}-{triple}/
    strip_prefix = "ripgrep-{}-{}".format(version, triple) if triple else ""

    return {
        "type":             "archive",
        "strip_prefix":     strip_prefix,
        "executable_paths": [exe, "rg"],
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {
        "PATH": install_dir,
    }
