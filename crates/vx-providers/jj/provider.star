# provider.star - Jujutsu (jj) provider
#
# Demonstrates the Starlark "inheritance via load()" pattern:
#   - fetch_versions: fully reused from @vx//stdlib:github.star (zero custom code)
#   - download_url:   overridden because jj uses musl (not gnu) on Linux and
#                     has a specific "jj-v{version}-{triple}.{ext}" naming scheme
#
# Equivalent Rust code replaced:
#   - JjRuntime::fetch_versions()  → make_fetch_versions("jj-vcs", "jj")
#   - JjUrlBuilder::download_url() → custom download_url() below

load("@vx//stdlib:github.star",   "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:platform.star", "is_windows", "exe_ext")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "jj"

def description():
    return "Jujutsu (jj) - A Git-compatible DVCS that is both simple and powerful"

def homepage():
    return "https://github.com/jj-vcs/jj"

def repository():
    return "https://github.com/jj-vcs/jj"

def license():
    return "Apache-2.0"

def ecosystem():
    return "devtools"

def aliases():
    return ["jujutsu"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "jj",
        "executable":  "jj",
        "description": "Jujutsu - A Git-compatible DVCS",
        "aliases":     ["jujutsu"],
        "priority":    100,
    },
]

# ---------------------------------------------------------------------------
# Permissions (Deno/Buck2-inspired declarative security model)
# ---------------------------------------------------------------------------

permissions = {
    "http": ["api.github.com", "github.com"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# fetch_versions — fully inherited from github.star
#
# jj tags are "v0.38.0"; strip_v_prefix is handled by releases_to_versions()
# inside github.star via parse_github_tag(), so versions are stored as "0.38.0".
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("jj-vcs", "jj", include_prereleases = False)

# ---------------------------------------------------------------------------
# download_url — custom override
#
# Why override instead of using make_download_url()?
#   - Linux uses musl (not gnu): "x86_64-unknown-linux-musl"
#   - Asset naming: "jj-v{version}-{triple}.{ext}"
#   - URL path:     ".../v{version}/jj-v{version}-{triple}.{ext}"
#
# The version stored is WITHOUT 'v' prefix (e.g. "0.38.0"), so we add it back
# when building the tag and asset name.
# ---------------------------------------------------------------------------

def _jj_triple(ctx):
    """Map platform to jj's Rust target triple."""
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]

    triples = {
        "windows/x64":  "x86_64-pc-windows-msvc",
        "windows/arm64": "aarch64-pc-windows-msvc",
        "macos/x64":    "x86_64-apple-darwin",
        "macos/arm64":  "aarch64-apple-darwin",
        "linux/x64":    "x86_64-unknown-linux-musl",   # musl for portability
        "linux/arm64":  "aarch64-unknown-linux-musl",
    }
    return triples.get("{}/{}".format(os, arch))

def download_url(ctx, version):
    """Build the jj download URL for the given version and platform.

    Args:
        ctx:     Provider context (platform info injected by vx runtime)
        version: Version string WITHOUT 'v' prefix, e.g. "0.38.0"

    Returns:
        Download URL string, or None if platform is unsupported
    """
    triple = _jj_triple(ctx)
    if not triple:
        return None

    os  = ctx["platform"]["os"]
    ext = "zip" if os == "windows" else "tar.gz"

    # Asset: "jj-v0.38.0-x86_64-pc-windows-msvc.zip"
    asset = "jj-v{}-{}.{}".format(version, triple, ext)

    # Tag:   "v0.38.0"
    tag = "v{}".format(version)

    return github_asset_url("jj-vcs", "jj", tag, asset)

# ---------------------------------------------------------------------------
# install_layout — tells vx how to extract the downloaded archive
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    """Describe how to extract the downloaded archive.

    jj archives contain the executable directly in the root (no subdirectory).

    Returns:
        Layout dict consumed by the vx installer
    """
    os = ctx["platform"]["os"]
    exe = "jj.exe" if os == "windows" else "jj"

    return {
        "type":             "archive",
        "strip_prefix":     "",          # no subdirectory to strip
        "executable_paths": [exe, "jj"], # try platform-specific first, then fallback
    }

# ---------------------------------------------------------------------------
# environment — PATH and env vars to set after installation
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    """Return environment variables to set for this runtime.

    Args:
        ctx:         Provider context
        version:     Installed version string
        install_dir: Path where jj was installed

    Returns:
        Dict of env var name → value
    """
    return {
        "PATH": install_dir,  # prepend install_dir to PATH
    }

# ---------------------------------------------------------------------------
# constraints — runtime recommendations
# ---------------------------------------------------------------------------

constraints = [
    {
        "when":       "*",
        "recommends": [
            {
                "runtime": "git",
                "version": ">=2.41",
                "reason":  "jj uses git as backend for version control",
            },
        ],
    },
]
