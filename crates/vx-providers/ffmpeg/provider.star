# provider.star - FFmpeg provider
#
# FFmpeg - A complete, cross-platform solution to record, convert and stream
# audio and video.
#
# Binary source (official): https://ffmpeg.org/download.html
# Mirror: https://github.com/vx-org/mirrors (permanent archive, all versions)
#
# Upstream static builds: BtbN/FFmpeg-Builds
#   Windows x64:    ffmpeg-{ver}-win64-lgpl.zip      (bin/ffmpeg.exe etc.)
#   Linux x64:      ffmpeg-{ver}-linux64-lgpl.tar.xz (bin/ffmpeg etc.)
#   Linux arm64:    ffmpeg-{ver}-linuxarm64-lgpl.tar.xz
# macOS: system_install (brew) — no reliable static build source

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
# Platform mapping
# ---------------------------------------------------------------------------

_ASSET_MAP = {
    # (os, arch): (asset_name, archive_subdir)
    "windows/x64":  ("ffmpeg-{v}-win64-lgpl.zip",          "ffmpeg-{v}-win64-lgpl"),
    "linux/x64":    ("ffmpeg-{v}-linux64-lgpl.tar.xz",     "ffmpeg-{v}-linux64-lgpl"),
    "linux/arm64":  ("ffmpeg-{v}-linuxarm64-lgpl.tar.xz",  "ffmpeg-{v}-linuxarm64-lgpl"),
}

# ---------------------------------------------------------------------------
# fetch_versions — from vx-org/mirrors tags (format: ffmpeg-{version})
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("vx-org", "mirrors", tag_prefix = "ffmpeg-")

# ---------------------------------------------------------------------------
# download_url — Windows/Linux: vx-org/mirrors; macOS: system_install
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    os   = ctx.platform.os
    arch = ctx.platform.arch

    key = os + "/" + arch
    if key not in _ASSET_MAP:
        # macOS: no static build, fall through to system_install
        return None

    asset_tpl = _ASSET_MAP[key][0]
    asset = asset_tpl.replace("{v}", version)
    return github_asset_url("vx-org", "mirrors", "ffmpeg-" + version, asset)

# ---------------------------------------------------------------------------
# install_layout — archive with bin/ subdir
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    os   = ctx.platform.os
    arch = ctx.platform.arch
    key  = os + "/" + arch

    if key not in _ASSET_MAP:
        return None

    subdir_tpl = _ASSET_MAP[key][1]
    subdir = subdir_tpl.replace("{v}", version)

    if os == "windows":
        return {
            "__type__":         "archive",
            "strip_prefix":     subdir,
            "executable_paths": ["bin/ffmpeg.exe", "bin/ffprobe.exe", "bin/ffplay.exe"],
        }
    else:
        return {
            "__type__":         "archive",
            "strip_prefix":     subdir,
            "executable_paths": ["bin/ffmpeg", "bin/ffprobe"],
        }

# ---------------------------------------------------------------------------
# Path + env
# ---------------------------------------------------------------------------

paths            = path_fns("ffmpeg")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]

def environment(ctx, _version):
    return [{"op": "prepend", "key": "PATH", "value": ctx.install_dir + "/bin"}]

def post_install(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# system_install — primary on macOS, fallback on Windows/Linux
# ---------------------------------------------------------------------------

system_install = system_install_strategies([
    brew_install("ffmpeg"),
    apt_install("ffmpeg"),
    winget_install("Gyan.FFmpeg", priority = 90),
    choco_install("ffmpeg", priority = 70),
])

def deps(_ctx, _version):
    return []
