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

permissions = github_permissions(extra_hosts = [])

# ---------------------------------------------------------------------------
# No fetch_versions (use system_install only)
# ---------------------------------------------------------------------------

def fetch_versions(_ctx):
    return []

# ---------------------------------------------------------------------------
# No download_url (use system_install only)
# ---------------------------------------------------------------------------

def download_url(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# No install_layout (use system_install only)
# ---------------------------------------------------------------------------

def install_layout(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# Path + env functions (for system_install)
# ---------------------------------------------------------------------------

paths            = path_fns("ffmpeg")
store_root       = paths["store_root"]
get_execute_path = paths["get_execute_path"]

def environment(_ctx, _version):
    return []

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
