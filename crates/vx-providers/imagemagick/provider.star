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

load("@vx//stdlib:env.star", "env_prepend")
# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "imagemagick"
description = "ImageMagick - Powerful image manipulation and conversion tool"
homepage    = "https://imagemagick.org"
repository  = "https://github.com/ImageMagick/ImageMagick"
license     = "ImageMagick"
ecosystem   = "system"
aliases     = ["magick"]

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
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check", "expected_output": "ImageMagick"},
        ],
    },
    {
        "name":        "convert",
        "executable":  "convert",
        "description": "ImageMagick convert tool (legacy alias for magick)",
        "bundled_with": "magick",
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check", "expected_output": "ImageMagick"},
        ],
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
    os   = ctx.platform.os
    arch = ctx.platform.arch

    if os == "linux" and arch == "x64":
        # e.g. ImageMagick--gcc-x86_64.AppImage
        asset = "ImageMagick--gcc-x86_64.AppImage"
        return github_asset_url("ImageMagick", "ImageMagick", version, asset)

    # Windows/macOS: no portable archive, use system_install
    return None

# ---------------------------------------------------------------------------
# install_layout — Linux AppImage (single binary)
# ---------------------------------------------------------------------------

def install_layout(_ctx, _version):
    return {
        "type":             "binary",
        "executable_paths": ["magick"],
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

# ---------------------------------------------------------------------------
# system_install — Windows and macOS
# ---------------------------------------------------------------------------

def system_install(ctx):
    os = ctx.platform.os
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
    return ctx.vx_home + "/store/imagemagick"

def get_execute_path(ctx, version):
    """Return the executable path for the given version."""
    os = ctx.platform.os
    exe = "magick.exe" if os == "windows" else "magick"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    """No post-install steps needed for imagemagick."""
    return None

# ---------------------------------------------------------------------------
# uninstall — delegate to system package manager on Windows/macOS
# ---------------------------------------------------------------------------

def uninstall(ctx, _version):
    """Uninstall ImageMagick.

    Linux: return False to let vx remove the AppImage store directory.
    Windows/macOS: invoke the system package manager that was used to install.
    """
    os = ctx.platform.os
    if os == "windows":
        # Try winget first, then choco, then scoop
        return {
            "type": "system_uninstall",
            "strategies": [
                {"manager": "winget", "package": "ImageMagick.ImageMagick", "priority": 95},
                {"manager": "choco",  "package": "imagemagick",             "priority": 80},
                {"manager": "scoop",  "package": "imagemagick",             "priority": 60},
            ],
        }
    elif os == "macos":
        return {
            "type": "system_uninstall",
            "strategies": [
                {"manager": "brew", "package": "imagemagick", "priority": 90},
            ],
        }
    # Linux: AppImage stored in vx store — let default directory removal handle it
    return False

# ---------------------------------------------------------------------------
# deps
# ---------------------------------------------------------------------------

def deps(_ctx, _version):
    return []
