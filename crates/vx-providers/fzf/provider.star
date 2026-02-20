# provider.star - fzf provider
#
# fzf: A command-line fuzzy finder
# Releases: https://github.com/junegunn/fzf/releases
# URL format: fzf-{version}-{os}_{arch}.{ext}
#
# Inheritance level: 2 (fetch_versions inherited, download_url overridden)
# Reason: fzf uses Go-style platform suffix (linux_amd64) instead of Rust triple

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "fzf"

def description():
    return "fzf - A command-line fuzzy finder"

def homepage():
    return "https://github.com/junegunn/fzf"

def repository():
    return "https://github.com/junegunn/fzf"

def license():
    return "MIT"

def ecosystem():
    return "devtools"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "fzf",
        "executable":  "fzf",
        "description": "A command-line fuzzy finder",
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

fetch_versions = make_fetch_versions("junegunn", "fzf")

# ---------------------------------------------------------------------------
# download_url — custom override
#
# fzf uses Go-style platform suffix: {os}_{arch}
# e.g. linux_amd64, darwin_arm64, windows_amd64
# Asset: fzf-{version}-{os}_{arch}.{ext}
# Tag:   v{version}
# ---------------------------------------------------------------------------

def _fzf_platform(ctx):
    """Map platform to fzf's Go-style platform suffix."""
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]

    platforms = {
        "windows/x64":   "windows_amd64",
        "windows/arm64": "windows_arm64",
        "macos/x64":     "darwin_amd64",
        "macos/arm64":   "darwin_arm64",
        "linux/x64":     "linux_amd64",
        "linux/arm64":   "linux_arm64",
    }
    return platforms.get("{}/{}".format(os, arch))

def download_url(ctx, version):
    """Build the fzf download URL.

    Args:
        ctx:     Provider context
        version: Version string WITHOUT 'v' prefix, e.g. "0.57.0"

    Returns:
        Download URL string, or None if platform is unsupported
    """
    platform = _fzf_platform(ctx)
    if not platform:
        return None

    os  = ctx["platform"]["os"]
    ext = "zip" if os == "windows" else "tar.gz"

    # Asset: "fzf-0.57.0-linux_amd64.tar.gz"
    asset = "fzf-{}-{}.{}".format(version, platform, ext)
    tag   = "v{}".format(version)

    return github_asset_url("junegunn", "fzf", tag, asset)


# ---------------------------------------------------------------------------
# Path queries (RFC 0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    return "{vx_home}/store/fzf"

def get_execute_path(ctx, version):
    os = ctx["platform"]["os"]
    if os == "windows":
        return "{install_dir}/fzf.exe"
    else:
        return "{install_dir}/fzf"

def post_install(ctx, version, install_dir):
    return None
