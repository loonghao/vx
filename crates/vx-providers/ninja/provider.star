# provider.star - Ninja build system provider
#
# Ninja uses a non-standard platform naming scheme (win/mac/linux/linux-aarch64)
# instead of Rust target triples, so download_url is fully custom.
# fetch_versions uses jsDelivr (GitHub tag API) instead of GitHub Releases API.
#
# Asset format: ninja-{platform}.zip
# URL format:   https://github.com/ninja-build/ninja/releases/download/v{version}/ninja-{platform}.zip

load("@vx//stdlib:github.star", "github_asset_url")
load("@vx//stdlib:http.star",   "jsdelivr_versions")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "ninja"

def description():
    return "Ninja - A small build system with a focus on speed"

def homepage():
    return "https://ninja-build.org/"

def repository():
    return "https://github.com/ninja-build/ninja"

def license():
    return "Apache-2.0"

def ecosystem():
    return "build-system"

def aliases():
    return ["ninja-build"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "ninja",
        "executable":  "ninja",
        "description": "Ninja - A small build system with a focus on speed",
        "aliases":     ["ninja-build"],
        "priority":    100,
    },
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = {
    "http": ["registry.npmjs.org", "cdn.jsdelivr.net", "github.com"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# fetch_versions — jsDelivr (GitHub tag API)
#
# Ninja uses jsDelivr for version listing (same as the Rust implementation).
# ---------------------------------------------------------------------------

def fetch_versions(ctx):
    """Fetch available Ninja versions via jsDelivr GitHub tag API.

    Args:
        ctx: Provider context

    Returns:
        list of VersionInfo dicts
    """
    return jsdelivr_versions(ctx, "ninja-build", "ninja", limit = 30)

# ---------------------------------------------------------------------------
# download_url — custom (non-standard platform names)
#
# Ninja uses short platform names instead of Rust target triples:
#   windows/x64   → "win"
#   macos/*       → "mac"   (universal binary)
#   linux/x64     → "linux"
#   linux/arm64   → "linux-aarch64"
# ---------------------------------------------------------------------------

def _ninja_platform(ctx):
    """Map platform to ninja's short platform name."""
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]

    if os == "windows" and arch == "x64":
        return "win"
    elif os == "macos":
        # macOS uses a universal binary for both x64 and arm64
        return "mac"
    elif os == "linux" and arch == "x64":
        return "linux"
    elif os == "linux" and arch == "arm64":
        return "linux-aarch64"
    else:
        return None

def download_url(ctx, version):
    """Build the ninja download URL for the given version and platform.

    Args:
        ctx:     Provider context
        version: Version string WITHOUT 'v' prefix, e.g. "1.12.1"

    Returns:
        Download URL string, or None if platform is unsupported
    """
    platform = _ninja_platform(ctx)
    if not platform:
        return None

    # Asset: "ninja-linux.zip"
    asset = "ninja-{}.zip".format(platform)

    # Tag: "v1.12.1"
    tag = "v{}".format(version)

    return github_asset_url("ninja-build", "ninja", tag, asset)

# ---------------------------------------------------------------------------
# install_layout — ninja extracts directly (no subdirectory)
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    """Describe how to extract the downloaded archive.

    Ninja zip contains the executable directly in the root.

    Returns:
        Layout dict consumed by the vx installer
    """
    os  = ctx["platform"]["os"]
    exe = "ninja.exe" if os == "windows" else "ninja"

    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": [exe, "ninja"],
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {
        "PATH": install_dir,
    }
