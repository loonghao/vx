# provider.star - FFmpeg provider
#
# FFmpeg - A complete, cross-platform solution to record, convert and stream
# audio and video.
#
# Windows: Gyan.dev releases (essentials build)
# macOS:   evermeet.cx (Intel binary, use brew as fallback)
# Linux:   system package manager (apt/yum)
#
# Uses stdlib templates from @vx//stdlib:provider.star

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

permissions = github_permissions(extra_hosts = ["evermeet.cx", "www.gyan.dev"])

# ---------------------------------------------------------------------------
# fetch_versions — from GyanD/ffmpeg (use same tags for all platforms)
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("GyanD", "ffmpeg")

# ---------------------------------------------------------------------------
# download_url — Windows: Gyan.dev; macOS: evermeet.cx; Linux: None (system)
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    os = ctx.platform.os
    if os == "windows":
        # Gyan.dev: https://www.gyan.dev/ffmpeg/builds/
        # Asset: ffmpeg-{version}-essentials_build.zip
        asset = "ffmpeg-{}-essentials_build.zip".format(version)
        return github_asset_url("GyanD", "ffmpeg", version, asset)
    elif os == "darwin":
        # evermeet.cx: https://evermeet.cx/ffmpeg/
        # Asset: ffmpeg-{version}.zip (Intel only, no arm64)
        # Use system_install (brew) as primary for macOS
        return None
    elif os == "linux":
        # Use system package manager (apt/yum)
        return None
    return None

# ---------------------------------------------------------------------------
# install_layout — Windows: .zip archive; macOS/Linux: system_install
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    if ctx.platform.os == "windows":
        return {
            "type":             "archive",
            "strip_prefix":     "ffmpeg-{}-essentials_build".format(version),
            "executable_paths": ["bin/ffmpeg.exe", "bin/ffprobe.exe", "bin/ffplay.exe"],
        }
    # macOS/Linux: use system_install
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
    return [{"op": "prepend", "key": "PATH", "value": ctx.install_dir}]

def post_install(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# system_install — all platforms
# ---------------------------------------------------------------------------

system_install = system_install_strategies([
    brew_install("ffmpeg"),
    apt_install("ffmpeg"),
    winget_install("Gyan.FFmpeg", priority = 90),
    choco_install("ffmpeg", priority = 70),
])

def deps(_ctx, _version):
    return []
