# provider.star - FFmpeg provider
#
# FFmpeg - A complete, cross-platform solution to record, convert and stream
# audio and video.
#
# Windows: vx-org/mirrors GitHub Releases (essentials build, always available)
# macOS:   system_install (brew)
# Linux:   system_install (apt)
#
# Mirror source: https://github.com/vx-org/mirrors

load("@vx//stdlib:provider.star",
     "runtime_def", "bundled_runtime_def", "github_permissions",
     "path_fns",
     "system_install_strategies", "brew_install", "apt_install",
     "choco_install", "winget_install")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "ffmpeg"
description = "FFmpeg - A complete, cross-platform solution to record, convert and stream audio and video"
homepage    = "https://ffmpeg.org"
repository  = "https://github.com/FFmpeg/FFmpeg"
license     = "LGPL-2.1"
ecosystem   = "system"
aliases     = ["avconv"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------

runtimes = [
    runtime_def("ffmpeg",
        aliases = ["avconv"],
        test_commands = [
            {"command": "{executable} -version", "name": "version_check",
             "expected_output": "ffmpeg version"},
        ],
    ),
    bundled_runtime_def("ffprobe", "ffmpeg",
        description = "FFmpeg media stream analyzer",
        test_commands = [
            {"command": "{executable} -version", "name": "version_check",
             "expected_output": "ffprobe version"},
        ],
    ),
    bundled_runtime_def("ffplay", "ffmpeg",
        description = "FFmpeg media player",
        test_commands = [
            {"command": "{executable} -version", "name": "version_check",
             "expected_output": "ffplay version"},
        ],
    ),
]

# ---------------------------------------------------------------------------
# Permissions
# ---------------------------------------------------------------------------

permissions = github_permissions()

# ---------------------------------------------------------------------------
# fetch_versions — from vx-org/mirrors tags (format: ffmpeg-{version})
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("vx-org", "mirrors",
    tag_prefix = "ffmpeg-")

# ---------------------------------------------------------------------------
# download_url — Windows: vx-org/mirrors; macOS/Linux: system_install
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    if ctx.platform.os == "windows":
        asset = "ffmpeg-{}-essentials_build.zip".format(version)
        return github_asset_url("vx-org", "mirrors", "ffmpeg-" + version, asset)
    # macOS/Linux: use system_install (brew/apt)
    return None

# ---------------------------------------------------------------------------
# install_layout — Windows: .zip archive; macOS/Linux: None (system_install)
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    if ctx.platform.os == "windows":
        return {
            "__type__":         "archive",
            "strip_prefix":     "ffmpeg-{}-essentials_build".format(version),
            "executable_paths": ["bin/ffmpeg.exe", "bin/ffprobe.exe", "bin/ffplay.exe"],
        }
    return None

# ---------------------------------------------------------------------------
# Path + env functions
# ---------------------------------------------------------------------------

paths            = path_fns("ffmpeg")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]

def environment(ctx, _version):
    if ctx.platform.os == "windows":
        return [{"op": "prepend", "key": "PATH", "value": ctx.install_dir + "/bin"}]
    return []

def post_install(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# system_install — all platforms (primary on macOS/Linux, fallback on Windows)
# ---------------------------------------------------------------------------

system_install = system_install_strategies([
    brew_install("ffmpeg"),
    apt_install("ffmpeg"),
    winget_install("Gyan.FFmpeg", priority = 90),
    choco_install("ffmpeg", priority = 70),
])

def deps(_ctx, _version):
    return []
