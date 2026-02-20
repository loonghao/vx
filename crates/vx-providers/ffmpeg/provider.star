# provider.star - FFmpeg provider
#
# Version source: https://github.com/GyanD/codexffmpeg/releases (pre-built Windows binaries)
#                 https://github.com/FFmpeg/FFmpeg/releases (official, Unix)
#
# Bundled runtimes: ffprobe, ffplay (included in every FFmpeg release)
#
# Inheritance pattern: Level 2 (custom download_url for platform-specific asset naming)

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:install.star", "flatten_dir")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------

def name():
    return "ffmpeg"

def description():
    return "FFmpeg - Complete solution for recording, converting and streaming audio and video"

def homepage():
    return "https://ffmpeg.org"

def repository():
    return "https://github.com/FFmpeg/FFmpeg"

def license():
    return "LGPL-2.1-or-later"

def ecosystem():
    return "system"

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    {
        "name":        "ffmpeg",
        "executable":  "ffmpeg",
        "description": "FFmpeg multimedia framework",
        "priority":    100,
    },
    {
        "name":        "ffprobe",
        "executable":  "ffprobe",
        "description": "FFprobe multimedia stream analyzer (bundled with FFmpeg)",
        "bundled_with": "ffmpeg",
    },
    {
        "name":        "ffplay",
        "executable":  "ffplay",
        "description": "FFplay simple media player (bundled with FFmpeg)",
        "bundled_with": "ffmpeg",
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
# fetch_versions — GyanD pre-built releases (Windows) / FFmpeg official (Unix)
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("GyanD", "codexffmpeg")

# ---------------------------------------------------------------------------
# download_url
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    """Build FFmpeg download URL.

    Windows: GyanD pre-built binaries (ffmpeg-{version}-essentials_build.zip)
    Linux:   Static builds from johnvansickle.com
    macOS:   evermeet.cx static builds
    """
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]

    if os == "windows":
        # GyanD Windows builds: ffmpeg-7.1-essentials_build.zip
        asset = "ffmpeg-{}-essentials_build.zip".format(version)
        return github_asset_url("GyanD", "codexffmpeg", version, asset)

    elif os == "linux" and arch == "x64":
        # John Van Sickle static builds for Linux
        return "https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-amd64-static.tar.xz"

    elif os == "linux" and arch == "arm64":
        return "https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-arm64-static.tar.xz"

    elif os == "macos":
        # evermeet.cx static builds for macOS
        return "https://evermeet.cx/ffmpeg/ffmpeg-{}.zip".format(version)

    return None

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    os = ctx["platform"]["os"]

    if os == "windows":
        # GyanD: ffmpeg-{version}-essentials_build/bin/ffmpeg.exe
        return {
            "type":             "archive",
            "strip_prefix":     "ffmpeg-{}-essentials_build".format(version),
            "executable_paths": ["bin/ffmpeg.exe", "bin/ffprobe.exe", "bin/ffplay.exe"],
        }
    elif os == "linux":
        return {
            "type":             "archive",
            "strip_prefix":     "",
            "executable_paths": ["ffmpeg", "ffprobe", "ffplay"],
        }
    else:
        return {
            "type":             "archive",
            "strip_prefix":     "",
            "executable_paths": ["ffmpeg", "ffprobe", "ffplay"],
        }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------

def environment(ctx, version, install_dir):
    os = ctx["platform"]["os"]
    if os == "windows":
        return {"PATH": install_dir + "/bin"}
    return {"PATH": install_dir}

# ---------------------------------------------------------------------------
# post_extract — flatten nested directory on Linux
#
# Linux static builds from johnvansickle.com extract to a dynamic top-level
# directory (e.g. ffmpeg-7.1-amd64-static/). We flatten it so that ffmpeg,
# ffprobe, ffplay are directly in the install root.
#
# Windows: already handled by install_layout strip_prefix.
# macOS:   evermeet.cx zip contains the binary directly, no nesting.
# ---------------------------------------------------------------------------

def post_extract(ctx, version, install_dir):
    """Flatten the nested directory structure for Linux static builds.

    johnvansickle.com archives extract to a single top-level directory
    (e.g. ffmpeg-7.1-amd64-static/) containing ffmpeg, ffprobe, ffplay
    and other files. We flatten it so executables are in the install root.

    Args:
        ctx:         Provider context
        version:     Installed version string
        install_dir: Path to the installation directory

    Returns:
        List of post-extract actions, or empty list if not Linux
    """
    os = ctx["platform"]["os"]

    # Only Linux static builds need flattening;
    # Windows uses strip_prefix in install_layout, macOS zip is already flat.
    if os == "linux":
        return [flatten_dir()]

    return []

# ---------------------------------------------------------------------------
# deps
# ---------------------------------------------------------------------------

def deps(ctx, version):
    return []


# ---------------------------------------------------------------------------
# Path queries (RFC 0037)
# ---------------------------------------------------------------------------

def store_root(ctx):
    return "{vx_home}/store/ffmpeg"

def get_execute_path(ctx, version):
    os = ctx["platform"]["os"]
    if os == "windows":
        return "{install_dir}/ffmpeg.exe"
    else:
        return "{install_dir}/ffmpeg"

def post_install(ctx, version, install_dir):
    return None
