# provider.star - Starship prompt provider
#
# Inheritance level: 2 (fetch_versions inherited, download_url overridden)
#
# Why override download_url?
#   - Asset naming: "starship-{triple}.{ext}" (NO version in asset name)
#   - URL path:     ".../v{version}/starship-{triple}.{ext}"
#   - Both Linux x64 and arm64 use musl
#
# Equivalent Rust replaced:
#   - StarshipUrlBuilder::download_url() → custom download_url() below

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "starship"

def description():
    return "Starship - The minimal, blazing-fast, and infinitely customizable prompt for any shell"

def homepage():
    return "https://starship.rs"

def repository():
    return "https://github.com/starship/starship"

def license():
    return "ISC"

def ecosystem():
    return "devtools"

def aliases():
    return []

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "starship",
        "executable":  "starship",
        "description": "The minimal, blazing-fast, and infinitely customizable prompt",
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

fetch_versions = make_fetch_versions("starship", "starship")

# ---------------------------------------------------------------------------
# download_url — custom override
#
# Key difference from standard make_download_url:
#   Asset: "starship-{triple}.{ext}"  ← NO version in asset filename
#   URL:   ".../v{version}/starship-{triple}.{ext}"
# ---------------------------------------------------------------------------

def _starship_triple(ctx):
    """Map platform to Starship's Rust target triple."""
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]

    triples = {
        "windows/x64":   "x86_64-pc-windows-msvc",
        "macos/x64":     "x86_64-apple-darwin",
        "macos/arm64":   "aarch64-apple-darwin",
        "linux/x64":     "x86_64-unknown-linux-musl",
        "linux/arm64":   "aarch64-unknown-linux-musl",
    }
    return triples.get("{}/{}".format(os, arch))

def download_url(ctx, version):
    """Build the Starship download URL.

    Args:
        ctx:     Provider context
        version: Version string WITHOUT 'v' prefix, e.g. "1.21.1"

    Returns:
        Download URL string, or None if platform is unsupported
    """
    triple = _starship_triple(ctx)
    if not triple:
        return None

    os  = ctx["platform"]["os"]
    ext = "zip" if os == "windows" else "tar.gz"

    # Asset: "starship-x86_64-pc-windows-msvc.zip"  (no version in name!)
    asset = "starship-{}.{}".format(triple, ext)
    tag   = "v{}".format(version)

    return github_asset_url("starship", "starship", tag, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    os  = ctx["platform"]["os"]
    exe = "starship.exe" if os == "windows" else "starship"
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": [exe, "starship"],
    }

# ---------------------------------------------------------------------------
# Path queries (RFC-0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    """Return the vx store root directory for starship."""
    return "{vx_home}/store/starship"

def get_execute_path(ctx, version):
    """Return the executable path for the given version."""
    os = ctx["platform"]["os"]
    exe = "starship.exe" if os == "windows" else "starship"
    return "{install_dir}/" + exe

def post_install(ctx, version, install_dir):
    """No post-install steps needed for starship."""
    return None

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {
        "PATH": install_dir,
    }
