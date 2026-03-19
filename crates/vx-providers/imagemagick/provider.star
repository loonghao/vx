# provider.star - ImageMagick provider
#
# Linux x86_64: AppImage binary from GitHub releases
# macOS/Windows: system package manager
# Bundled runtimes: convert (legacy alias)
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "bundled_runtime_def", "github_permissions",
     "multi_platform_install", "winget_install", "choco_install",
     "scoop_install", "brew_install", "apt_install", "dnf_install")
load("@vx//stdlib:github.star", "make_fetch_versions")
load("@vx//stdlib:env.star",    "env_prepend")

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
    runtime_def("magick",
        aliases = ["imagemagick"],
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "ImageMagick"},
        ],
    ),
    bundled_runtime_def("convert", bundled_with = "magick",
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "ImageMagick"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("ImageMagick", "ImageMagick")

# ---------------------------------------------------------------------------
# download_url — Linux AppImage only; Windows/macOS use system_install
# ---------------------------------------------------------------------------
# Note: AppImage filename includes a commit hash that changes per release.
# Use system package manager on Linux instead (apt_install), or rely on
# download_url returning None to trigger system_install fallback.

def download_url(ctx, version):
    # AppImage has unpredictable hash in filename; let system_install handle it
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
# system_install — Windows and macOS
# ---------------------------------------------------------------------------

system_install = multi_platform_install(
    windows_strategies = [
        winget_install("ImageMagick.ImageMagick", priority = 95),
        choco_install("imagemagick",               priority = 80),
        scoop_install("imagemagick",               priority = 60),
    ],
    macos_strategies = [
        brew_install("imagemagick"),
    ],
    linux_strategies = [
        apt_install("imagemagick"),
        dnf_install("ImageMagick"),
    ],
)

# ---------------------------------------------------------------------------
# Path queries + environment
# ---------------------------------------------------------------------------

def store_root(ctx):
    return ctx.vx_home + "/store/imagemagick"

def get_execute_path(ctx, _version):
    exe = "magick.exe" if ctx.platform.os == "windows" else "magick"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def deps(_ctx, _version):
    return []

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
