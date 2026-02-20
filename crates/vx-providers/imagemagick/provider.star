# provider.star - ImageMagick provider
#
# Version source: https://github.com/ImageMagick/ImageMagick/releases
#
# Installation strategy by platform:
#   Linux x86_64: Direct AppImage binary download from GitHub releases
#   macOS:        Homebrew (brew install imagemagick)
#   Windows:      winget / choco / scoop
#
# Bundled runtimes: convert (legacy alias, maps to magick)
#
# Inheritance pattern: Level 2 (custom download_url + system_install)

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "imagemagick"

def description():
    return "ImageMagick - Powerful image manipulation and conversion tool"

def homepage():
    return "https://imagemagick.org"

def repository():
    return "https://github.com/ImageMagick/ImageMagick"

def license():
    return "ImageMagick"

def ecosystem():
    return "system"

def aliases():
    return ["magick"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "magick",
        "executable":  "magick",
        "description": "ImageMagick unified command-line tool",
        "aliases":     ["imagemagick"],
        "priority":    100,
    },
    {
        "name":        "convert",
        "executable":  "convert",
        "description": "ImageMagick convert tool (legacy alias for magick)",
        "bundled_with": "magick",
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
# fetch_versions — ImageMagick GitHub releases
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("ImageMagick", "ImageMagick")

# ---------------------------------------------------------------------------
# download_url — Linux AppImage only; Windows/macOS use system_install
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    """Build ImageMagick download URL.

    Linux x86_64: AppImage binary from GitHub releases
    Windows/macOS: None (use system_install)
    """
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]

    if os == "linux" and arch == "x64":
        # e.g. ImageMagick--gcc-x86_64.AppImage
        asset = "ImageMagick--gcc-x86_64.AppImage"
        return github_asset_url("ImageMagick", "ImageMagick", version, asset)

    # Windows/macOS: no portable archive, use system_install
    return None

# ---------------------------------------------------------------------------
# install_layout — Linux AppImage (single binary)
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    return {
        "type":             "binary",
        "executable_paths": ["magick"],
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    return {"PATH": install_dir}

# ---------------------------------------------------------------------------
# system_install — Windows and macOS
# ---------------------------------------------------------------------------

def system_install(ctx):
    os = ctx["platform"]["os"]
    if os == "windows":
        return {
            "strategies": [
                {"manager": "winget", "package": "ImageMagick.ImageMagick", "priority": 95},
                {"manager": "choco",  "package": "imagemagick",             "priority": 80},
                {"manager": "scoop",  "package": "imagemagick",             "priority": 60},
            ],
        }
    elif os == "macos":
        return {
            "strategies": [
                {"manager": "brew", "package": "imagemagick", "priority": 90},
            ],
        }
    elif os == "linux":
        return {
            "strategies": [
                {"manager": "apt", "package": "imagemagick", "priority": 80},
                {"manager": "dnf", "package": "ImageMagick", "priority": 80},
            ],
        }
    return {}

# ---------------------------------------------------------------------------
# Path queries (RFC 0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    """Return the vx store root directory for imagemagick."""
    return "{vx_home}/store/imagemagick"

def get_execute_path(ctx, version):
    """Return the executable path for the given version."""
    os = ctx["platform"]["os"]
    exe = "magick.exe" if os == "windows" else "magick"
    return "{install_dir}/" + exe

def post_install(ctx, version, install_dir):
    """No post-install steps needed for imagemagick."""
    return None

# ---------------------------------------------------------------------------
# deps
# ---------------------------------------------------------------------------

def deps(ctx, version):
    return []
