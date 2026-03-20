# provider.star - FFmpeg provider
#
# FFmpeg - A complete, cross-platform solution to record, convert and stream
# audio and video.
#
# Windows: GyanD/codexffmpeg releases (essentials build)
# Linux:   johnvansickle.com static builds
# macOS:   brew install (evermeet.cx only has Intel, uses different versioning)
#
# Uses stdlib templates from @vx//stdlib:provider.star

load("@vx//stdlib:provider.star",
     "runtime_def", "bundled_runtime_def", "github_permissions", "post_extract_flatten",
     "path_fns",
     "system_install_strategies", "brew_install", "apt_install", "choco_install",
     "winget_install")
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

permissions = github_permissions(extra_hosts = ["johnvansickle.com", "evermeet.cx"])

# ---------------------------------------------------------------------------
# fetch_versions — from GyanD/codexffmpeg (Windows) or johnvansickle.com
# ---------------------------------------------------------------------------

fetch_versions = make_fetch_versions("GyanD", "codexffmpeg")

# ---------------------------------------------------------------------------
# Platform helpers
# ---------------------------------------------------------------------------

def download_url(ctx, version):
    os   = ctx.platform.os
    arch = ctx.platform.arch
    if os == "windows":
        asset = "ffmpeg-{}-essentials_build.zip".format(version)
        return github_asset_url("GyanD", "codexffmpeg", version, asset)
    elif os == "linux":
        arch_map = {"x64": "amd64", "arm64": "arm64"}
        arch_str = arch_map.get(arch)
        if not arch_str:
            return None
        return "https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-{}-static.tar.xz".format(arch_str)
    elif os == "macos":
        # evermeet.cx uses its own versioning (not compatible with GyanD tags)
        # and only provides Intel binaries. Use system_install (brew) instead.
        return None
    return None

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------

def install_layout(ctx, version):
    os = ctx.platform.os
    if os == "windows":
        return {
            "type":             "archive",
            "strip_prefix":     "ffmpeg-{}-essentials_build".format(version),
            "executable_paths": ["bin/ffmpeg.exe", "bin/ffprobe.exe", "bin/ffplay.exe"],
        }
    # Linux static build (johnvansickle.com) does NOT include ffplay
    # (ffplay requires SDL which can't be statically linked).
    # The archive has a top-level directory like "ffmpeg-7.0-amd64-static/"
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": ["ffmpeg", "ffprobe"],
    }

# ---------------------------------------------------------------------------
# post_extract — flatten Linux static build's top-level dir
# ---------------------------------------------------------------------------

post_extract = post_extract_flatten()

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
# system_install — static dict with all platforms' strategies
# ---------------------------------------------------------------------------
# NOTE: Use static dict (not function) so parse_system_install_strategies
# can read it directly without calling. Platform filtering is handled
# automatically by the per-manager helpers (brew_install, apt_install, etc.)
# which set the "platforms" field on each strategy dict.

system_install = system_install_strategies([
    brew_install("ffmpeg"),
    apt_install("ffmpeg"),
    winget_install("Gyan.FFmpeg", priority = 90),
    choco_install("ffmpeg", priority = 70),
])

def deps(_ctx, _version):
    return []
